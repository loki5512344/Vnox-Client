mod chat;
mod social;
mod status;

use vnox_client::{
    audio::IncomingVoice,
    net::{NetCommand, NetEvent, NetHandle},
};

use crate::state::ConnState;

use super::VnoxApp;

impl VnoxApp {
    pub(crate) fn poll_net(&mut self) {
        while let Some(ev) = self.net.try_recv() {
            match ev {
                NetEvent::Connected {
                    session_id,
                    node_name,
                    restored,
                    private_mode,
                } => self.on_connected(session_id, node_name, restored, private_mode),
                NetEvent::Disconnected { reason, will_retry } => {
                    self.stop_voice();
                    status::handle_disconnected(&mut self.ui, &reason, will_retry);
                }
                NetEvent::Reconnecting {
                    attempt,
                    delay_secs,
                } => {
                    self.stop_voice();
                    status::handle_reconnecting(&mut self.ui, attempt, delay_secs);
                }
                NetEvent::VoicePacket {
                    channel_id,
                    voice_seq,
                    timestamp,
                    opus_data,
                    sender_id,
                    ..
                } => {
                    // Track which remote user is speaking (Phase 1.3 attribution).
                    if !sender_id.is_empty() {
                        self.ui.last_remote_speaker_id = sender_id.clone();
                        self.ui.last_voice_packet_ms = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as u64)
                            .unwrap_or(0);
                    }
                    if !self.ui.deafen
                        && let Some(pipeline) = self.audio.as_ref()
                    {
                        pipeline.push_voice(IncomingVoice {
                            voice_seq,
                            timestamp,
                            channel_id,
                            opus_data,
                        });
                    }
                }
                event => process_net_event(&mut self.ui, &self.net, event),
            }
            self.sync_voice_audio();
            self.flush_encoded_voice();
        }
        // Refresh speaking indicators from the audio pipeline.
        if let Some(pipeline) = self.audio.as_ref() {
            self.ui.local_speaking = pipeline.local_speaking();
            self.ui.remote_speaking = pipeline.remote_speaking(500);
        } else {
            self.ui.local_speaking = false;
            self.ui.remote_speaking = false;
        }
        // Decay remote speaker id if no packet arrived in 700 ms.
        if !self.ui.remote_speaking {
            self.ui.last_remote_speaker_id.clear();
        }
    }

    pub(crate) fn on_connected(
        &mut self,
        session_id: String,
        node_name: String,
        restored: bool,
        private_mode: bool,
    ) {
        self.ui.conn = ConnState::Connected {
            session_id,
            node_name,
        };
        self.ui.private_mode = private_mode;
        if restored {
            self.ui.status_msg = Some("reconnected".into());
            if let Some(ch) = self.ui.active_channel.clone() {
                let net = self.net.clone();
                tokio::spawn(async move {
                    net.send(NetCommand::JoinChannel { channel_id: ch }).await;
                });
            }
        } else {
            self.ui.seed_default_channels();
            self.ui.active_channel = Some("general".into());
            self.ui.status_msg = None;
            let net = self.net.clone();
            tokio::spawn(async move {
                net.send(NetCommand::JoinChannel {
                    channel_id: "general".into(),
                })
                .await;
            });
        }
    }
}

pub(crate) fn process_net_event(ui: &mut crate::state::UiState, net: &NetHandle, event: NetEvent) {
    match &event {
        NetEvent::ChannelState { .. }
        | NetEvent::ChannelCreated { .. }
        | NetEvent::ChannelDeleted { .. }
        | NetEvent::ChannelListEvent { .. }
        | NetEvent::ChatMessage { .. }
        | NetEvent::ChatHistory { .. }
        | NetEvent::UserJoin { .. }
        | NetEvent::UserLeave { .. }
        | NetEvent::Error { .. }
        | NetEvent::ReactionAdded { .. }
        | NetEvent::ReactionRemoved { .. }
        | NetEvent::MessageEdited { .. }
        | NetEvent::MessageDeleted { .. }
        | NetEvent::DmStart { .. }
        | NetEvent::DmMessage { .. }
        | NetEvent::DmHistory { .. }
        | NetEvent::ReadReceiptBroadcast { .. }
        | NetEvent::UserColor { .. } => chat::handle(ui, net, event),

        NetEvent::GuildList { .. }
        | NetEvent::GuildCreated { .. }
        | NetEvent::GuildDeleted { .. }
        | NetEvent::GuildMemberJoined { .. }
        | NetEvent::GuildMemberLeft { .. }
        | NetEvent::GuildMemberKicked { .. }
        | NetEvent::InviteCreated { .. }
        | NetEvent::InviteAccepted { .. }
        | NetEvent::InviteDeleted { .. }
        | NetEvent::RoleCreated { .. }
        | NetEvent::RoleDeleted { .. }
        | NetEvent::GuildAuditLog { .. }
        | NetEvent::GuildMemberList { .. }
        | NetEvent::GuildRoleList { .. }
        | NetEvent::FriendList { .. }
        | NetEvent::FriendRequested { .. }
        | NetEvent::FriendAccepted { .. }
        | NetEvent::FriendRemoved { .. }
        | NetEvent::BlockList { .. }
        | NetEvent::BlockedUser { .. }
        | NetEvent::UnblockedUser { .. } => social::handle(ui, event),

        NetEvent::LatencyUpdate { .. }
        | NetEvent::TypingStart { .. }
        | NetEvent::PresenceSync { .. }
        | NetEvent::PresenceUpdated { .. } => status::handle(ui, net, event),

        NetEvent::Connected { .. }
        | NetEvent::Disconnected { .. }
        | NetEvent::Reconnecting { .. }
        | NetEvent::VoicePacket { .. } => {}
    }
}
