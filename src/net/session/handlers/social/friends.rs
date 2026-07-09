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
        NetCommand::PresenceUpdate {
            status,
            custom_status,
            activity,
            activity_text,
        } => {
            let payload = serde_json::to_vec(&PresenceUpdatePayload {
                status,
                activity_type: activity,
                activity_text,
                custom_status,
            })?;
            framing::io::write_encrypted(stream, PID_PRESENCE_UPDATE, seq, &payload, crypto)
                .await?;
        }
        NetCommand::FriendRequest { target_user_id } => {
            let payload = serde_json::to_vec(&FriendRequestPayload { target_user_id })?;
            framing::io::write_encrypted(stream, PID_FRIEND_REQUEST, seq, &payload, crypto).await?;
        }
        NetCommand::FriendAccept { user_id } => {
            let payload = serde_json::to_vec(&FriendAcceptPayload { user_id })?;
            framing::io::write_encrypted(stream, PID_FRIEND_ACCEPT, seq, &payload, crypto).await?;
        }
        NetCommand::FriendDecline { user_id } => {
            let payload = serde_json::to_vec(&FriendDeclinePayload { user_id })?;
            framing::io::write_encrypted(stream, PID_FRIEND_DECLINE, seq, &payload, crypto).await?;
        }
        NetCommand::FriendRemove { user_id } => {
            let payload = serde_json::to_vec(&FriendRemovePayload { user_id })?;
            framing::io::write_encrypted(stream, PID_FRIEND_REMOVE, seq, &payload, crypto).await?;
        }
        NetCommand::FriendList => {
            framing::io::write_encrypted(stream, PID_FRIEND_LIST, seq, b"{}", crypto).await?;
        }
        NetCommand::BlockUser { user_id } => {
            let payload = serde_json::to_vec(&BlockUserPayload { user_id })?;
            framing::io::write_encrypted(stream, PID_BLOCK_USER, seq, &payload, crypto).await?;
        }
        NetCommand::UnblockUser { user_id } => {
            let payload = serde_json::to_vec(&UnblockUserPayload { user_id })?;
            framing::io::write_encrypted(stream, PID_UNBLOCK_USER, seq, &payload, crypto).await?;
        }
        NetCommand::BlockList => {
            framing::io::write_encrypted(stream, PID_BLOCK_LIST, seq, b"{}", crypto).await?;
        }
        _ => {}
    }
    Ok(())
}
