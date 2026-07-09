//! Encrypted vault for the identity keypair.
//!
//! Format on disk (JSON):
//! ```json
//! {
//!   "version": 1,
//!   "scheme": "argon2id+chacha20-poly1305",
//!   "salt": "<hex 16 bytes>",
//!   "nonce": "<hex 12 bytes>",
//!   "ciphertext": "<hex, plaintext is the inner Identity JSON>"
//! }
//! ```
//!
//! When `passphrase` is `None`, the identity is stored as plain JSON (legacy behavior).

use anyhow::Result;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, aead::Aead};
use serde::{Deserialize, Serialize};
const VAULT_VERSION: u32 = 1;
const VAULT_SCHEME: &str = "argon2id+chacha20-poly1305";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub version: u32,
    pub scheme: String,
    /// Hex-encoded salt for Argon2id.
    #[serde(default)]
    pub salt: String,
    /// Hex-encoded ChaCha20-Poly1305 nonce.
    #[serde(default)]
    pub nonce: String,
    /// Hex-encoded ciphertext (encrypts the inner Identity JSON).
    /// Empty when no passphrase is set.
    #[serde(default)]
    pub ciphertext: String,
    /// Plaintext Identity JSON — only set when no passphrase is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plaintext: Option<String>,
}

impl Vault {
    /// Seal the given JSON-serialized identity with a passphrase.
    /// If `passphrase` is empty, stores as plaintext (no encryption).
    pub fn seal(identity_json: &str, passphrase: Option<&str>) -> Result<Self> {
        if passphrase.map(|p| p.is_empty()).unwrap_or(true) {
            return Ok(Self {
                version: VAULT_VERSION,
                scheme: "plain".into(),
                salt: String::new(),
                nonce: String::new(),
                ciphertext: String::new(),
                plaintext: Some(identity_json.into()),
            });
        }
        let mut salt = [0u8; SALT_LEN];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut salt);
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce_bytes);

        let key = derive_key(passphrase.unwrap(), &salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("chacha key init: {e}"))?;
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
        let ct = cipher
            .encrypt(nonce, identity_json.as_bytes())
            .map_err(|e| anyhow::anyhow!("encrypt identity: {e:?}"))?;

        Ok(Self {
            version: VAULT_VERSION,
            scheme: VAULT_SCHEME.into(),
            salt: hex::encode(salt),
            nonce: hex::encode(nonce_bytes),
            ciphertext: hex::encode(&ct),
            plaintext: None,
        })
    }

    /// Open the vault and return the inner Identity JSON.
    /// If `passphrase` is None but the vault is encrypted, returns an error.
    pub fn open(&self, passphrase: Option<&str>) -> Result<String> {
        if self.scheme == "plain" {
            return self
                .plaintext
                .clone()
                .ok_or_else(|| anyhow::anyhow!("plain vault has no plaintext"));
        }
        let passphrase = passphrase
            .ok_or_else(|| anyhow::anyhow!("vault is encrypted but no passphrase provided"))?;
        let salt = hex::decode(&self.salt).map_err(|e| anyhow::anyhow!("bad vault salt: {e}"))?;
        let nonce_bytes =
            hex::decode(&self.nonce).map_err(|e| anyhow::anyhow!("bad vault nonce: {e}"))?;
        let ct = hex::decode(&self.ciphertext)
            .map_err(|e| anyhow::anyhow!("bad vault ciphertext: {e}"))?;
        let key = derive_key(passphrase, &salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("chacha key init: {e}"))?;
        let nonce = chacha20poly1305::Nonce::from_slice(&nonce_bytes);
        let pt = cipher
            .decrypt(nonce, ct.as_ref())
            .map_err(|_| anyhow::anyhow!("wrong passphrase or corrupted vault"))?;
        String::from_utf8(pt).map_err(|e| anyhow::anyhow!("vault plaintext is not UTF-8: {e}"))
    }

    pub fn is_encrypted(&self) -> bool {
        self.scheme != "plain"
    }
}

fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; KEY_LEN]> {
    // m=64 MiB, t=3, p=4 — moderate cost, ~250 ms on a modern CPU.
    let params = Params::new(64 * 1024, 3, 4, Some(KEY_LEN))
        .map_err(|e| anyhow::anyhow!("argon2 params: {e}"))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; KEY_LEN];
    argon
        .hash_password_into(passphrase.as_bytes(), salt, &mut out)
        .map_err(|e| anyhow::anyhow!("argon2 derive: {e}"))?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypted() {
        let inner = r#"{"nickname":"alice","pubkey_hex":"abcd","privkey_hex":"0011"}"#;
        let v = Vault::seal(inner, Some("hunter2")).unwrap();
        assert!(v.is_encrypted());
        let opened = v.open(Some("hunter2")).unwrap();
        assert_eq!(opened, inner);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let v = Vault::seal("secret", Some("hunter2")).unwrap();
        assert!(v.open(Some("wrong")).is_err());
    }

    #[test]
    fn plaintext_roundtrip() {
        let inner = r#"{"nickname":"bob"}"#;
        let v = Vault::seal(inner, None).unwrap();
        assert!(!v.is_encrypted());
        let opened = v.open(None).unwrap();
        assert_eq!(opened, inner);
    }

    #[test]
    fn empty_passphrase_treated_as_plain() {
        let v = Vault::seal("x", Some("")).unwrap();
        assert!(!v.is_encrypted());
    }
}
