use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct E2eeDmKeyExchangePayload {
    pub dm_id: String,
    pub e2ee_public_key: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct E2eeDmKeyExchangeAckPayload {
    pub dm_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct E2eeDmMessagePayload {
    pub dm_id: String,
    pub sender_id: String,
    #[serde(with = "serde_bytes")]
    pub ciphertext: Vec<u8>,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct E2eeDmHistoryPayload {
    pub dm_id: String,
    #[serde(default)]
    pub messages: Vec<E2eeDmMessagePayload>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DmStartPayload {
    pub target_user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DmStartResponsePayload {
    pub dm_id: String,
    pub other_user_id: String,
    pub other_nickname: String,
    pub messages: Vec<DmMessagePayload>,
    #[serde(default)]
    pub unread_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DmMessagePayload {
    pub dm_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DmHistoryPayload {
    pub dm_id: String,
    #[serde(default)]
    pub messages: Vec<DmMessagePayload>,
    #[serde(default)]
    pub search_query: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PresenceUpdatePayload {
    pub status: String,
    /// Free-form activity type ("playing", "listening", "watching", "streaming").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    /// Free-form activity text (e.g. "CS2 ranked").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_text: Option<String>,
    /// Free-form custom status text (Discord-style "what's on your mind?").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PresenceInfoPayload {
    pub user_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PresenceSyncPayload {
    pub presences: Vec<PresenceInfoPayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PresenceEventPayload {
    pub user_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRequestPayload {
    pub target_user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendAcceptPayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendDeclinePayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRemovePayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FriendInfoPayload {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub since: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendListPayload {
    pub friends: Vec<FriendInfoPayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockUserPayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnblockUserPayload {
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockListPayload {
    pub blocked: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendEventPayload {
    pub event_type: String,
    pub user_id: String,
    pub nickname: Option<String>,
}
