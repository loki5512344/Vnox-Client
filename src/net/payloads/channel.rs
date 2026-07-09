use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HelloPayload {
    pub lnex_version: String,
    pub server_pubkey: String,
    pub challenge_nonce: String,
    pub node_name: String,
    pub server_eph_pubkey: String,
    #[serde(default)]
    pub private_mode: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthPayload {
    pub client_pubkey: String,
    pub nickname: String,
    pub lnex_version: String,
    pub signature: String,
    pub client_eph_pubkey: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionPayload {
    pub session_id: String,
    pub token: String,
    pub expires_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PingPayload {
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PongPayload {
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinChannelPayload {
    pub channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaveChannelPayload {
    pub channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChannelStatePayload {
    pub channel_id: String,
    pub channel_name: String,
    pub kind: String,
    pub members: Vec<WireMember>,
    pub voice_endpoint: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelCreatePayload {
    pub channel_id: String,
    pub channel_name: String,
    /// "text" or "voice".
    pub kind: String,
    #[serde(default)]
    pub guild_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelDeletePayload {
    pub channel_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelListPayload {
    pub channels: Vec<ChannelListItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelListItem {
    pub channel_id: String,
    pub channel_name: String,
    pub kind: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WireMember {
    pub user_id: String,
    pub nickname: String,
    pub in_voice: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessagePayload {
    pub message_id: String,
    pub channel_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(default)]
    pub edited: bool,
    /// Optional message_id this message is replying to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatHistoryPayload {
    pub channel_id: String,
    pub messages: Vec<ChatMessagePayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReactionPayload {
    pub message_id: String,
    pub channel_id: String,
    pub emoji: String,
    #[serde(default)]
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageEditPayload {
    pub message_id: String,
    pub channel_id: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageDeletePayload {
    pub message_id: String,
    pub channel_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserJoinPayload {
    pub channel_id: String,
    pub user_id: String,
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserLeavePayload {
    pub channel_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorPayload {
    pub code: u32,
    pub message: String,
}
