use anyhow::{Result, anyhow};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use super::payloads::{HDR_SIZE, MAX_PAYLOAD};

pub struct RawPacket {
    pub packet_id: u16,
    pub payload: Vec<u8>,
}

pub(super) fn parse_header(buf: &[u8; HDR_SIZE]) -> Result<(u16, u16, u32, u32)> {
    let packet_id = u16::from_be_bytes([buf[0], buf[1]]);
    let flags = u16::from_be_bytes([buf[2], buf[3]]);
    let sequence = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
    let payload_length = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
    if payload_length > MAX_PAYLOAD {
        return Err(anyhow!("payload too large: {payload_length}"));
    }
    Ok((packet_id, flags, sequence, payload_length))
}

pub fn encode_frame(id: u16, seq: &mut u32, payload: &[u8]) -> Vec<u8> {
    let mut hdr = [0u8; HDR_SIZE];
    hdr[0..2].copy_from_slice(&id.to_be_bytes());
    hdr[4..8].copy_from_slice(&seq.to_be_bytes());
    hdr[8..12].copy_from_slice(&(payload.len() as u32).to_be_bytes());
    *seq = seq.wrapping_add(1);
    let mut frame = Vec::with_capacity(HDR_SIZE + payload.len());
    frame.extend_from_slice(&hdr);
    frame.extend_from_slice(payload);
    frame
}

pub fn decode_frame(bytes: &[u8]) -> Result<RawPacket> {
    if bytes.len() < HDR_SIZE {
        return Err(anyhow!("frame too short"));
    }
    let hdr: [u8; HDR_SIZE] = bytes[..HDR_SIZE].try_into().expect("header length");
    let (packet_id, _, _, payload_length) = parse_header(&hdr)?;
    let end = HDR_SIZE + payload_length as usize;
    if bytes.len() < end {
        return Err(anyhow!("truncated frame"));
    }
    Ok(RawPacket {
        packet_id,
        payload: bytes[HDR_SIZE..end].to_vec(),
    })
}

pub async fn read(stream: &mut TcpStream) -> Result<RawPacket> {
    let mut buf = [0u8; HDR_SIZE];
    stream.read_exact(&mut buf).await?;
    let (packet_id, _, _, payload_length) = parse_header(&buf)?;
    let mut payload = vec![0u8; payload_length as usize];
    if !payload.is_empty() {
        stream.read_exact(&mut payload).await?;
    }
    Ok(RawPacket { packet_id, payload })
}

pub async fn write(stream: &mut TcpStream, id: u16, seq: &mut u32, payload: &[u8]) -> Result<()> {
    let frame = encode_frame(id, seq, payload);
    stream.write_all(&frame).await?;
    Ok(())
}

pub mod io;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::payloads::PID_PING;

    #[test]
    fn encode_decode_roundtrip() {
        let payload = br#"{"ping":true}"#;
        let mut seq = 7u32;
        let frame = encode_frame(PID_PING, &mut seq, payload);
        assert_eq!(seq, 8);
        let pkt = decode_frame(&frame).unwrap();
        assert_eq!(pkt.packet_id, PID_PING);
        assert_eq!(pkt.payload, payload);
    }

    #[test]
    fn decode_rejects_oversized_length() {
        let mut hdr = [0u8; HDR_SIZE];
        hdr[0..2].copy_from_slice(&PID_PING.to_be_bytes());
        hdr[8..12].copy_from_slice(&(MAX_PAYLOAD + 1).to_be_bytes());
        assert!(decode_frame(&hdr).is_err());
    }

    #[test]
    fn decode_rejects_truncated_body() {
        let mut seq = 0u32;
        let frame = encode_frame(PID_PING, &mut seq, b"x");
        let truncated = &frame[..frame.len() - 1];
        assert!(decode_frame(truncated).is_err());
    }
}
