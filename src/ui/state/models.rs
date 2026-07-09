use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum SettingsPage {
    Identity,
    #[default]
    Voice,
    AudioOutput,
    Network,
    Overlay,
    Appearance,
    Keybinds,
    Plugins,
    Advanced,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub kind: String,
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
    pub other_user_id: String,
    pub other_nickname: String,
    pub unread_count: u32,
}

#[derive(Debug, Clone)]
pub struct GuildState {
    pub guild_id: String,
    pub name: String,
    pub owner_id: String,
    pub member_count: u32,
}

#[derive(Debug, Clone)]
pub struct FriendState {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub since: i64,
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

pub(crate) fn now_utc_hms() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let h = (secs % 86400) / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}
