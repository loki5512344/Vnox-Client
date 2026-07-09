use std::time::Duration;

use anyhow::Result;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{MissedTickBehavior, interval};
use tracing::warn;

use crate::identity::Identity;
use crate::net::{
    framing, handshake, internal,
    payloads::*,
    types::{NetCommand, NetEvent},
};
use serde_json;

use super::super::switch::ReconnectRequested;
use super::{VoiceState, handle_cmd};

pub async fn session_loop(
    identity: &Identity,
    address: &str,
    channels: &mut Vec<String>,
    restored: bool,
    cmd_rx: &mut mpsc::Receiver<NetCommand>,
    event_tx: &mpsc::Sender<NetEvent>,
) -> Result<()> {
    let mut stream = TcpStream::connect(address).await?;
    let mut seq = 0u32;

    let (session_id, node_name, private_mode, crypto) =
        handshake::run(&mut stream, identity, &mut seq).await?;
    let _ = event_tx
        .send(NetEvent::Connected {
            session_id: session_id.clone(),
            node_name,
            restored,
            private_mode,
        })
        .await;

    for channel_id in channels.clone() {
        framing::io::write_encrypted(
            &mut stream,
            PID_JOIN_CHANNEL,
            &mut seq,
            &serde_json::to_vec(&JoinChannelPayload { channel_id })?,
            &crypto,
        )
        .await?;
    }

    let mut voice_state = VoiceState::new(&crypto, event_tx).await?;
    let mut voice_endpoint: Option<std::net::SocketAddr> = None;
    let mut ping_tick = interval(Duration::from_secs(2));
    ping_tick.set_missed_tick_behavior(MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = ping_tick.tick() => {
                let ts = now_ms();
                let payload = serde_json::to_vec(&PingPayload { timestamp: ts })?;
                framing::io::write_encrypted(&mut stream, PID_PING, &mut seq, &payload, &crypto).await?;
            }
            result = framing::io::read_encrypted(&mut stream, &crypto) => {
                let pkt = result?;
                if pkt.packet_id == PID_CHANNEL_STATE
                    && let Ok(p) = serde_json::from_slice::<ChannelStatePayload>(&pkt.payload)
                {
                    voice_endpoint = if p.voice_endpoint.is_empty() {
                        None
                    } else {
                        p.voice_endpoint.parse().ok()
                    };
                }
                internal::dispatch::incoming(pkt.packet_id, &pkt.payload, event_tx).await?;
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    None | Some(NetCommand::Disconnect) => {
                        let _ = framing::io::write_encrypted(&mut stream, PID_DISCONNECT, &mut seq,
                            &serde_json::to_vec(&serde_json::json!({"reason":"bye"})).unwrap(), &crypto).await;
                        return Ok(());
                    }
                    Some(NetCommand::Connect { address, auto_reconnect }) => {
                        warn!("connect while connected to {address}, reconnecting");
                        if channels.is_empty() {
                            *channels = vec!["general".into()];
                        }
                        return Err(anyhow::Error::new(ReconnectRequested {
                            address,
                            auto_reconnect,
                        }));
                    }
                    Some(NetCommand::JoinChannel { channel_id }) => {
                        track_channel(channels, &channel_id);
                        framing::io::write_encrypted(&mut stream, PID_JOIN_CHANNEL, &mut seq,
                            &serde_json::to_vec(&JoinChannelPayload { channel_id })?, &crypto).await?;
                    }
                    Some(NetCommand::LeaveChannel { channel_id }) => {
                        channels.retain(|c| c != &channel_id);
                        framing::io::write_encrypted(&mut stream, PID_LEAVE_CHANNEL, &mut seq,
                            &serde_json::to_vec(&LeaveChannelPayload { channel_id })?, &crypto).await?;
                    }
                    Some(NetCommand::ChannelCreate { channel_id, channel_name, kind }) => {
                        let payload = serde_json::to_vec(&ChannelCreatePayload {
                            channel_id,
                            channel_name,
                            kind,
                            guild_id: None,
                        })?;
                        framing::io::write_encrypted(&mut stream, PID_CHANNEL_CREATE, &mut seq, &payload, &crypto).await?;
                    }
                    Some(NetCommand::ChannelDelete { channel_id }) => {
                        let payload = serde_json::to_vec(&ChannelDeletePayload { channel_id })?;
                        framing::io::write_encrypted(&mut stream, PID_CHANNEL_DELETE, &mut seq, &payload, &crypto).await?;
                    }
                    Some(NetCommand::ChannelList) => {
                        framing::io::write_encrypted(&mut stream, PID_CHANNEL_LIST, &mut seq, b"{}", &crypto).await?;
                    }
                    Some(NetCommand::SendChat { channel_id, content, reply_to }) => {
                        let msg = ChatMessagePayload {
                            message_id: uuid_v4(), channel_id,
                            sender_id: identity.pubkey_hex.clone(),
                            content, timestamp: now_ms(),
                            edited: false,
                            reply_to,
                        };
                        framing::io::write_encrypted(&mut stream, PID_CHAT_MESSAGE, &mut seq, &serde_json::to_vec(&msg)?, &crypto).await?;
                    }
                    Some(NetCommand::SendVoice { channel_id, voice_seq: voice_pkt_seq, timestamp, opus_data }) => {
                        let sender_pubkey = identity.verifying_key()
                            .map(|k| k.to_bytes())
                            .unwrap_or([0u8; 32]);
                        voice_state.send_voice(channel_id, voice_pkt_seq, timestamp, &opus_data, &crypto, voice_endpoint, &sender_pubkey).await?;
                    }
                    Some(NetCommand::ReadReceipt { channel_id, last_read_message_id }) => {
                        let payload = serde_json::to_vec(&serde_json::json!({
                            "channel_id": channel_id,
                            "last_read_message_id": last_read_message_id,
                            "user_id": identity.pubkey_hex.clone(),
                        }))?;
                        framing::io::write_encrypted(&mut stream, PID_READ_RECEIPT, &mut seq, &payload, &crypto).await?;
                    }
                    Some(other) => handle_cmd(other, &mut stream, &mut seq, &crypto, identity).await?,
                }
            }
        }
    }
}

pub(super) fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

pub(super) fn uuid_v4() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn track_channel(channels: &mut Vec<String>, channel_id: &str) {
    if !channels.iter().any(|c| c == channel_id) {
        channels.push(channel_id.to_string());
    }
}
