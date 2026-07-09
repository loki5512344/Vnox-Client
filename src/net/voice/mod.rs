use super::crypto::SessionCrypto;
use super::payloads::PID_VOICE_PACKET;

mod receive;
pub use receive::spawn_recv;

/// Stable string channel id to u64 key for UDP voice packets.
pub fn channel_key(id: &str) -> u64 {
    let mut h: u64 = 5381;
    for b in id.bytes() {
        h = h.wrapping_mul(33).wrapping_add(u64::from(b));
    }
    h
}

/// Build a UDP voice packet with encryption.
///
/// Plaintext layout (Phase 1.2+):
///   `[voice_seq:4][timestamp:4][channel_id:8][sender_id:32][opus_data:N]`
///
/// `sender_id` is the raw Ed25519 public key of the sender, so receivers can
/// attribute the voice activity to a specific user (Phase 1.3 speaking indicators).
pub fn build_packet(
    channel_id: u64,
    voice_seq: u32,
    timestamp: u32,
    opus: &[u8],
    crypto: &SessionCrypto,
    packet_seq: u64,
    sender_pubkey: &[u8; 32],
) -> Vec<u8> {
    let mut plaintext = Vec::with_capacity(16 + 32 + opus.len());
    plaintext.extend_from_slice(&voice_seq.to_be_bytes());
    plaintext.extend_from_slice(&timestamp.to_be_bytes());
    plaintext.extend_from_slice(&channel_id.to_be_bytes());
    plaintext.extend_from_slice(sender_pubkey);
    plaintext.extend_from_slice(opus);

    let encrypted = crypto.encrypt_c2s(packet_seq, &plaintext);

    let mut buf = Vec::with_capacity(4 + encrypted.len());
    buf.extend_from_slice(&PID_VOICE_PACKET.to_be_bytes());
    buf.extend_from_slice(&0u16.to_be_bytes());
    buf.extend_from_slice(&encrypted);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_key_is_stable() {
        assert_eq!(channel_key("voice"), channel_key("voice"));
        assert_ne!(channel_key("voice"), channel_key("general"));
    }

    #[test]
    fn build_packet_header_layout() {
        let key = channel_key("voice");
        let shared = [0u8; 32];
        let crypto = SessionCrypto::derive(&shared, "test");
        let sender = [0xABu8; 32];
        let pkt = build_packet(key, 42, 1000, b"opus", &crypto, 0, &sender);

        let packet_id = u16::from_be_bytes([pkt[0], pkt[1]]);
        assert_eq!(packet_id, PID_VOICE_PACKET);

        let encrypted_payload = &pkt[4..];
        let plaintext = crypto.decrypt_c2s(0, encrypted_payload).unwrap();
        assert_eq!(plaintext.len(), 16 + 32 + 4);

        let pkt_voice_seq = u32::from_be_bytes(plaintext[0..4].try_into().unwrap());
        let timestamp = u32::from_be_bytes(plaintext[4..8].try_into().unwrap());
        let channel_id = u64::from_be_bytes(plaintext[8..16].try_into().unwrap());
        let sender_id: [u8; 32] = plaintext[16..48].try_into().unwrap();

        assert_eq!(pkt_voice_seq, 42);
        assert_eq!(timestamp, 1000);
        assert_eq!(channel_id, key);
        assert_eq!(sender_id, sender);
        assert_eq!(&plaintext[48..], b"opus");
    }
}
