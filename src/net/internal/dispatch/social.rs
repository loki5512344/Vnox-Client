use anyhow::Result;
use serde_json;
use tokio::sync::mpsc;

use super::super::super::payloads::*;
use super::super::super::types::*;

pub async fn handle(pid: u16, payload: &[u8], tx: &mpsc::Sender<NetEvent>) -> Result<()> {
    match pid {
        PID_GUILD_LIST => {
            let p: GuildListPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildList {
                    guilds: p
                        .guilds
                        .into_iter()
                        .map(|g| GuildInfo {
                            guild_id: g.guild_id,
                            name: g.name,
                            owner_id: g.owner_id,
                            icon_url: g.icon_url,
                            member_count: g.member_count,
                        })
                        .collect(),
                })
                .await;
        }
        PID_GUILD_CREATE => {
            let p: GuildInfoPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildCreated {
                    guild_id: p.guild_id,
                    name: p.name,
                    owner_id: p.owner_id,
                })
                .await;
        }
        PID_GUILD_DELETE => {
            let p: GuildDeletePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildDeleted {
                    guild_id: p.guild_id,
                })
                .await;
        }
        PID_GUILD_MEMBER_JOIN => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildMemberJoined {
                    guild_id: p["guild_id"].as_str().unwrap_or_default().into(),
                    user_id: p["user_id"].as_str().unwrap_or_default().into(),
                    nickname: p["nickname"].as_str().unwrap_or_default().into(),
                })
                .await;
        }
        PID_GUILD_MEMBER_LEAVE => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildMemberLeft {
                    guild_id: p["guild_id"].as_str().unwrap_or_default().into(),
                    user_id: p["user_id"].as_str().unwrap_or_default().into(),
                })
                .await;
        }
        PID_USER_ROLE_UPDATE => {
            let p: UserRoleUpdatePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::UserColor {
                    user_id: p.user_id,
                    color: p.color,
                })
                .await;
        }
        PID_PRESENCE_SYNC => {
            let p: PresenceSyncPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::PresenceSync {
                    presences: p
                        .presences
                        .into_iter()
                        .map(|p| PresenceInfo {
                            user_id: p.user_id,
                            status: p.status,
                            custom_status: p.custom_status,
                            activity: p.activity,
                            activity_text: p.activity_text,
                        })
                        .collect(),
                })
                .await;
        }
        PID_PRESENCE_EVENT => {
            let p: PresenceEventPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::PresenceUpdated {
                    user_id: p.user_id,
                    status: p.status,
                    custom_status: p.custom_status,
                    activity: p.activity,
                })
                .await;
        }
        PID_GUILD_MEMBER_KICK => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildMemberKicked {
                    guild_id: p["guild_id"].as_str().unwrap_or_default().into(),
                    user_id: p["user_id"].as_str().unwrap_or_default().into(),
                })
                .await;
        }
        PID_INVITE_CREATE => {
            let p: InviteInfoPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::InviteCreated {
                    id: p.id,
                    guild_id: p.guild_id,
                    guild_name: p.guild_name,
                    code: p.code,
                    creator_id: p.creator_id,
                    max_uses: p.max_uses,
                    uses: p.uses,
                    expires_at: p.expires_at,
                    created_at: p.created_at,
                })
                .await;
        }
        PID_INVITE_ACCEPT => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::InviteAccepted {
                    guild_id: p["guild_id"].as_str().unwrap_or_default().into(),
                    guild_name: p["guild_name"].as_str().unwrap_or_default().into(),
                })
                .await;
        }
        PID_INVITE_DELETE => {
            let p: InviteDeletePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::InviteDeleted {
                    guild_id: p.guild_id,
                    invite_id: p.invite_id,
                })
                .await;
        }
        PID_ROLE_CREATE => {
            let p: serde_json::Value = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::RoleCreated {
                    role_id: p["id"].as_str().unwrap_or_default().into(),
                    guild_id: p["guild_id"].as_str().unwrap_or_default().into(),
                    name: p["name"].as_str().unwrap_or_default().into(),
                })
                .await;
        }
        PID_ROLE_DELETE => {
            let p: RoleDeletePayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::RoleDeleted {
                    role_id: p.role_id,
                    guild_id: p.guild_id,
                })
                .await;
        }
        PID_GUILD_AUDIT_LOG => {
            let p: GuildAuditLogPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildAuditLog {
                    guild_id: p.guild_id,
                    entries: p.entries,
                })
                .await;
        }
        PID_GUILD_MEMBER_LIST => {
            let p: GuildMemberListPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildMemberList {
                    guild_id: p.guild_id,
                    members: p.members,
                })
                .await;
        }
        PID_GUILD_ROLE_LIST => {
            let p: GuildRoleListPayload = serde_json::from_slice(payload)?;
            let _ = tx
                .send(NetEvent::GuildRoleList {
                    guild_id: p.guild_id,
                    roles: p.roles,
                })
                .await;
        }
        PID_BLOCK_LIST => {
            // Server may send {"blocked": [...]} — try to parse properly first,
            // fall back to JSON value extraction.
            let blocked: Vec<String> = match serde_json::from_slice::<BlockListPayload>(payload) {
                Ok(p) => p.blocked,
                Err(_) => {
                    let v: serde_json::Value = serde_json::from_slice(payload)?;
                    v["blocked"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|x| x.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default()
                }
            };
            let _ = tx.send(NetEvent::BlockList { blocked }).await;
        }
        PID_BLOCK_USER => {
            let v: serde_json::Value = serde_json::from_slice(payload).unwrap_or_default();
            let user_id = v["user_id"]
                .as_str()
                .or_else(|| v["blocked"].as_str())
                .unwrap_or_default()
                .to_string();
            let _ = tx.send(NetEvent::BlockedUser { user_id }).await;
        }
        PID_UNBLOCK_USER => {
            let v: serde_json::Value = serde_json::from_slice(payload).unwrap_or_default();
            let user_id = v["user_id"]
                .as_str()
                .or_else(|| v["unblocked"].as_str())
                .unwrap_or_default()
                .to_string();
            let _ = tx.send(NetEvent::UnblockedUser { user_id }).await;
        }
        _ => {}
    }
    Ok(())
}
