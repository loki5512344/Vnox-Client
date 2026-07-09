use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};

pub mod cipher;

/// Session encryption context (mirrors gateway/src/proto/crypto.rs).
///
/// Two derived keys:
/// - `c2s_key`: client → server
/// - `s2c_key`: server → client
///
/// Nonce: `cid (8 bytes) || packet_seq (4 bytes LE)`.
#[derive(Clone)]
pub struct SessionCrypto {
    c2s_key: [u8; 32],
    s2c_key: [u8; 32],
    cid: [u8; 8],
}

impl SessionCrypto {
    pub fn derive(shared_secret: &[u8; 32], session_id: &str) -> Self {
        let salt = b"VNOX-LNEx-KDF-v1";
        let hk = Hkdf::<Sha256>::new(Some(salt), shared_secret);

        let mut keys = [0u8; 64];
        hk.expand(b"session-keys", &mut keys)
            .expect("64 bytes within HKDF max");

        let c2s_key: [u8; 32] = keys[..32].try_into().unwrap();
        let s2c_key: [u8; 32] = keys[32..64].try_into().unwrap();

        let hash = Sha256::digest(session_id.as_bytes());
        let cid: [u8; 8] = hash[..8].try_into().unwrap();

        Self {
            c2s_key,
            s2c_key,
            cid,
        }
    }

    pub fn encrypt_c2s(&self, seq: u64, plaintext: &[u8]) -> Vec<u8> {
        cipher::encrypt_with_key(&self.c2s_key, &self.cid, seq, plaintext)
    }

    pub fn decrypt_s2c(&self, seq: u64, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        cipher::decrypt_with_key(&self.s2c_key, &self.cid, seq, ciphertext)
    }

    pub fn decrypt_c2s(&self, seq: u64, ciphertext: &[u8]) -> anyhow::Result<Vec<u8>> {
        cipher::decrypt_with_key(&self.c2s_key, &self.cid, seq, ciphertext)
    }

    pub fn c2s_key(&self) -> &[u8; 32] {
        &self.c2s_key
    }

    pub fn cid(&self) -> &[u8; 8] {
        &self.cid
    }

    pub fn new_ephemeral() -> (EphemeralSecret, PublicKey) {
        let mut rng = rand::thread_rng();
        let sk = EphemeralSecret::random_from_rng(&mut rng);
        let pk = PublicKey::from(&sk);
        (sk, pk)
    }

    pub fn ecdh(secret: EphemeralSecret, peer_public: &PublicKey) -> SharedSecret {
        secret.diffie_hellman(peer_public)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        let shared_secret = [0xABu8; 32];
        let crypto = SessionCrypto::derive(&shared_secret, "test-session");

        let plaintext = b"hello encrypted world";
        let ct = crypto.encrypt_c2s(42, plaintext);
        let decrypted = crypto.decrypt_c2s(42, &ct).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let shared_secret = [0xABu8; 32];
        let crypto = SessionCrypto::derive(&shared_secret, "test-session");

        let mut ct = crypto.encrypt_c2s(0, b"data");
        ct[5] ^= 0xFF;
        assert!(crypto.decrypt_c2s(0, &ct).is_err());
    }
}
