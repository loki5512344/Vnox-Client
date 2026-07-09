use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

pub const TYPING_TIMEOUT: Duration = Duration::from_secs(4);

#[derive(Debug, Clone, PartialEq, Default)]
pub enum VoiceUiState {
    #[default]
    Off,
    Active {
        channel: String,
    },
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnState {
    Disconnected,
    Connecting,
    Reconnecting {
        attempt: u32,
    },
    Connected {
        session_id: String,
        node_name: String,
    },
}

pub struct UiState {
    pub conn: ConnState,
    pub connect_input: String,
    pub bookmarks: Vec<super::NodeBookmark>,
    pub selected_bookmark: usize,
    pub status_msg: Option<String>,

    pub channels: Vec<super::Channel>,
    pub active_channel: Option<String>,
    pub messages: HashMap<String, Vec<super::ChatMessage>>,
    pub chat_input: String,
    pub editing_message_id: Option<String>,
    /// When set, the next sent message will be a reply to this message_id.
    pub replying_to: Option<(String, String)>, // (message_id, sender_id)
    /// Message currently shown in the context menu (right-click).
    pub context_menu_message_id: Option<String>,
    pub scroll_bottom: bool,

    pub voice: VoiceUiState,
    pub voice_joined_at: Option<Instant>,

    pub user_names: HashMap<String, String>,
    pub server_channel: Option<String>,
    pub rtt_ms: Option<u32>,

    pub settings_open: bool,
    pub settings_page: super::SettingsPage,

    pub mic_enabled: bool,
    pub deafen: bool,
    pub noise_suppress: bool,
    pub echo_cancel: bool,
    pub vad_mode: usize,
    pub vad_threshold: f32,
    pub voice_bitrate: f32,
    pub packet_interval: usize,
    pub mic_device: usize,
    pub input_volume: f32,
    pub input_device_labels: Vec<String>,
    pub output_device_labels: Vec<String>,
    pub input_device_metas: Vec<vnox_client::audio::devices::DeviceLabel>,
    pub output_device_metas: Vec<vnox_client::audio::devices::DeviceLabel>,
    pub last_device_scan: Option<Instant>,

    pub output_device: usize,
    pub output_volume: f32,
    pub jitter_size: usize,
    pub adaptive_buffer: bool,

    pub overlay_enabled: bool,
    pub overlay_speakers: bool,
    pub overlay_latency: bool,

    pub relay_enabled: bool,
    pub relay_address: String,
    pub auto_relay: bool,
    pub force_relay: bool,
    pub udp_port: String,
    pub auto_reconnect: bool,

    pub ui_scale: f32,

    pub kb_ptt: String,
    pub kb_mute: String,
    pub kb_deafen: String,
    pub kb_overlay: String,

    pub log_level: usize,
    pub log_voice_packets: bool,
    pub show_packet_stats: bool,
    pub disable_encryption: bool,

    pub private_mode: bool,

    pub guilds: Vec<super::GuildState>,
    pub active_guild_id: Option<String>,

    pub presences: HashMap<String, crate::net::PresenceInfo>,
    pub presence_status: Option<String>,
    /// Free-form custom status text ("making tea").
    pub custom_status: Option<String>,
    /// Activity type ("playing", "listening", "watching", "streaming").
    pub activity_type: Option<String>,
    /// Activity text (e.g. "Counter-Strike 2").
    pub activity_text: Option<String>,

    pub friends: Vec<super::FriendState>,
    pub pending_friend_requests: Vec<super::FriendState>,
    /// Blocked user IDs.
    pub blocked_users: Vec<String>,
    /// Input buffer for the block-user text field.
    pub block_input: Option<String>,
    /// When the blocked list was last fetched from the server.
    pub block_list_fetched_at: Option<Instant>,

    pub dm_conversations: Vec<super::DmConversation>,
    pub dm_messages: HashMap<String, Vec<super::ChatMessage>>,
    pub active_dm_id: Option<String>,
    pub new_dm_input: Option<String>,
    pub dm_search_query: String,

    pub user_colors: HashMap<String, eframe::egui::Color32>,

    pub per_user_volumes: HashMap<String, f32>,

    pub add_server_open: bool,
    pub add_server_input: String,
    pub invite_accept_open: bool,
    pub invite_accept_code: String,

    pub create_guild_open: bool,
    pub create_guild_input: String,

    /// "Create a Channel" popup state.
    pub create_channel_open: bool,
    pub create_channel_input: String,
    pub create_channel_kind: String,

    /// Vault UI state — set/remove passphrase modals.
    pub vault_set_open: bool,
    pub vault_remove_open: bool,
    pub vault_passphrase_input: String,
    pub vault_passphrase_confirm: String,
    pub vault_error: Option<String>,

    /// Export/import keyfile UI state.
    pub export_keyfile_open: bool,
    pub export_keyfile_pass: String,
    pub export_keyfile_confirm: String,
    pub export_keyfile_output: String,
    pub export_keyfile_error: Option<String>,
    pub import_keyfile_open: bool,
    pub import_keyfile_input: String,
    pub import_keyfile_pass: String,
    pub import_keyfile_error: Option<String>,

    /// Audit log viewer state.
    pub audit_log_open: bool,
    pub audit_log_guild_id: Option<String>,
    pub audit_log_entries: Vec<vnox_client::net::payloads::AuditLogEntryPayload>,

    /// Guild member list viewer state.
    pub guild_members_open: bool,
    pub guild_member_list_guild_id: Option<String>,
    pub guild_member_list: Vec<vnox_client::net::payloads::GuildMemberInfoPayload>,

    /// Guild role list state.
    pub guild_role_list_guild_id: Option<String>,
    pub guild_role_list: Vec<vnox_client::net::payloads::GuildRoleInfoPayload>,

    pub add_friend_open: bool,
    pub add_friend_input: String,

    pub kb_listening: Option<String>,

    pub collapsed_categories: HashSet<String>,

    pub startup_time_str: String,

    pub typing_users: HashMap<String, Vec<(String, Instant)>>,
    pub last_typing_send: Instant,

    pub read_receipts: HashMap<String, HashMap<String, String>>,

    /// Active tab in the Friends panel ("online" / "all" / "pending" / "blocked").
    pub friends_tab: String,

    /// True when local user is transmitting voice (PTT held or VAD above threshold).
    pub local_speaking: bool,
    /// True when a remote peer's voice packet arrived in the last ~500ms.
    pub remote_speaking: bool,
    /// Most recently received voice-packet sender's user_id (hex pubkey).
    /// Empty when no remote user has spoken recently.
    pub last_remote_speaker_id: String,
    /// UNIX millisecond timestamp of the last voice packet from any remote peer.
    pub last_voice_packet_ms: u64,
}
