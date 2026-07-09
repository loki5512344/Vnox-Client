use crate::state::{ChatMessage, UiState};
use std::collections::HashMap;
use vnox_client::net::{NetEvent, NetHandle};

use super::into_msg;

pub(super) fn handle(ui: &mut UiState, _net: &NetHandle, event: NetEvent) {
    match event {
        NetEvent::ChatMessage {
            message_id,
            channel_id,
            sender_id,
            content,
            timestamp,
            reply_to,
            ..
        } => {
            ui.messages
                .entry(channel_id)
                .or_default()
                .push(ChatMessage {
                    message_id,
                    sender_id,
                    content,
                    timestamp,
                    edited: false,
                    reactions: HashMap::new(),
                    reply_to,
                });
            ui.scroll_bottom = true;
        }
        NetEvent::ChatHistory {
            channel_id,
            messages,
        } => {
            ui.messages
                .insert(channel_id, messages.into_iter().map(into_msg).collect());
            ui.scroll_bottom = true;
        }
        NetEvent::Error { message, .. } => {
            ui.status_msg = Some(format!("error: {message}"));
        }
        NetEvent::ReactionAdded {
            channel_id,
            message_id,
            user_id,
            emoji,
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id)
                && let Some(msg) = msgs.iter_mut().find(|m| m.message_id == message_id)
            {
                msg.reactions.entry(emoji).or_default().push(user_id);
            }
        }
        NetEvent::ReactionRemoved {
            channel_id,
            message_id,
            user_id,
            emoji,
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id)
                && let Some(msg) = msgs.iter_mut().find(|m| m.message_id == message_id)
                && let Some(users) = msg.reactions.get_mut(&emoji)
            {
                users.retain(|u| u != &user_id);
                if users.is_empty() {
                    msg.reactions.remove(&emoji);
                }
            }
        }
        NetEvent::MessageEdited {
            channel_id,
            message_id,
            content,
            ..
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id)
                && let Some(msg) = msgs.iter_mut().find(|m| m.message_id == message_id)
            {
                msg.content = content;
                msg.edited = true;
            }
        }
        NetEvent::MessageDeleted {
            channel_id,
            message_id,
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id) {
                msgs.retain(|m| m.message_id != message_id);
            }
        }
        NetEvent::DmMessage {
            dm_id,
            sender_id,
            content,
            timestamp,
            ..
        } => {
            ui.dm_messages
                .entry(dm_id.clone())
                .or_default()
                .push(ChatMessage {
                    message_id: uuid::Uuid::new_v4().to_string(),
                    sender_id,
                    content,
                    timestamp,
                    edited: false,
                    reactions: HashMap::new(),
                    reply_to: None,
                });
            if ui.active_dm_id.as_deref() != Some(&dm_id)
                && let Some(c) = ui.dm_conversations.iter_mut().find(|c| c.dm_id == dm_id)
            {
                c.unread_count += 1;
            }
        }
        NetEvent::DmHistory { dm_id, messages } => {
            ui.dm_messages
                .insert(dm_id, messages.into_iter().map(into_msg).collect());
        }
        NetEvent::ReadReceiptBroadcast {
            channel_id,
            user_id,
            last_read_message_id,
        } => {
            ui.read_receipts
                .entry(channel_id)
                .or_default()
                .insert(user_id, last_read_message_id);
        }
        NetEvent::UserColor { user_id, color } => {
            if let Some(c) = parse_hex_color(&color) {
                ui.user_colors.insert(user_id, c);
            }
        }
        _ => {}
    }
}

fn parse_hex_color(hex: &str) -> Option<eframe::egui::Color32> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(eframe::egui::Color32::from_rgb(r, g, b))
}
