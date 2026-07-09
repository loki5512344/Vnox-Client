mod friends;
mod guild;

use anyhow::Result;
use serde_json;
use tokio::net::TcpStream;

use crate::identity::Identity;
use crate::net::{crypto::SessionCrypto, framing, payloads::*, types::NetCommand};

use super::core::now_ms;

pub async fn handle_cmd(
    cmd: NetCommand,
    stream: &mut TcpStream,
    seq: &mut u32,
    crypto: &SessionCrypto,
    identity: &Identity,
) -> Result<()> {
    match cmd {
        cmd @ (NetCommand::GuildCreate { .. }
        | NetCommand::GuildDelete { .. }
        | NetCommand::GuildList
        | NetCommand::GuildJoin { .. }
        | NetCommand::GuildLeave { .. }
        | NetCommand::InviteCreate { .. }
        | NetCommand::InviteAccept { .. }
        | NetCommand::InviteDelete { .. }
        | NetCommand::RoleCreate { .. }
        | NetCommand::RoleDelete { .. }
        | NetCommand::GuildMemberKick { .. }
        | NetCommand::GuildAuditLogFetch { .. }
        | NetCommand::GuildMemberListFetch { .. }
        | NetCommand::GuildRoleAssign { .. }
        | NetCommand::GuildRoleUnassign { .. }
        | NetCommand::GuildRoleListFetch { .. }) => {
            guild::handle_cmd(cmd, stream, seq, crypto).await?
        }

        cmd @ (NetCommand::PresenceUpdate { .. }
        | NetCommand::FriendRequest { .. }
        | NetCommand::FriendAccept { .. }
        | NetCommand::FriendDecline { .. }
        | NetCommand::FriendRemove { .. }
        | NetCommand::FriendList
        | NetCommand::BlockUser { .. }
        | NetCommand::UnblockUser { .. }
        | NetCommand::BlockList) => friends::handle_cmd(cmd, stream, seq, crypto).await?,

        NetCommand::ReactionAdd {
            channel_id,
            message_id,
            emoji,
        } => {
            let payload = serde_json::to_vec(&ReactionPayload {
                message_id,
                channel_id,
                emoji,
                user_id: String::new(),
            })?;
            framing::io::write_encrypted(stream, PID_MESSAGE_REACTION_ADD, seq, &payload, crypto)
                .await?;
        }
        NetCommand::ReactionRemove {
            channel_id,
            message_id,
            emoji,
        } => {
            let payload = serde_json::to_vec(&ReactionPayload {
                message_id,
                channel_id,
                emoji,
                user_id: String::new(),
            })?;
            framing::io::write_encrypted(
                stream,
                PID_MESSAGE_REACTION_REMOVE,
                seq,
                &payload,
                crypto,
            )
            .await?;
        }
        NetCommand::MessageEdit {
            channel_id,
            message_id,
            content,
        } => {
            let payload = serde_json::to_vec(&MessageEditPayload {
                message_id,
                channel_id,
                content,
            })?;
            framing::io::write_encrypted(stream, PID_MESSAGE_EDIT, seq, &payload, crypto).await?;
        }
        NetCommand::MessageDelete {
            channel_id,
            message_id,
        } => {
            let payload = serde_json::to_vec(&MessageDeletePayload {
                message_id,
                channel_id,
            })?;
            framing::io::write_encrypted(stream, PID_MESSAGE_DELETE, seq, &payload, crypto).await?;
        }
        NetCommand::DmStart { target_user_id } => {
            let payload = serde_json::to_vec(&DmStartPayload { target_user_id })?;
            framing::io::write_encrypted(stream, PID_DM_START, seq, &payload, crypto).await?;
        }
        NetCommand::DmSend { dm_id, content } => {
            let msg = DmMessagePayload {
                dm_id,
                sender_id: identity.pubkey_hex.clone(),
                content,
                timestamp: now_ms(),
            };
            framing::io::write_encrypted(
                stream,
                PID_DM_MESSAGE,
                seq,
                &serde_json::to_vec(&msg)?,
                crypto,
            )
            .await?;
        }
        NetCommand::TypingStart { channel_id } => {
            let payload = serde_json::to_vec(&serde_json::json!({"channel_id": channel_id}))?;
            framing::io::write_encrypted(stream, PID_TYPING_START, seq, &payload, crypto).await?;
        }
        NetCommand::DmReadAck { dm_id } => {
            let payload = serde_json::to_vec(&serde_json::json!({"dm_id": dm_id}))?;
            framing::io::write_encrypted(stream, PID_DM_READ_ACK, seq, &payload, crypto).await?;
        }
        NetCommand::DmSearch { dm_id, query } => {
            let payload = serde_json::to_vec(&DmHistoryPayload {
                dm_id,
                messages: vec![],
                search_query: if query.is_empty() { None } else { Some(query) },
                limit: None,
            })?;
            framing::io::write_encrypted(stream, PID_DM_HISTORY, seq, &payload, crypto).await?;
        }
        _ => {}
    }
    Ok(())
}
