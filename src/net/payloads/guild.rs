use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildCreatePayload {
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildInfoPayload {
    pub guild_id: String,
    pub name: String,
    pub owner_id: String,
    pub icon_url: Option<String>,
    pub member_count: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildListPayload {
    pub guilds: Vec<GuildInfoPayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildDeletePayload {
    pub guild_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildJoinPayload {
    pub guild_id: String,
    pub invite_code: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildLeavePayload {
    pub guild_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRoleUpdatePayload {
    pub user_id: String,
    pub guild_id: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteCreatePayload {
    pub guild_id: String,
    pub max_uses: Option<i64>,
    pub expires_in_seconds: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteAcceptPayload {
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteDeletePayload {
    pub guild_id: String,
    pub invite_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleCreatePayload {
    pub guild_id: String,
    pub name: String,
    pub color: Option<String>,
    pub permissions: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleDeletePayload {
    pub guild_id: String,
    pub role_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildMemberKickPayload {
    pub guild_id: String,
    pub user_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InviteInfoPayload {
    pub id: String,
    pub guild_id: String,
    pub guild_name: String,
    pub code: String,
    pub creator_id: String,
    pub max_uses: Option<i64>,
    pub uses: i64,
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildAuditLogFetchPayload {
    pub guild_id: String,
    #[serde(default = "default_audit_limit")]
    pub limit: i64,
}

fn default_audit_limit() -> i64 {
    50
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuditLogEntryPayload {
    pub id: String,
    pub guild_id: String,
    pub actor_id: String,
    pub action: String,
    #[serde(default)]
    pub target_id: Option<String>,
    #[serde(default)]
    pub target_type: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildAuditLogPayload {
    pub guild_id: String,
    pub entries: Vec<AuditLogEntryPayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildMemberListFetchPayload {
    pub guild_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildMemberInfoPayload {
    pub user_id: String,
    pub nickname: String,
    pub joined_at: i64,
    pub role_color: String,
    pub role_name: String,
    pub is_owner: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildMemberListPayload {
    pub guild_id: String,
    pub members: Vec<GuildMemberInfoPayload>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleAssignPayload {
    pub guild_id: String,
    pub user_id: String,
    pub role_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GuildRoleListFetchPayload {
    pub guild_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildRoleInfoPayload {
    pub id: String,
    pub guild_id: String,
    pub name: String,
    pub color: String,
    pub permissions: u64,
    pub position: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuildRoleListPayload {
    pub guild_id: String,
    pub roles: Vec<GuildRoleInfoPayload>,
}
