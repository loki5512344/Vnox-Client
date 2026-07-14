use chacha20poly1305::aead::Aead;
use chacha20poly1305::{ChaCha20Poly1305, KeyInit, Nonce};
use hkdf::Hkdf;
use sha2::{Digest, Sha256, Sha512};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::identity::Identity;

pub fn derive_e2ee_keypair(identity: &Identity) -> (StaticSecret, PublicKey) {
    let sk = identity.signing_key().expect("signing key required");
    let hash = Sha512::digest(sk.to_bytes());
    let mut e2ee_seed = [0u8; 32];
    e2ee_seed.copy_from_slice(&hash[..32]);
    let static_secret = StaticSecret::from(e2ee_seed);
    let public = PublicKey::from(&static_secret);
    (static_secret, public)
}

pub fn compute_shared_secret(my_secret: &StaticSecret, peer_public_bytes: &[u8]) -> [u8; 32] {
    let mut peer_pk = [0u8; 32];
    peer_pk.copy_from_slice(peer_public_bytes);
    let peer_public = PublicKey::from(peer_pk);
    let shared = my_secret.diffie_hellman(&peer_public);
    let hk = Hkdf::<Sha256>::new(Some(b"VNOX-E2EE-v1"), shared.as_bytes());
    let mut msg_key = [0u8; 32];
    hk.expand(b"dm-key", &mut msg_key).expect("32 bytes");
    msg_key
}

pub fn encrypt_message(msg_key: &[u8; 32], message_id: &str, plaintext: &str) -> Vec<u8> {
    let cipher = ChaCha20Poly1305::new_from_slice(msg_key).unwrap();
    let hash = Sha256::digest(message_id.as_bytes());
    let nonce = Nonce::from_slice(&hash[..12]);
    cipher.encrypt(nonce, plaintext.as_bytes()).unwrap()
}

pub fn decrypt_message(msg_key: &[u8; 32], message_id: &str, ciphertext: &[u8]) -> String {
    let cipher = ChaCha20Poly1305::new_from_slice(msg_key).unwrap();
    let hash = Sha256::digest(message_id.as_bytes());
    let nonce = Nonce::from_slice(&hash[..12]);
    let plaintext = cipher.decrypt(nonce, ciphertext).unwrap();
    String::from_utf8(plaintext).unwrap()
}
