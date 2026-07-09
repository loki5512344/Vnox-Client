use anyhow::{Result, anyhow};
use tokio::net::TcpStream;
use tracing::{debug, info};

use super::crypto::SessionCrypto;
use super::framing;
use super::payloads::*;
use crate::identity::Identity;

/// Perform HELLO → AUTH → SESSION with X25519 ECDH key exchange.
///
/// Returns (session_id, node_name, private_mode, crypto).
/// All subsequent packets must be encrypted with `crypto`.
pub async fn run(
    stream: &mut TcpStream,
    identity: &Identity,
    seq: &mut u32,
) -> Result<(String, String, bool, SessionCrypto)> {
    // HELLO — receive server's ephemeral X25519 public key
    let pkt = framing::read(stream).await?;
    if pkt.packet_id != PID_HELLO {
        return Err(anyhow!("expected HELLO, got 0x{:04X}", pkt.packet_id));
    }
    let hello: HelloPayload = serde_json::from_slice(&pkt.payload)?;
    debug!("← HELLO node={}", hello.node_name);

    // Generate our own ephemeral X25519 keypair
    let (eph_sk, eph_pk) = SessionCrypto::new_ephemeral();
    let eph_pk_hex = hex::encode(eph_pk.as_bytes());

    // Parse server's ephemeral key from HELLO
    let server_eph_raw =
        hex::decode(&hello.server_eph_pubkey).map_err(|_| anyhow!("bad server_eph_pubkey hex"))?;
    let server_eph_key: [u8; 32] = server_eph_raw
        .try_into()
        .map_err(|_| anyhow!("bad server_eph_pubkey length"))?;
    let server_eph_pk = x25519_dalek::PublicKey::from(server_eph_key);

    // AUTH — include our ephemeral public key
    let nonce = hex::decode(&hello.challenge_nonce)?;
    let sk = identity
        .signing_key()
        .ok_or_else(|| anyhow!("no signing key"))?;
    use ed25519_dalek::Signer;
    let sig = sk.sign(&nonce);

    let auth = AuthPayload {
        client_pubkey: identity.pubkey_hex.clone(),
        nickname: identity.nickname.clone(),
        lnex_version: LNEX_VERSION.into(),
        signature: hex::encode(sig.to_bytes()),
        client_eph_pubkey: eph_pk_hex,
    };
    framing::write(stream, PID_AUTH, seq, &serde_json::to_vec(&auth)?).await?;
    debug!("→ AUTH nick={}", identity.nickname);

    // SESSION
    let pkt = framing::read(stream).await?;
    if pkt.packet_id == PID_ERROR {
        let e: ErrorPayload = serde_json::from_slice(&pkt.payload)?;
        return Err(anyhow!("auth rejected: {}", e.message));
    }
    if pkt.packet_id != PID_SESSION {
        return Err(anyhow!("expected SESSION, got 0x{:04X}", pkt.packet_id));
    }
    let sess: SessionPayload = serde_json::from_slice(&pkt.payload)?;

    // Derive encryption keys from ECDH shared secret
    let shared_secret = SessionCrypto::ecdh(eph_sk, &server_eph_pk);
    let crypto = SessionCrypto::derive(shared_secret.as_bytes(), &sess.session_id);

    info!(
        "authenticated — session {} encrypted",
        &sess.session_id[..8]
    );

    Ok((sess.session_id, hello.node_name, hello.private_mode, crypto))
}
