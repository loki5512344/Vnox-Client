use vnox_client::net::{NetEvent, NetHandle, PresenceInfo};

use crate::state::ConnState;

use crate::state::UiState;

pub(crate) fn handle_disconnected(ui: &mut UiState, reason: &str, will_retry: bool) {
    if will_retry {
        ui.status_msg = Some(format!("connection lost: {reason}"));
    } else {
        ui.conn = ConnState::Disconnected;
        ui.channels.clear();
        ui.active_channel = None;
        ui.user_names.clear();
        ui.server_channel = None;
        ui.rtt_ms = None;
        ui.status_msg = Some(format!("disconnected: {reason}"));
    }
    ui.dm_conversations.clear();
    ui.dm_messages.clear();
    ui.active_dm_id = None;
    ui.new_dm_input = None;
    ui.private_mode = false;
}

pub(crate) fn handle_reconnecting(ui: &mut UiState, attempt: u32, delay_secs: u64) {
    ui.conn = ConnState::Reconnecting { attempt };
    ui.status_msg = Some(format!("reconnecting in {delay_secs}s (attempt {attempt})"));
}

pub(crate) fn handle(ui: &mut UiState, _net: &NetHandle, event: NetEvent) {
    match event {
        NetEvent::LatencyUpdate { rtt_ms } => {
            ui.rtt_ms = Some(rtt_ms);
        }
        NetEvent::TypingStart {
            user_id,
            nickname,
            channel_id,
        } => {
            let entry = ui.typing_users.entry(channel_id).or_default();
            entry.retain(|(u, _)| u != &user_id);
            entry.push((nickname, std::time::Instant::now()));
        }
        NetEvent::PresenceSync { presences } => {
            for p in presences {
                ui.presences.insert(p.user_id.clone(), p);
            }
        }
        NetEvent::PresenceUpdated {
            user_id,
            status,
            custom_status,
            activity,
        } => {
            ui.presences.insert(
                user_id.clone(),
                PresenceInfo {
                    user_id,
                    status,
                    custom_status,
                    activity,
                    activity_text: None,
                },
            );
        }
        _ => {}
    }
}
