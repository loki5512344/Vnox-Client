use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::warn;

use super::super::crypto::SessionCrypto;
use super::super::payloads::PID_VOICE_PACKET;
use super::super::types::NetEvent;

/// Spawn a task that reads UDP datagrams, decrypts them, and forwards as VoicePacket events.
pub fn spawn_recv(udp: Arc<UdpSocket>, crypto: SessionCrypto, tx: mpsc::Sender<NetEvent>) {
    tokio::spawn(async move {
        let mut buf = vec![0u8; 1472];
        let mut voice_seq: u64 = 0;

        loop {
            match udp.recv_from(&mut buf).await {
                Ok((len, src)) => {
                    if len < 4 {
                        continue;
                    }

                    let packet_id = u16::from_be_bytes([buf[0], buf[1]]);
                    if packet_id != PID_VOICE_PACKET {
                        continue;
                    }

                    let encrypted_payload = &buf[4..len];
                    let plaintext = match crypto.decrypt_s2c(voice_seq, encrypted_payload) {
                        Ok(p) => p,
                        Err(e) => {
                            warn!("voice packet decryption failed from {src}: {e}");
                            voice_seq += 1;
                            continue;
                        }
                    };

                    // Layout: [voice_seq:4][timestamp:4][channel_id:8][sender_id:32][opus]
                    // Legacy format (no sender_id) is 16+ bytes — gracefully handle.
                    if plaintext.len() < 16 {
                        warn!("voice packet payload too short from {src}");
                        voice_seq += 1;
                        continue;
                    }

                    let pkt_voice_seq = u32::from_be_bytes(plaintext[0..4].try_into().unwrap());
                    let timestamp = u32::from_be_bytes(plaintext[4..8].try_into().unwrap());
                    let channel_id = u64::from_be_bytes(plaintext[8..16].try_into().unwrap());

                    // sender_id: 32 raw bytes if present, hex-encoded.
                    // Empty string when packet is from legacy sender without sender_id.
                    let (sender_id_hex, opus_data) = if plaintext.len() >= 48 {
                        let sender_bytes: [u8; 32] = plaintext[16..48].try_into().unwrap();
                        (hex::encode(sender_bytes), plaintext[48..].to_vec())
                    } else {
                        // Legacy: no sender_id in payload.
                        (String::new(), plaintext[16..].to_vec())
                    };

                    let _ = tx
                        .send(NetEvent::VoicePacket {
                            channel_id,
                            voice_seq: pkt_voice_seq,
                            timestamp,
                            sender: src,
                            sender_id: sender_id_hex,
                            opus_data,
                        })
                        .await;

                    voice_seq += 1;
                }
                Err(e) => {
                    warn!("UDP recv: {e}");
                    break;
                }
            }
        }
    });
}
