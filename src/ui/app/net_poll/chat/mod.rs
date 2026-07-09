mod channel;
mod messages;

use crate::state::{ChatMessage, UiState};
use std::collections::HashMap;
use vnox_client::net::{ChatMsg, NetEvent, NetHandle};

fn into_msg(m: ChatMsg) -> ChatMessage {
    ChatMessage {
        message_id: m.message_id,
        sender_id: m.sender_id,
        content: m.content,
        timestamp: m.timestamp,
        edited: false,
        reactions: HashMap::new(),
        reply_to: m.reply_to,
    }
}

pub(crate) fn handle(ui: &mut UiState, net: &NetHandle, event: NetEvent) {
    match event {
        NetEvent::ChannelState { .. }
        | NetEvent::UserJoin { .. }
        | NetEvent::UserLeave { .. }
        | NetEvent::DmStart { .. } => channel::handle(ui, net, event),
        e => messages::handle(ui, net, e),
    }
}
