use std::collections::{HashMap, HashSet};
use std::time::Instant;

use super::models::{default_bookmarks, now_utc_hms};
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
            editing_message_id: None,
            replying_to: None,
            context_menu_message_id: None,
            scroll_bottom: false,
            voice: VoiceUiState::Off,
            voice_joined_at: None,
            user_names: Default::default(),
            server_channel: None,
            rtt_ms: None,

            settings_open: false,
            settings_page: Default::default(),

            mic_enabled: true,
            deafen: false,
            noise_suppress: true,
            echo_cancel: true,
            vad_mode: 0,
            vad_threshold: 40.0,
            voice_bitrate: 64.0,
            packet_interval: 1,
            mic_device: 0,
            input_volume: 80.0,
            input_device_labels: vec!["Default".into()],
            input_device_metas: Vec::new(),
            output_device_labels: vec!["Default".into()],
            output_device_metas: Vec::new(),
            last_device_scan: None,

            output_device: 0,
            output_volume: 90.0,
            jitter_size: 1,
            adaptive_buffer: true,

            overlay_enabled: false,
            overlay_speakers: true,
            overlay_latency: true,

            relay_enabled: true,
            relay_address: "relay.nightcore.lnex".into(),
            auto_relay: true,
            force_relay: false,
            udp_port: "7700".into(),
            auto_reconnect: true,

            ui_scale: 100.0,

            kb_ptt: "mouse4".into(),
            kb_mute: "ctrl+m".into(),
            kb_deafen: "ctrl+d".into(),
            kb_overlay: "ctrl+shift+o".into(),

            log_level: 0,
            log_voice_packets: false,
            show_packet_stats: false,
            disable_encryption: false,

            guilds: Vec::new(),
            active_guild_id: None,
            presences: Default::default(),
            presence_status: Some("online".into()),
            custom_status: None,
            activity_type: None,
            activity_text: None,
            friends: Vec::new(),
            pending_friend_requests: Vec::new(),
            blocked_users: Vec::new(),
            block_input: None,
            block_list_fetched_at: None,
            dm_conversations: Vec::new(),
            dm_messages: Default::default(),
            active_dm_id: None,
            new_dm_input: None,

            add_server_open: false,
            add_server_input: String::new(),
            invite_accept_open: false,
            invite_accept_code: String::new(),
            create_guild_open: false,
            create_guild_input: String::new(),

            create_channel_open: false,
            create_channel_input: String::new(),
            create_channel_kind: "text".into(),

            vault_set_open: false,
            vault_remove_open: false,
            vault_passphrase_input: String::new(),
            vault_passphrase_confirm: String::new(),
            vault_error: None,

            export_keyfile_open: false,
            export_keyfile_pass: String::new(),
            export_keyfile_confirm: String::new(),
            export_keyfile_output: String::new(),
            export_keyfile_error: None,
            import_keyfile_open: false,
            import_keyfile_input: String::new(),
            import_keyfile_pass: String::new(),
            import_keyfile_error: None,

            audit_log_open: false,
            audit_log_guild_id: None,
            audit_log_entries: Vec::new(),

            guild_members_open: false,
            guild_member_list_guild_id: None,
            guild_member_list: Vec::new(),
            guild_role_list_guild_id: None,
            guild_role_list: Vec::new(),

            add_friend_open: false,
            add_friend_input: String::new(),

            dm_search_query: String::new(),
            user_colors: HashMap::new(),
            per_user_volumes: Default::default(),
            kb_listening: None,

            private_mode: false,

            collapsed_categories: HashSet::new(),
            startup_time_str: now_utc_hms(),
            typing_users: HashMap::new(),
            last_typing_send: Instant::now(),
            read_receipts: HashMap::new(),
            friends_tab: "all".into(),
            local_speaking: false,
            remote_speaking: false,
            last_remote_speaker_id: String::new(),
            last_voice_packet_ms: 0,
        }
    }
}
