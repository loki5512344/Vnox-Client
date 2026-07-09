use vnox_client::net::NetEvent;

use crate::state::{FriendState, GuildState};

use crate::state::UiState;

pub(crate) fn handle(ui: &mut UiState, event: NetEvent) {
    match event {
        NetEvent::GuildAuditLog { guild_id, entries } => {
            ui.audit_log_entries = entries;
            ui.audit_log_guild_id = Some(guild_id);
        }
        NetEvent::GuildMemberList { guild_id, members } => {
            ui.guild_member_list_guild_id = Some(guild_id);
            ui.guild_member_list = members;
        }
        NetEvent::GuildRoleList { guild_id, roles } => {
            ui.guild_role_list_guild_id = Some(guild_id);
            ui.guild_role_list = roles;
        }
        NetEvent::BlockList { blocked } => {
            ui.blocked_users = blocked;
        }
        NetEvent::BlockedUser { user_id } => {
            if !ui.blocked_users.contains(&user_id) {
                ui.blocked_users.push(user_id);
            }
        }
        NetEvent::UnblockedUser { user_id } => {
            ui.blocked_users.retain(|u| u != &user_id);
        }
        NetEvent::GuildList { guilds } => {
            ui.guilds = guilds
                .into_iter()
                .map(|g| GuildState {
                    guild_id: g.guild_id,
                    name: g.name,
                    owner_id: g.owner_id,
                    member_count: g.member_count,
                })
                .collect();
        }
        NetEvent::GuildCreated {
            guild_id,
            name,
            owner_id,
        } => {
            ui.guilds.push(GuildState {
                guild_id: guild_id.clone(),
                name,
                owner_id,
                member_count: 1,
            });
            ui.active_guild_id = Some(guild_id);
        }
        NetEvent::GuildDeleted { guild_id } => {
            ui.guilds.retain(|g| g.guild_id != guild_id);
            if ui.active_guild_id.as_deref() == Some(&guild_id) {
                ui.active_guild_id = None;
            }
        }
        NetEvent::GuildMemberJoined {
            guild_id,
            user_id: _,
            nickname: _,
        } => {
            if let Some(g) = ui.guilds.iter_mut().find(|g| g.guild_id == guild_id) {
                g.member_count += 1;
            }
        }
        NetEvent::GuildMemberLeft {
            guild_id,
            user_id: _,
        } => {
            if let Some(g) = ui.guilds.iter_mut().find(|g| g.guild_id == guild_id) {
                g.member_count = g.member_count.saturating_sub(1);
            }
        }
        NetEvent::GuildMemberKicked {
            guild_id,
            user_id: _,
        } => {
            if let Some(g) = ui.guilds.iter_mut().find(|g| g.guild_id == guild_id) {
                g.member_count = g.member_count.saturating_sub(1);
            }
        }
        NetEvent::InviteCreated { .. } => {
            // UI will fetch invite list separately if needed
        }
        NetEvent::InviteAccepted {
            guild_id,
            guild_name,
        } if !ui.guilds.iter().any(|g| g.guild_id == guild_id) => {
            ui.guilds.push(GuildState {
                guild_id,
                name: guild_name,
                owner_id: String::new(),
                member_count: 1,
            });
        }
        NetEvent::InviteDeleted {
            guild_id: _,
            invite_id: _,
        } => {}
        NetEvent::RoleCreated { .. } => {}
        NetEvent::RoleDeleted { .. } => {}
        NetEvent::FriendList { friends } => {
            ui.friends = friends
                .into_iter()
                .map(|f| FriendState {
                    user_id: f.user_id,
                    nickname: f.nickname,
                    status: f.status,
                    since: f.since,
                })
                .collect();
        }
        NetEvent::FriendRequested { user_id, nickname } => {
            ui.pending_friend_requests.push(FriendState {
                user_id,
                nickname,
                status: "offline".into(),
                since: 0,
            });
        }
        NetEvent::FriendAccepted { user_id, nickname } => {
            // Remove from pending requests
            ui.pending_friend_requests.retain(|f| f.user_id != user_id);
            // Avoid duplicate entries in friends list
            if !ui.friends.iter().any(|f| f.user_id == user_id) {
                ui.friends.push(FriendState {
                    user_id,
                    nickname,
                    status: "online".into(),
                    since: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as i64)
                        .unwrap_or(0),
                });
            }
        }
        NetEvent::FriendRemoved { user_id } => {
            ui.friends.retain(|f| f.user_id != user_id);
            ui.pending_friend_requests.retain(|f| f.user_id != user_id);
        }
        _ => {}
    }
}
