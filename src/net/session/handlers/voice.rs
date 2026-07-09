use std::sync::Arc;

use anyhow::Result;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::net::{crypto::SessionCrypto, types::NetEvent};

pub struct State {
    pub udp: Arc<UdpSocket>,
    pub seq: u64,
}

impl State {
    pub async fn new(crypto: &SessionCrypto, event_tx: &mpsc::Sender<NetEvent>) -> Result<Self> {
        let udp = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        crate::net::voice::spawn_recv(udp.clone(), (*crypto).clone(), event_tx.clone());
        Ok(Self { udp, seq: 0 })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_voice(
        &mut self,
        channel_id: u64,
        voice_pkt_seq: u32,
        timestamp: u32,
        opus_data: &[u8],
        crypto: &SessionCrypto,
        endpoint: Option<std::net::SocketAddr>,
        sender_pubkey: &[u8; 32],
    ) -> Result<()> {
        if let Some(ep) = endpoint {
            let pkt = crate::net::voice::build_packet(
                channel_id,
                voice_pkt_seq,
                timestamp,
                opus_data,
                crypto,
                self.seq,
                sender_pubkey,
            );
            let _ = self.udp.send_to(&pkt, ep).await;
            self.seq += 1;
        }
        Ok(())
    }
}
