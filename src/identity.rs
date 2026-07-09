use anyhow::Result;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::identity_vault::Vault;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub nickname: String,
    /// hex-encoded public key (64 chars)
    pub pubkey_hex: String,
    /// hex-encoded private key — never leave device
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privkey_hex: Option<String>,
}

impl Identity {
    /// Short form of the pubkey for display (first 8 hex chars + …)
    pub fn short_id(&self) -> String {
        format!("{}…", &self.pubkey_hex[..8])
    }

    pub fn signing_key(&self) -> Option<SigningKey> {
        let hex = self.privkey_hex.as_ref()?;
        let bytes = hex::decode(hex).ok()?;
        SigningKey::from_bytes(bytes.as_slice().try_into().ok()?).into()
    }

    pub fn verifying_key(&self) -> Option<VerifyingKey> {
        let bytes = hex::decode(&self.pubkey_hex).ok()?;
        VerifyingKey::from_bytes(bytes.as_slice().try_into().ok()?).ok()
    }
}

/// Load identity from disk or generate a new one.
///
/// Tries the vault format first (`identity.vault.json`). Falls back to the
/// legacy plain `identity.json` for existing installs.
///
/// If a vault is found and is encrypted, returns
/// `LoadError::PassphraseRequired` so the caller can prompt the user.
pub fn load_or_generate() -> Result<Identity> {
    let vault_path = vault_path();
    let legacy_path = identity_path();

    if vault_path.exists() {
        let text = std::fs::read_to_string(&vault_path)?;
        let vault: Vault = serde_json::from_str(&text)?;
        if vault.is_encrypted() {
            return Err(anyhow::anyhow!(LoadError::PassphraseRequired {
                path: vault_path,
            }));
        }
        let inner = vault.open(None)?;
        let identity: Identity = serde_json::from_str(&inner)?;
        return Ok(identity);
    }

    if legacy_path.exists() {
        let text = std::fs::read_to_string(&legacy_path)?;
        let identity: Identity = serde_json::from_str(&text)?;
        return Ok(identity);
    }

    // Generate new keypair and save as plain vault.
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    let identity = Identity {
        nickname: "user".into(),
        pubkey_hex: hex::encode(verifying_key.to_bytes()),
        privkey_hex: Some(hex::encode(signing_key.to_bytes())),
    };
    save(&identity, None)?;
    Ok(identity)
}

/// Load an encrypted vault with the given passphrase.
pub fn load_with_passphrase(passphrase: &str) -> Result<Identity> {
    let vault_path = vault_path();
    let text = std::fs::read_to_string(&vault_path)?;
    let vault: Vault = serde_json::from_str(&text)?;
    let inner = vault.open(Some(passphrase))?;
    let identity: Identity = serde_json::from_str(&inner)?;
    Ok(identity)
}

/// Save identity to disk, optionally encrypting with a passphrase.
/// When `passphrase` is `None` or empty, writes a plain vault.
pub fn save(identity: &Identity, passphrase: Option<&str>) -> Result<()> {
    let path = vault_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let inner_json = serde_json::to_string_pretty(identity)?;
    let vault = Vault::seal(&inner_json, passphrase)?;
    std::fs::write(&path, serde_json::to_string_pretty(&vault)?)?;
    Ok(())
}

/// True if the on-disk vault is encrypted and requires a passphrase.
pub fn is_vault_encrypted() -> bool {
    let path = vault_path();
    if !path.exists() {
        return false;
    }
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return false,
    };
    serde_json::from_str::<Vault>(&text)
        .map(|v| v.is_encrypted())
        .unwrap_or(false)
}

/// Ephemeral identity for tests (not persisted).
pub fn generate(nickname: &str) -> Identity {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    Identity {
        nickname: nickname.into(),
        pubkey_hex: hex::encode(verifying_key.to_bytes()),
        privkey_hex: Some(hex::encode(signing_key.to_bytes())),
    }
}

/// Export the identity to a portable keyfile (JSON).
///
/// When a passphrase is provided, the keyfile is encrypted with Argon2id +
/// ChaCha20-Poly1305 (same scheme as the on-disk vault). When no passphrase
/// is provided, the keyfile is plain JSON — useful for tests but **not**
/// recommended for production use.
///
/// The returned string can be written to a `.vnoxkey` file or pasted into
/// a chat message for transfer.
pub fn export_keyfile(identity: &Identity, passphrase: Option<&str>) -> Result<String> {
    let inner = serde_json::to_string_pretty(identity)?;
    let vault = crate::identity_vault::Vault::seal(&inner, passphrase)?;
    Ok(serde_json::to_string_pretty(&vault)?)
}

/// Import an identity from a previously exported keyfile.
///
/// If the keyfile is encrypted, a passphrase must be provided. Returns the
/// deserialized identity without writing it to disk — call `save()` to persist.
pub fn import_keyfile(keyfile_json: &str, passphrase: Option<&str>) -> Result<Identity> {
    let vault: crate::identity_vault::Vault = serde_json::from_str(keyfile_json)?;
    let inner = vault.open(passphrase)?;
    let identity: Identity = serde_json::from_str(&inner)?;
    Ok(identity)
}

/// Errors that can occur during identity loading.
#[derive(Debug)]
pub enum LoadError {
    /// The vault on disk is encrypted and needs a passphrase to unlock.
    PassphraseRequired { path: PathBuf },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::PassphraseRequired { path } => {
                write!(
                    f,
                    "identity vault at {} is encrypted; passphrase required",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for LoadError {}

fn vault_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vnox")
        .join("identity.vault.json")
}

fn identity_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vnox")
        .join("identity.json")
}
