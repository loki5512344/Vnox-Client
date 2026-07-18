use std::collections::HashMap;

use super::models::default_bookmarks;
use super::types::{ConnState, UiState, VoiceUiState};

impl Default for UiState {
    fn default() -> Self {
        Self {
            conn: ConnState::Disconnected,
            connect_input: "127.0.0.1:7600".into(),
            bookmarks: default_bookmarks(),
            selected_bookmark: 0,
            status_msg: None,
            channels: Vec::new(),
            active_channel: None,
            messages: Default::default(),
            chat_input: String::new(),
            scroll_bottom: false,
            voice: VoiceUiState::Off,
            voice_joined_at: None,
            user_names: Default::default(),
            rtt_ms: None,

            settings_open: false,

            mic_enabled: true,
            deafen: false,
            vad_mode: 0,
            voice_bitrate: 64.0,
            packet_interval: 1,
            mic_device: 0,

            output_device: 0,
            output_volume: 90.0,

            auto_reconnect: true,

            guilds: Vec::new(),
            active_guild_id: None,
            presences: Default::default(),
            friends: Vec::new(),
            pending_friend_requests: Vec::new(),
            blocked_users: Vec::new(),
            dm_conversations: Vec::new(),
            dm_messages: Default::default(),
            active_dm_id: None,
            new_dm_input: None,

            user_colors: HashMap::new(),

            private_mode: false,

            typing_users: HashMap::new(),
            read_receipts: HashMap::new(),
            friends_tab: "all".into(),
            local_speaking: false,
            remote_speaking: false,
            last_remote_speaker_id: String::new(),
            last_voice_packet_ms: 0,

            replying_to_message: None,
            replying_to_sender: String::new(),
            replying_to_content: String::new(),
            editing_message_id: None,

            settings_passphrase_open: false,
            settings_passphrase_input: String::new(),
            settings_passphrase_confirm: String::new(),
            settings_passphrase_error: None,

            settings_export_open: false,
            settings_export_pass: String::new(),
            settings_export_confirm: String::new(),
            settings_export_output: None,
            settings_export_error: None,

            settings_import_open: false,
            settings_import_input: String::new(),
            settings_import_pass: String::new(),
            settings_import_error: None,
        }
    }
}
