use anyhow::Result;
use tokio::sync::mpsc;
use tracing::debug;

use super::super::payloads::*;
use super::super::types::*;

mod channel;
mod dm;
mod social;

pub async fn incoming(pid: u16, payload: &[u8], tx: &mpsc::Sender<NetEvent>) -> Result<()> {
    match pid {
        PID_CHANNEL_STATE
        | PID_CHAT_MESSAGE
        | PID_CHAT_HISTORY
        | PID_USER_JOIN
        | PID_USER_LEAVE
        | PID_PONG
        | PID_READ_RECEIPT_BROADCAST
        | PID_ERROR
        | PID_DISCONNECT
        | PID_TYPING_START => channel::handle(pid, payload, tx).await?,

        PID_DM_START
        | PID_DM_MESSAGE
        | PID_DM_HISTORY
        | PID_FRIEND_LIST
        | PID_MESSAGE_REACTION_ADD
        | PID_MESSAGE_REACTION_REMOVE
        | PID_MESSAGE_EDIT
        | PID_MESSAGE_DELETE
        | PID_FRIEND_EVENT
        | PID_E2EE_DM_KEY_EXCHANGE
        | PID_E2EE_DM_KEY_EXCHANGE_ACK
        | PID_E2EE_DM_MESSAGE
        | PID_E2EE_DM_HISTORY => dm::handle(pid, payload, tx).await?,

        PID_GUILD_LIST
        | PID_GUILD_CREATE
        | PID_GUILD_DELETE
        | PID_GUILD_MEMBER_JOIN
        | PID_GUILD_MEMBER_LEAVE
        | PID_GUILD_MEMBER_KICK
        | PID_INVITE_CREATE
        | PID_INVITE_ACCEPT
        | PID_INVITE_DELETE
        | PID_ROLE_CREATE
        | PID_ROLE_DELETE
        | PID_USER_ROLE_UPDATE
        | PID_PRESENCE_SYNC
        | PID_PRESENCE_EVENT => social::handle(pid, payload, tx).await?,

        other => debug!("unhandled 0x{:04X}", other),
    }
    Ok(())
}
