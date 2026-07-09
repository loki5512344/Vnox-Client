use anyhow::Result;
use serde_json;
use tokio::net::TcpStream;

use crate::net::{crypto::SessionCrypto, framing, payloads::*, types::NetCommand};

pub async fn handle_cmd(
    cmd: NetCommand,
    stream: &mut TcpStream,
    seq: &mut u32,
    crypto: &SessionCrypto,
) -> Result<()> {
    match cmd {
        NetCommand::GuildCreate { name } => {
            let payload = serde_json::to_vec(&GuildCreatePayload {
                name,
                icon_url: None,
            })?;
            framing::io::write_encrypted(stream, PID_GUILD_CREATE, seq, &payload, crypto).await?;
        }
        NetCommand::GuildDelete { guild_id } => {
            let payload = serde_json::to_vec(&GuildDeletePayload { guild_id })?;
            framing::io::write_encrypted(stream, PID_GUILD_DELETE, seq, &payload, crypto).await?;
        }
        NetCommand::GuildList => {
            framing::io::write_encrypted(stream, PID_GUILD_LIST, seq, b"{}", crypto).await?;
        }
        NetCommand::GuildJoin {
            guild_id,
            invite_code,
        } => {
            let payload = serde_json::to_vec(&GuildJoinPayload {
                guild_id,
                invite_code,
            })?;
            framing::io::write_encrypted(stream, PID_GUILD_MEMBER_JOIN, seq, &payload, crypto)
                .await?;
        }
        NetCommand::GuildLeave { guild_id } => {
            let payload = serde_json::to_vec(&GuildLeavePayload { guild_id })?;
            framing::io::write_encrypted(stream, PID_GUILD_MEMBER_LEAVE, seq, &payload, crypto)
                .await?;
        }
        NetCommand::InviteCreate {
            guild_id,
            max_uses,
            expires_in_seconds,
        } => {
            let payload = serde_json::to_vec(&InviteCreatePayload {
                guild_id,
                max_uses,
                expires_in_seconds,
            })?;
            framing::io::write_encrypted(stream, PID_INVITE_CREATE, seq, &payload, crypto).await?;
        }
        NetCommand::InviteAccept { code } => {
            let payload = serde_json::to_vec(&InviteAcceptPayload { code })?;
            framing::io::write_encrypted(stream, PID_INVITE_ACCEPT, seq, &payload, crypto).await?;
        }
        NetCommand::InviteDelete {
            guild_id,
            invite_id,
        } => {
            let payload = serde_json::to_vec(&InviteDeletePayload {
                guild_id,
                invite_id,
            })?;
            framing::io::write_encrypted(stream, PID_INVITE_DELETE, seq, &payload, crypto).await?;
        }
        NetCommand::RoleCreate {
            guild_id,
            name,
            color,
            permissions,
        } => {
            let payload = serde_json::to_vec(&RoleCreatePayload {
                guild_id,
                name,
                color,
                permissions,
            })?;
            framing::io::write_encrypted(stream, PID_ROLE_CREATE, seq, &payload, crypto).await?;
        }
        NetCommand::RoleDelete { guild_id, role_id } => {
            let payload = serde_json::to_vec(&RoleDeletePayload { guild_id, role_id })?;
            framing::io::write_encrypted(stream, PID_ROLE_DELETE, seq, &payload, crypto).await?;
        }
        NetCommand::GuildMemberKick { guild_id, user_id } => {
            let payload = serde_json::to_vec(&GuildMemberKickPayload { guild_id, user_id })?;
            framing::io::write_encrypted(stream, PID_GUILD_MEMBER_KICK, seq, &payload, crypto)
                .await?;
        }
        NetCommand::GuildAuditLogFetch { guild_id, limit } => {
            let payload = serde_json::to_vec(&GuildAuditLogFetchPayload { guild_id, limit })?;
            framing::io::write_encrypted(stream, PID_GUILD_AUDIT_LOG_FETCH, seq, &payload, crypto)
                .await?;
        }
        NetCommand::GuildMemberListFetch { guild_id } => {
            let payload = serde_json::to_vec(&GuildMemberListFetchPayload { guild_id })?;
            framing::io::write_encrypted(
                stream,
                PID_GUILD_MEMBER_LIST_FETCH,
                seq,
                &payload,
                crypto,
            )
            .await?;
        }
        NetCommand::GuildRoleAssign {
            guild_id,
            user_id,
            role_id,
        } => {
            let payload = serde_json::to_vec(&RoleAssignPayload {
                guild_id,
                user_id,
                role_id,
            })?;
            framing::io::write_encrypted(stream, PID_GUILD_ROLE_ASSIGN, seq, &payload, crypto)
                .await?;
        }
        NetCommand::GuildRoleUnassign {
            guild_id,
            user_id,
            role_id,
        } => {
            let payload = serde_json::to_vec(&RoleAssignPayload {
                guild_id,
                user_id,
                role_id,
            })?;
            framing::io::write_encrypted(stream, PID_GUILD_ROLE_UNASSIGN, seq, &payload, crypto)
                .await?;
        }
        NetCommand::GuildRoleListFetch { guild_id } => {
            let payload = serde_json::to_vec(&GuildRoleListFetchPayload { guild_id })?;
            framing::io::write_encrypted(stream, PID_GUILD_ROLE_LIST_FETCH, seq, &payload, crypto)
                .await?;
        }
        _ => {}
    }
    Ok(())
}
