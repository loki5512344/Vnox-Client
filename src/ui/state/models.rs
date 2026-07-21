use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub guild_id: Option<String>,
    pub members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub message_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
    pub edited: bool,
    pub reactions: HashMap<String, Vec<String>>,
    /// Optional message_id this message is replying to.
    #[cfg_attr(not(test), allow(dead_code))]
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DmConversation {
    pub dm_id: String,
    pub other_nickname: String,
    pub unread_count: u32,
    pub e2ee_enabled: bool,
    pub e2ee_peer_public_key: Option<Vec<u8>>,
    pub e2ee_shared_secret: Option<[u8; 32]>,
}

#[derive(Debug, Clone)]
pub struct GuildState {
    pub guild_id: String,
    pub name: String,
    pub member_count: u32,
}

#[derive(Debug, Clone)]
pub struct FriendState {
    pub user_id: String,
    pub nickname: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeBookmark {
    pub label: String,
    pub address: String,
}

pub(crate) fn default_bookmarks() -> Vec<NodeBookmark> {
    Vec::new()
}

pub(crate) fn bookmark_label_from_address(address: &str) -> String {
    address
        .split(':')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or(address)
        .chars()
        .take(12)
        .collect()
}
