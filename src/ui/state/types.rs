use std::collections::HashMap;
use std::time::Instant;

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
    pub scroll_bottom: bool,

    pub voice: VoiceUiState,
    pub voice_joined_at: Option<Instant>,

    pub user_names: HashMap<String, String>,
    pub rtt_ms: Option<u32>,

    pub settings_open: bool,

    pub mic_enabled: bool,
    pub deafen: bool,
    pub vad_mode: usize,
    pub voice_bitrate: f32,
    pub packet_interval: usize,
    pub mic_device: usize,

    pub output_device: usize,
    pub output_volume: f32,

    pub auto_reconnect: bool,

    pub private_mode: bool,

    pub guilds: Vec<super::GuildState>,
    pub active_guild_id: Option<String>,

    pub presences: HashMap<String, crate::net::PresenceInfo>,

    pub friends: Vec<super::FriendState>,
    pub pending_friend_requests: Vec<super::FriendState>,
    pub blocked_users: Vec<String>,

    pub dm_conversations: Vec<super::DmConversation>,
    pub dm_messages: HashMap<String, Vec<super::ChatMessage>>,
    pub active_dm_id: Option<String>,
    pub new_dm_input: Option<String>,

    pub user_colors: HashMap<String, String>,

    pub typing_users: HashMap<String, Vec<(String, Instant)>>,

    pub read_receipts: HashMap<String, HashMap<String, String>>,

    pub friends_tab: String,

    pub local_speaking: bool,
    pub remote_speaking: bool,
    pub last_remote_speaker_id: String,
    pub last_voice_packet_ms: u64,

    pub replying_to_message: Option<String>,
    pub replying_to_sender: String,
    pub replying_to_content: String,
    pub editing_message_id: Option<String>,

    pub settings_passphrase_open: bool,
    pub settings_passphrase_input: String,
    pub settings_passphrase_confirm: String,
    pub settings_passphrase_error: Option<String>,

    pub settings_export_open: bool,
    pub settings_export_pass: String,
    pub settings_export_confirm: String,
    pub settings_export_output: Option<String>,
    pub settings_export_error: Option<String>,

    pub settings_import_open: bool,
    pub settings_import_input: String,
    pub settings_import_pass: String,
    pub settings_import_error: Option<String>,
}
