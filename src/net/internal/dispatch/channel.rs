use anyhow::{Result, anyhow};
use serde_json;
use tokio::sync::mpsc;

use super::super::super::payloads::*;
use super::super::super::types::NetEvent;
use super::super::super::types::{ChannelListItem as InfoChannelListItem, ChatMsg, MemberInfo};

pub async fn handle(pid: u16, payload: &[u8], tx: &mpsc::Sender<NetEvent>) -> Result<()> {
    match pid {
        PID_CHANNEL_STATE => {
            let p: ChannelStatePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ChannelState {
                    channel_id: p.channel_id,
                    channel_name: p.channel_name,
                    kind: p.kind,
                    members: p
                        .members
                        .into_iter()
                        .map(|m| MemberInfo {
                            user_id: m.user_id,
                            nickname: m.nickname,
                            in_voice: m.in_voice,
                        })
                        .collect(),
                    voice_endpoint: p.voice_endpoint,
                    guild_id: p.guild_id,
                })
                .await;
        }
        PID_CHANNEL_CREATE => {
            let p: ChannelCreatePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ChannelCreated {
                    channel_id: p.channel_id,
                    channel_name: p.channel_name,
                    kind: p.kind,
                    guild_id: p.guild_id,
                })
                .await;
        }
        PID_CHANNEL_DELETE => {
            let p: ChannelDeletePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ChannelDeleted {
                    channel_id: p.channel_id,
                })
                .await;
        }
        PID_CHANNEL_LIST => {
            let p: ChannelListPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ChannelListEvent {
                    channels: p
                        .channels
                        .into_iter()
                        .map(|c| InfoChannelListItem {
                            channel_id: c.channel_id,
                            channel_name: c.channel_name,
                            kind: c.kind,
                            guild_id: c.guild_id,
                        })
                        .collect(),
                })
                .await;
        }
        PID_CHAT_MESSAGE => {
            let p: ChatMessagePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ChatMessage {
                    message_id: p.message_id,
                    channel_id: p.channel_id,
                    sender_id: p.sender_id.clone(),
                    nickname: p.sender_id,
                    content: p.content,
                    timestamp: p.timestamp,
                    reply_to: p.reply_to,
                })
                .await;
        }
        PID_CHAT_HISTORY => {
            let p: ChatHistoryPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ChatHistory {
                    channel_id: p.channel_id,
                    messages: p
                        .messages
                        .into_iter()
                        .map(|m| ChatMsg {
                            message_id: m.message_id,
                            sender_id: m.sender_id,
                            content: m.content,
                            timestamp: m.timestamp,
                            reply_to: m.reply_to,
                        })
                        .collect(),
                })
                .await;
        }
        PID_USER_JOIN => {
            let p: UserJoinPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::UserJoin {
                    channel_id: p.channel_id,
                    user_id: p.user_id,
                    nickname: p.nickname,
                })
                .await;
        }
        PID_USER_LEAVE => {
            let p: UserLeavePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::UserLeave {
                    channel_id: p.channel_id,
                    user_id: p.user_id,
                })
                .await;
        }
        PID_PONG => {
            let p: PongPayload = serde_json::from_slice(payload)?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64;
            let rtt_ms = (now - p.timestamp).max(0) as u32;
            let _ = tx.send(NetEvent::LatencyUpdate { rtt_ms }).await;
        }
        PID_READ_RECEIPT_BROADCAST => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ReadReceiptBroadcast {
                    channel_id: p["channel_id"].as_str().unwrap_or_default().into(),
                    user_id: p["user_id"].as_str().unwrap_or_default().into(),
                    last_read_message_id: p["last_read_message_id"]
                        .as_str()
                        .unwrap_or_default()
                        .into(),
                })
                .await;
        }
        PID_ERROR => {
            let p: ErrorPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::Error {
                    code: p.code,
                    message: p.message,
                })
                .await;
        }
        PID_DISCONNECT => return Err(anyhow!("server disconnected")),
        PID_TYPING_START => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::TypingStart {
                    user_id: p["user_id"].as_str().unwrap_or_default().into(),
                    nickname: p["nickname"].as_str().unwrap_or_default().into(),
                    channel_id: p["channel_id"].as_str().unwrap_or_default().into(),
                })
                .await;
        }
        _ => {}
    }
    Ok(())
}
