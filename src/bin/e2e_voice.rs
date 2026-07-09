//! Headless voice path smoke test: two TCP sessions, UDP relay between them.
//!
//! Requires gateway and voice-node running:
//!   cargo run -p vnox-gateway -- --config dev/config.toml
//!   cargo run -p vnox-voice-node -- --config dev/config.toml

use anyhow::{Context, Result, anyhow};
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::{Duration, timeout};
use tracing::info;
use vnox_client::{
    identity,
    net::{crypto::SessionCrypto, framing, handshake, payloads::*, voice},
};

const GATEWAY: &str = "127.0.0.1:7600";
const VOICE_CHANNEL: &str = "voice";

struct Session {
    _stream: TcpStream,
    _crypto: SessionCrypto,
    udp: UdpSocket,
    voice_endpoint: std::net::SocketAddr,
}

async fn connect_client(nickname: &str) -> Result<Session> {
    let identity = identity::generate(nickname);
    let mut stream = TcpStream::connect(GATEWAY).await.context("TCP connect")?;
    let mut seq = 0u32;
    let (_session_id, _node_name, _private_mode, crypto) =
        handshake::run(&mut stream, &identity, &mut seq).await?;

    framing::io::write_encrypted(
        &mut stream,
        PID_JOIN_CHANNEL,
        &mut seq,
        &serde_json::to_vec(&JoinChannelPayload {
            channel_id: VOICE_CHANNEL.into(),
        })?,
        &crypto,
    )
    .await?;

    let pkt = framing::io::read_encrypted(&mut stream, &crypto).await?;
    if pkt.packet_id != PID_CHANNEL_STATE {
        return Err(anyhow!(
            "expected CHANNEL_STATE, got 0x{:04X}",
            pkt.packet_id
        ));
    }
    let state: ChannelStatePayload = serde_json::from_slice(&pkt.payload)?;
    let voice_endpoint: std::net::SocketAddr = state
        .voice_endpoint
        .parse()
        .context("parse voice_endpoint")?;

    let udp = UdpSocket::bind("0.0.0.0:0").await?;
    info!(
        "{nickname}: voice_endpoint={voice_endpoint} udp={}",
        udp.local_addr()?
    );

    Ok(Session {
        _stream: stream,
        _crypto: crypto,
        udp,
        voice_endpoint,
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("VNOX_LOG").unwrap_or_else(|_| "info".into()))
        .init();

    let channel_key = voice::channel_key(VOICE_CHANNEL);
    let marker = b"vnox-e2e-test-payload";

    // Dummy crypto for the test packet
    let dummy_shared = [0u8; 32];
    let dummy_crypto = SessionCrypto::derive(&dummy_shared, "test");
    let sender_pubkey = [0u8; 32];
    let packet = voice::build_packet(channel_key, 1, 0, marker, &dummy_crypto, 0, &sender_pubkey);

    let alice = connect_client("alice").await?;
    let bob = connect_client("bob").await?;

    // Register both clients on the voice node.
    alice
        .udp
        .send_to(&packet, alice.voice_endpoint)
        .await
        .context("alice register")?;
    bob.udp
        .send_to(&packet, alice.voice_endpoint)
        .await
        .context("bob register")?;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let recv = tokio::spawn(async move {
        let mut buf = vec![0u8; 1472];
        let (len, from) = bob.udp.recv_from(&mut buf).await.context("bob recv")?;
        if len >= 20 && &buf[20..len] == marker {
            info!("PASS: bob received relayed packet from {from} ({len} bytes)");
            Ok(())
        } else {
            Err(anyhow!("unexpected UDP payload (len={len})"))
        }
    });

    // Alice speaks; voice node should relay to bob.
    alice
        .udp
        .send_to(&packet, alice.voice_endpoint)
        .await
        .context("alice send")?;

    info!(
        "sent voice packet to {} (channel_key={channel_key})",
        alice.voice_endpoint
    );

    timeout(Duration::from_secs(5), recv)
        .await
        .context("timed out waiting for voice relay")?
        .context("voice recv task failed")??;

    info!("e2e voice test passed");
    Ok(())
}
