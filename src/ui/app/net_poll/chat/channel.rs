use crate::state::{Channel, DmConversation, UiState};
use vnox_client::net::{NetCommand, NetEvent, NetHandle};

use super::into_msg;

pub(super) fn handle(ui: &mut UiState, net: &NetHandle, event: NetEvent) {
    match event {
        NetEvent::ChannelState {
            channel_id,
            channel_name,
            kind,
            members,
            ..
        } => {
            ui.server_channel = Some(channel_id.clone());
            let mems: Vec<String> = members
                .iter()
                .map(|m| {
                    ui.user_names.insert(m.user_id.clone(), m.nickname.clone());
                    m.nickname.clone()
                })
                .collect();
            if let Some(ch) = ui.channels.iter_mut().find(|c| c.id == channel_id) {
                ch.members = mems;
            } else {
                ui.channels.push(Channel {
                    id: channel_id.clone(),
                    name: channel_name,
                    kind: kind.clone(),
                    members: mems,
                });
            }
            if ui.active_channel.is_none() {
                ui.active_channel = Some(channel_id.clone());
            }
            let net = net.clone();
            let ch_id = channel_id;
            tokio::spawn(async move {
                net.send(NetCommand::ReadReceipt {
                    channel_id: ch_id,
                    last_read_message_id: uuid::Uuid::new_v4().to_string(),
                })
                .await;
            });
        }
        NetEvent::UserJoin {
            channel_id,
            user_id,
            nickname,
            ..
        } => {
            ui.user_names.insert(user_id, nickname.clone());
            if let Some(ch) = ui.channels.iter_mut().find(|c| c.id == channel_id)
                && !ch.members.contains(&nickname)
            {
                ch.members.push(nickname);
            }
        }
        NetEvent::UserLeave {
            channel_id,
            user_id,
        } => {
            if let Some(ch) = ui.channels.iter_mut().find(|c| c.id == channel_id) {
                let nickname = ui.user_names.remove(&user_id);
                ch.members.retain(|m| {
                    m != &user_id && nickname.as_deref().map(|n| m != n).unwrap_or(true)
                });
            }
        }
        NetEvent::ChannelCreated {
            channel_id,
            channel_name,
            kind,
        } => {
            // Another session created a channel — add to local list if not present.
            if !ui.channels.iter().any(|c| c.id == channel_id) {
                ui.channels.push(Channel {
                    id: channel_id,
                    name: channel_name,
                    kind,
                    members: Vec::new(),
                });
            }
        }
        NetEvent::ChannelDeleted { channel_id } => {
            ui.channels.retain(|c| c.id != channel_id);
            // Clear active_channel if it pointed at the deleted channel.
            if ui.active_channel.as_deref() == Some(&channel_id) {
                ui.active_channel = ui.channels.first().map(|c| c.id.clone());
            }
            // Drop any cached messages for the deleted channel.
            ui.messages.remove(&channel_id);
        }
        NetEvent::ChannelListEvent { channels } => {
            // Replace local channel list with the server-authoritative one.
            // Preserve `members` for channels we were already tracking.
            let old: std::collections::HashMap<String, Channel> = std::mem::take(&mut ui.channels)
                .into_iter()
                .map(|c| (c.id.clone(), c))
                .collect();
            ui.channels = channels
                .into_iter()
                .map(|c| {
                    let members = old
                        .get(&c.channel_id)
                        .map(|o| o.members.clone())
                        .unwrap_or_default();
                    Channel {
                        id: c.channel_id,
                        name: c.channel_name,
                        kind: c.kind,
                        members,
                    }
                })
                .collect();
        }
        NetEvent::DmStart {
            dm_id,
            other_user_id,
            other_nickname,
            messages,
            unread_count,
        } => {
            if !ui.dm_conversations.iter().any(|c| c.dm_id == dm_id) {
                ui.dm_conversations.push(DmConversation {
                    dm_id: dm_id.clone(),
                    other_user_id,
                    other_nickname,
                    unread_count,
                });
            }
            ui.dm_messages
                .insert(dm_id.clone(), messages.into_iter().map(into_msg).collect());
            ui.active_dm_id = Some(dm_id);
        }
        _ => {}
    }
}
