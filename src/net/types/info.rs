#[derive(Debug, Clone)]
pub struct MemberInfo {
    pub user_id: String,
    pub nickname: String,
    pub in_voice: bool,
}

#[derive(Debug, Clone)]
pub struct ChannelListItem {
    pub channel_id: String,
    pub channel_name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct ChatMsg {
    pub message_id: String,
    pub sender_id: String,
    pub content: String,
    pub timestamp: i64,
    /// Optional message_id this message is replying to.
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GuildInfo {
    pub guild_id: String,
    pub name: String,
    pub owner_id: String,
    pub icon_url: Option<String>,
    pub member_count: u32,
}

#[derive(Debug, Clone)]
pub struct PresenceInfo {
    pub user_id: String,
    pub status: String,
    pub custom_status: Option<String>,
    pub activity: Option<String>,
    pub activity_text: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FriendInfo {
    pub user_id: String,
    pub nickname: String,
    pub status: String,
    pub since: i64,
}
