#[derive(Debug)]
pub enum NetCommand {
    Connect {
        address: String,
        auto_reconnect: bool,
    },
    JoinChannel {
        channel_id: String,
    },
    LeaveChannel {
        channel_id: String,
    },
    ChannelCreate {
        channel_id: String,
        channel_name: String,
        kind: String,
    },
    ChannelDelete {
        channel_id: String,
    },
    ChannelList,
    SendChat {
        channel_id: String,
        content: String,
        /// Optional message_id this message is replying to.
        reply_to: Option<String>,
    },
    SendVoice {
        channel_id: u64,
        voice_seq: u32,
        timestamp: u32,
        opus_data: Vec<u8>,
    },
    DmStart {
        target_user_id: String,
    },
    DmSend {
        dm_id: String,
        content: String,
    },
    DmReadAck {
        dm_id: String,
    },
    DmSearch {
        dm_id: String,
        query: String,
    },
    GuildCreate {
        name: String,
    },
    GuildDelete {
        guild_id: String,
    },
    GuildList,
    GuildJoin {
        guild_id: String,
        invite_code: Option<String>,
    },
    GuildLeave {
        guild_id: String,
    },
    InviteCreate {
        guild_id: String,
        max_uses: Option<i64>,
        expires_in_seconds: Option<i64>,
    },
    InviteAccept {
        code: String,
    },
    InviteDelete {
        guild_id: String,
        invite_id: String,
    },
    RoleCreate {
        guild_id: String,
        name: String,
        color: Option<String>,
        permissions: Option<u64>,
    },
    RoleDelete {
        guild_id: String,
        role_id: String,
    },
    GuildMemberKick {
        guild_id: String,
        user_id: String,
    },
    GuildAuditLogFetch {
        guild_id: String,
        limit: i64,
    },
    GuildMemberListFetch {
        guild_id: String,
    },
    GuildRoleAssign {
        guild_id: String,
        user_id: String,
        role_id: String,
    },
    GuildRoleUnassign {
        guild_id: String,
        user_id: String,
        role_id: String,
    },
    GuildRoleListFetch {
        guild_id: String,
    },
    PresenceUpdate {
        status: String,
        custom_status: Option<String>,
        activity: Option<String>,
        /// Free-form text describing the current activity (e.g. "playing CS2").
        activity_text: Option<String>,
    },
    TypingStart {
        channel_id: String,
    },
    ReadReceipt {
        channel_id: String,
        last_read_message_id: String,
    },
    FriendRequest {
        target_user_id: String,
    },
    FriendAccept {
        user_id: String,
    },
    FriendDecline {
        user_id: String,
    },
    FriendRemove {
        user_id: String,
    },
    FriendList,
    BlockUser {
        user_id: String,
    },
    UnblockUser {
        user_id: String,
    },
    BlockList,
    ReactionAdd {
        channel_id: String,
        message_id: String,
        emoji: String,
    },
    ReactionRemove {
        channel_id: String,
        message_id: String,
        emoji: String,
    },
    MessageEdit {
        channel_id: String,
        message_id: String,
        content: String,
    },
    MessageDelete {
        channel_id: String,
        message_id: String,
    },
    E2eeDmKeyExchange {
        dm_id: String,
        e2ee_public_key: Vec<u8>,
    },
    E2eeDmKeyExchangeAck {
        dm_id: String,
    },
    E2eeDmSend {
        dm_id: String,
        ciphertext: Vec<u8>,
    },
    E2eeDmHistory {
        dm_id: String,
        limit: Option<i64>,
    },
    Disconnect,
}
