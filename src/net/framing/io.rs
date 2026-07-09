use anyhow::{Result, anyhow};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use super::super::crypto::SessionCrypto;
use super::super::payloads::HDR_SIZE;
use super::RawPacket;
use super::parse_header;

const ENCRYPTED_FLAG: u16 = 1 << 1;
const KNOWN_FLAGS: u16 = ENCRYPTED_FLAG;

pub async fn read_encrypted(stream: &mut TcpStream, crypto: &SessionCrypto) -> Result<RawPacket> {
    let mut buf = [0u8; HDR_SIZE];
    stream.read_exact(&mut buf).await?;
    let (packet_id, flags, sequence, payload_length) = parse_header(&buf)?;
    if flags & !KNOWN_FLAGS != 0 {
        return Err(anyhow!("unknown packet flags: 0x{:04X}", flags));
    }
    if flags & ENCRYPTED_FLAG == 0 {
        return Err(anyhow!("packet not encrypted"));
    }
    let mut encrypted = vec![0u8; payload_length as usize];
    if !encrypted.is_empty() {
        stream.read_exact(&mut encrypted).await?;
    }
    let payload = crypto.decrypt_s2c(sequence as u64, &encrypted)?;
    Ok(RawPacket { packet_id, payload })
}

pub async fn write_encrypted(
    stream: &mut TcpStream,
    id: u16,
    seq: &mut u32,
    payload: &[u8],
    crypto: &SessionCrypto,
) -> Result<()> {
    let encrypted = crypto.encrypt_c2s(*seq as u64, payload);
    let mut hdr = [0u8; HDR_SIZE];
    hdr[0..2].copy_from_slice(&id.to_be_bytes());
    hdr[2..4].copy_from_slice(&ENCRYPTED_FLAG.to_be_bytes());
    hdr[4..8].copy_from_slice(&seq.to_be_bytes());
    hdr[8..12].copy_from_slice(&(encrypted.len() as u32).to_be_bytes());
    *seq = seq.wrapping_add(1);
    let mut frame = Vec::with_capacity(HDR_SIZE + encrypted.len());
    frame.extend_from_slice(&hdr);
    frame.extend_from_slice(&encrypted);
    stream.write_all(&frame).await?;
    Ok(())
}
