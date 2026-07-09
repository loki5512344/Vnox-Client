use super::info::{ChannelListItem, ChatMsg, FriendInfo, GuildInfo, MemberInfo, PresenceInfo};
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub enum NetEvent {
    Connected {
        session_id: String,
        node_name: String,
        restored: bool,
        private_mode: bool,
    },
    Disconnected {
        reason: String,
        will_retry: bool,
    },
    Reconnecting {
        attempt: u32,
        delay_secs: u64,
    },
    ChannelState {
        channel_id: String,
        channel_name: String,
        kind: String,
        members: Vec<MemberInfo>,
        voice_endpoint: String,
    },
    ChannelCreated {
        channel_id: String,
        channel_name: String,
        kind: String,
    },
    ChannelDeleted {
        channel_id: String,
    },
    ChannelListEvent {
        channels: Vec<ChannelListItem>,
    },
    ChatMessage {
        message_id: String,
        channel_id: String,
        sender_id: String,
        nickname: String,
        content: String,
        timestamp: i64,
        /// Optional message_id this message is replying to.
        reply_to: Option<String>,
    },
    ChatHistory {
        channel_id: String,
        messages: Vec<ChatMsg>,
    },
    UserJoin {
        channel_id: String,
        user_id: String,
        nickname: String,
    },
    UserLeave {
        channel_id: String,
        user_id: String,
    },
    VoicePacket {
        channel_id: u64,
        voice_seq: u32,
        timestamp: u32,
        sender: SocketAddr,
        /// Hex-encoded Ed25519 public key of the sender (Phase 1.3 attribution).
        /// Empty for legacy packets that didn't include sender_id.
        sender_id: String,
        opus_data: Vec<u8>,
    },
    Error {
        code: u32,
        message: String,
    },
    LatencyUpdate {
        rtt_ms: u32,
    },
    DmStart {
        dm_id: String,
        other_user_id: String,
        other_nickname: String,
        messages: Vec<ChatMsg>,
        unread_count: u32,
    },
    DmMessage {
        dm_id: String,
        sender_id: String,
        content: String,
        timestamp: i64,
    },
    DmHistory {
        dm_id: String,
        messages: Vec<ChatMsg>,
    },
    TypingStart {
        user_id: String,
        nickname: String,
        channel_id: String,
    },
    ReadReceiptBroadcast {
        channel_id: String,
        user_id: String,
        last_read_message_id: String,
    },
    GuildList {
        guilds: Vec<GuildInfo>,
    },
    GuildCreated {
        guild_id: String,
        name: String,
        owner_id: String,
    },
    GuildDeleted {
        guild_id: String,
    },
    GuildMemberJoined {
        guild_id: String,
        user_id: String,
        nickname: String,
    },
    GuildMemberLeft {
        guild_id: String,
        user_id: String,
    },
    GuildMemberKicked {
        guild_id: String,
        user_id: String,
    },
    InviteCreated {
        id: String,
        guild_id: String,
        guild_name: String,
        code: String,
        creator_id: String,
        max_uses: Option<i64>,
        uses: i64,
        expires_at: Option<i64>,
        created_at: i64,
    },
    InviteAccepted {
        guild_id: String,
        guild_name: String,
    },
    InviteDeleted {
        guild_id: String,
        invite_id: String,
    },
    RoleCreated {
        role_id: String,
        guild_id: String,
        name: String,
    },
    RoleDeleted {
        role_id: String,
        guild_id: String,
    },
    GuildAuditLog {
        guild_id: String,
        entries: Vec<crate::net::payloads::AuditLogEntryPayload>,
    },
    GuildMemberList {
        guild_id: String,
        members: Vec<crate::net::payloads::GuildMemberInfoPayload>,
    },
    GuildRoleList {
        guild_id: String,
        roles: Vec<crate::net::payloads::GuildRoleInfoPayload>,
    },
    UserColor {
        user_id: String,
        color: String,
    },
    PresenceSync {
        presences: Vec<PresenceInfo>,
    },
    PresenceUpdated {
        user_id: String,
        status: String,
        custom_status: Option<String>,
        activity: Option<String>,
    },
    FriendList {
        friends: Vec<FriendInfo>,
    },
    FriendRequested {
        user_id: String,
        nickname: String,
    },
    FriendAccepted {
        user_id: String,
        nickname: String,
    },
    FriendRemoved {
        user_id: String,
    },
    BlockList {
        blocked: Vec<String>,
    },
    BlockedUser {
        user_id: String,
    },
    UnblockedUser {
        user_id: String,
    },
    ReactionAdded {
        channel_id: String,
        message_id: String,
        user_id: String,
        emoji: String,
    },
    ReactionRemoved {
        channel_id: String,
        message_id: String,
        user_id: String,
        emoji: String,
    },
    MessageEdited {
        channel_id: String,
        message_id: String,
        content: String,
        edited: bool,
    },
    MessageDeleted {
        channel_id: String,
        message_id: String,
    },
}
