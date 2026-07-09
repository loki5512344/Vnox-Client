use anyhow::Result;
use serde_json;
use tokio::sync::mpsc;
use tracing::debug;

use super::super::super::payloads::*;
use super::super::super::types::*;

pub async fn handle(pid: u16, payload: &[u8], tx: &mpsc::Sender<NetEvent>) -> Result<()> {
    match pid {
        PID_DM_START => {
            let p: DmStartResponsePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::DmStart {
                    dm_id: p.dm_id,
                    other_user_id: p.other_user_id,
                    other_nickname: p.other_nickname,
                    messages: p
                        .messages
                        .into_iter()
                        .map(|m| ChatMsg {
                            message_id: String::new(),
                            sender_id: m.sender_id,
                            content: m.content,
                            timestamp: m.timestamp,
                            reply_to: None,
                        })
                        .collect(),
                    unread_count: p.unread_count,
                })
                .await;
        }
        PID_DM_MESSAGE => {
            let p: DmMessagePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::DmMessage {
                    dm_id: p.dm_id,
                    sender_id: p.sender_id,
                    content: p.content,
                    timestamp: p.timestamp,
                })
                .await;
        }
        PID_DM_HISTORY => {
            let p: DmHistoryPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::DmHistory {
                    dm_id: p.dm_id,
                    messages: p
                        .messages
                        .into_iter()
                        .map(|m| ChatMsg {
                            message_id: String::new(),
                            sender_id: m.sender_id,
                            content: m.content,
                            timestamp: m.timestamp,
                            reply_to: None,
                        })
                        .collect(),
                })
                .await;
        }
        PID_FRIEND_LIST => {
            let p: FriendListPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::FriendList {
                    friends: p
                        .friends
                        .into_iter()
                        .map(|f| FriendInfo {
                            user_id: f.user_id,
                            nickname: f.nickname,
                            status: f.status,
                            since: f.since,
                        })
                        .collect(),
                })
                .await;
        }
        PID_MESSAGE_REACTION_ADD => {
            let p: ReactionPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ReactionAdded {
                    channel_id: p.channel_id,
                    message_id: p.message_id,
                    user_id: p.user_id,
                    emoji: p.emoji,
                })
                .await;
        }
        PID_MESSAGE_REACTION_REMOVE => {
            let p: ReactionPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::ReactionRemoved {
                    channel_id: p.channel_id,
                    message_id: p.message_id,
                    user_id: p.user_id,
                    emoji: p.emoji,
                })
                .await;
        }
        PID_MESSAGE_EDIT => {
            let p: ChatMessagePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::MessageEdited {
                    channel_id: p.channel_id,
                    message_id: p.message_id,
                    content: p.content,
                    edited: p.edited,
                })
                .await;
        }
        PID_MESSAGE_DELETE => {
            let p: MessageDeletePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::MessageDeleted {
                    channel_id: p.channel_id,
                    message_id: p.message_id,
                })
                .await;
        }
        PID_FRIEND_EVENT => {
            let p: FriendEventPayload = serde_json::from_slice(payload)?;
            match p.event_type.as_str() {
                "request" => {
                    let _ = tx
                        .send(NetEvent::FriendRequested {
                            user_id: p.user_id,
                            nickname: p.nickname.unwrap_or_default(),
                        })
                        .await;
                }
                "accepted" => {
                    let _ = tx
                        .send(NetEvent::FriendAccepted {
                            user_id: p.user_id,
                            nickname: p.nickname.unwrap_or_default(),
                        })
                        .await;
                }
                "removed" => {
                    let _ = tx
                        .send(NetEvent::FriendRemoved { user_id: p.user_id })
                        .await;
                }
                _ => debug!("unhandled friend event: {}", p.event_type),
            }
        }
        _ => {}
    }
    Ok(())
}
