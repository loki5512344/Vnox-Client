use std::collections::HashMap;

use vnox_client::audio::{IncomingVoice, PipelineConfig};
use vnox_client::net::{ChatMsg, NetCommand, NetEvent, NetHandle, PresenceInfo};

use crate::ui::state::{ChatMessage, ConnState, UiState, VoiceUiState};

use crate::AppState;

pub fn process_net_events(state: &mut AppState) {
    while let Some(event) = state.net.try_recv() {
        match event {
            NetEvent::Connected {
                session_id,
                node_name,
                restored,
                private_mode,
            } => {
                state.ui.conn = ConnState::Connected {
                    session_id,
                    node_name: node_name.clone(),
                };
                state.ui.private_mode = private_mode;
                if restored {
                    state.ui.status_msg = Some("reconnected".into());
                    if let Some(ch) = state.ui.active_channel.clone() {
                        let net = state.net.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::JoinChannel { channel_id: ch }).await;
                        });
                    }
                } else {
                    state.ui.seed_default_channels();
                    state.ui.active_channel = Some("general".into());
                    state.ui.status_msg = None;
                    let net = state.net.clone();
                    tokio::spawn(async move {
                        net.send(NetCommand::JoinChannel {
                            channel_id: "general".into(),
                        })
                        .await;
                    });
                }
            }
            NetEvent::Disconnected { reason, will_retry } => {
                state.ui.voice = VoiceUiState::Off;
                state.audio = None;
                state.voice_channel = None;
                state.last_pipeline_config = None;
                if will_retry {
                    state.ui.status_msg = Some(format!("connection lost: {reason}"));
                } else {
                    state.ui.conn = ConnState::Disconnected;
                    state.ui.channels.clear();
                    state.ui.active_channel = None;
                    state.ui.user_names.clear();
                    state.ui.rtt_ms = None;
                    state.ui.status_msg = Some(format!("disconnected: {reason}"));
                }
                state.ui.dm_conversations.clear();
                state.ui.dm_messages.clear();
                state.ui.active_dm_id = None;
                state.ui.new_dm_input = None;
                state.ui.private_mode = false;
            }
            NetEvent::Reconnecting {
                attempt,
                delay_secs,
            } => {
                state.ui.voice = VoiceUiState::Off;
                state.audio = None;
                state.voice_channel = None;
                state.last_pipeline_config = None;
                state.ui.conn = ConnState::Reconnecting { attempt };
                state.ui.status_msg =
                    Some(format!("reconnecting in {delay_secs}s (attempt {attempt})"));
            }
            NetEvent::VoicePacket {
                channel_id,
                voice_seq,
                timestamp,
                opus_data,
                sender_id,
                ..
            } => {
                if !sender_id.is_empty() {
                    state.ui.last_remote_speaker_id = sender_id.clone();
                    state.ui.last_voice_packet_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                }
                if !state.ui.deafen
                    && let Some(pipeline) = state.audio.as_ref()
                {
                    pipeline.push_voice(IncomingVoice {
                        voice_seq,
                        timestamp,
                        channel_id,
                        opus_data,
                    });
                }
            }
            NetEvent::E2eeDmKeyExchange {
                dm_id,
                e2ee_public_key,
            } => {
                let (secret, public) = vnox_client::e2ee::derive_e2ee_keypair(&state.identity);
                let shared = vnox_client::e2ee::compute_shared_secret(&secret, &e2ee_public_key);
                if let Some(conv) = state
                    .ui
                    .dm_conversations
                    .iter_mut()
                    .find(|c| c.dm_id == dm_id)
                {
                    conv.e2ee_enabled = true;
                    conv.e2ee_peer_public_key = Some(e2ee_public_key);
                    conv.e2ee_shared_secret = Some(shared);
                }
                let pubkey_bytes = public.to_bytes().to_vec();
                let net = state.net.clone();
                tokio::spawn(async move {
                    net.send(NetCommand::E2eeDmKeyExchange {
                        dm_id,
                        e2ee_public_key: pubkey_bytes,
                    })
                    .await;
                });
            }
            NetEvent::E2eeDmKeyExchangeAck { dm_id } => {
                if let Some(conv) = state
                    .ui
                    .dm_conversations
                    .iter_mut()
                    .find(|c| c.dm_id == dm_id)
                {
                    conv.e2ee_enabled = true;
                }
            }
            NetEvent::E2eeDmMessage {
                dm_id,
                sender_id,
                ciphertext,
                timestamp,
            } => {
                let shared = state
                    .ui
                    .dm_conversations
                    .iter()
                    .find(|c| c.dm_id == dm_id)
                    .and_then(|c| c.e2ee_shared_secret);
                let Some(shared) = shared else {
                    continue;
                };
                let msg_id = uuid::Uuid::new_v4().to_string();
                let content = vnox_client::e2ee::decrypt_message(&shared, &msg_id, &ciphertext);
                state
                    .ui
                    .dm_messages
                    .entry(dm_id.clone())
                    .or_default()
                    .push(ChatMessage {
                        message_id: msg_id,
                        sender_id,
                        content,
                        timestamp,
                        edited: false,
                        reactions: HashMap::new(),
                        reply_to: None,
                    });
                if state.ui.active_dm_id.as_deref() != Some(&dm_id)
                    && let Some(c) = state
                        .ui
                        .dm_conversations
                        .iter_mut()
                        .find(|c| c.dm_id == dm_id)
                {
                    c.unread_count += 1;
                }
            }
            NetEvent::E2eeDmHistory { dm_id, messages } => {
                let shared = state
                    .ui
                    .dm_conversations
                    .iter()
                    .find(|c| c.dm_id == dm_id)
                    .and_then(|c| c.e2ee_shared_secret);
                let Some(shared) = shared else {
                    continue;
                };
                let decrypted: Vec<ChatMessage> = messages
                    .into_iter()
                    .map(|m| {
                        let msg_id = uuid::Uuid::new_v4().to_string();
                        let content =
                            vnox_client::e2ee::decrypt_message(&shared, &msg_id, &m.ciphertext);
                        ChatMessage {
                            message_id: msg_id,
                            sender_id: m.sender_id,
                            content,
                            timestamp: m.timestamp,
                            edited: false,
                            reactions: HashMap::new(),
                            reply_to: None,
                        }
                    })
                    .collect();
                state.ui.dm_messages.insert(dm_id, decrypted);
            }
            event => process_net_event(&mut state.ui, &state.net, event),
        }
        sync_voice_audio(state);
        flush_encoded_voice(state);
    }
    if let Some(pipeline) = state.audio.as_ref() {
        state.ui.local_speaking = pipeline.local_speaking();
        state.ui.remote_speaking = pipeline.remote_speaking(500);
    } else {
        state.ui.local_speaking = false;
        state.ui.remote_speaking = false;
    }
    if !state.ui.remote_speaking {
        state.ui.last_remote_speaker_id.clear();
    }
}

fn process_net_event(ui: &mut UiState, net: &NetHandle, event: NetEvent) {
    match &event {
        NetEvent::ChannelState { .. }
        | NetEvent::ChannelCreated { .. }
        | NetEvent::ChannelDeleted { .. }
        | NetEvent::ChannelListEvent { .. }
        | NetEvent::ChatMessage { .. }
        | NetEvent::ChatHistory { .. }
        | NetEvent::UserJoin { .. }
        | NetEvent::UserLeave { .. }
        | NetEvent::Error { .. }
        | NetEvent::ReactionAdded { .. }
        | NetEvent::ReactionRemoved { .. }
        | NetEvent::MessageEdited { .. }
        | NetEvent::MessageDeleted { .. }
        | NetEvent::DmStart { .. }
        | NetEvent::DmMessage { .. }
        | NetEvent::DmHistory { .. }
        | NetEvent::ReadReceiptBroadcast { .. }
        | NetEvent::UserColor { .. } => chat_handle(ui, net, event),

        NetEvent::GuildList { .. }
        | NetEvent::GuildCreated { .. }
        | NetEvent::GuildDeleted { .. }
        | NetEvent::GuildMemberJoined { .. }
        | NetEvent::GuildMemberLeft { .. }
        | NetEvent::GuildMemberKicked { .. }
        | NetEvent::InviteCreated { .. }
        | NetEvent::InviteAccepted { .. }
        | NetEvent::InviteDeleted { .. }
        | NetEvent::RoleCreated { .. }
        | NetEvent::RoleDeleted { .. }
        | NetEvent::GuildAuditLog { .. }
        | NetEvent::GuildMemberList { .. }
        | NetEvent::GuildRoleList { .. }
        | NetEvent::FriendList { .. }
        | NetEvent::FriendRequested { .. }
        | NetEvent::FriendAccepted { .. }
        | NetEvent::FriendRemoved { .. }
        | NetEvent::BlockList { .. }
        | NetEvent::BlockedUser { .. }
        | NetEvent::UnblockedUser { .. } => social_handle(ui, event),

        NetEvent::LatencyUpdate { .. }
        | NetEvent::TypingStart { .. }
        | NetEvent::PresenceSync { .. }
        | NetEvent::PresenceUpdated { .. } => status_handle(ui, net, event),

        NetEvent::Connected { .. }
        | NetEvent::Disconnected { .. }
        | NetEvent::Reconnecting { .. }
        | NetEvent::VoicePacket { .. }
        | NetEvent::E2eeDmKeyExchange { .. }
        | NetEvent::E2eeDmKeyExchangeAck { .. }
        | NetEvent::E2eeDmMessage { .. }
        | NetEvent::E2eeDmHistory { .. } => {}
    }
}

fn into_msg(m: ChatMsg) -> ChatMessage {
    ChatMessage {
        message_id: m.message_id,
        sender_id: m.sender_id,
        content: m.content,
        timestamp: m.timestamp,
        edited: false,
        reactions: HashMap::new(),
        reply_to: m.reply_to,
    }
}

fn parse_hex_color(hex: &str) -> Option<String> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()?;
    Some(hex.to_string())
}

fn chat_handle(ui: &mut UiState, net: &NetHandle, event: NetEvent) {
    match event {
        NetEvent::ChannelState {
            channel_id,
            channel_name,
            kind,
            members,
            ..
        } => {
            let mems: Vec<String> = members
                .iter()
                .map(|m| {
                    ui.user_names.insert(m.user_id.clone(), m.nickname.clone());
                    m.nickname.clone()
                })
                .collect();
            if let Some(ch) = ui.channels.iter_mut().find(|c| c.id == channel_id) {
                ch.members = mems;
            } else {
                ui.channels.push(crate::ui::state::Channel {
                    id: channel_id.clone(),
                    name: channel_name,
                    kind: kind.clone(),
                    members: mems,
                });
            }
            if ui.active_channel.is_none() {
                ui.active_channel = Some(channel_id.clone());
            }
            let net = net.clone();
            let ch_id = channel_id;
            tokio::spawn(async move {
                net.send(NetCommand::ReadReceipt {
                    channel_id: ch_id,
                    last_read_message_id: uuid::Uuid::new_v4().to_string(),
                })
                .await;
            });
        }
        NetEvent::ChannelCreated {
            channel_id,
            channel_name,
            kind,
        } => {
            if !ui.channels.iter().any(|c| c.id == channel_id) {
                ui.channels.push(crate::ui::state::Channel {
                    id: channel_id,
                    name: channel_name,
                    kind,
                    members: Vec::new(),
                });
            }
        }
        NetEvent::ChannelDeleted { channel_id } => {
            ui.channels.retain(|c| c.id != channel_id);
            if ui.active_channel.as_deref() == Some(&channel_id) {
                ui.active_channel = ui.channels.first().map(|c| c.id.clone());
            }
            ui.messages.remove(&channel_id);
        }
        NetEvent::ChannelListEvent { channels } => {
            let old: std::collections::HashMap<String, crate::ui::state::Channel> =
                std::mem::take(&mut ui.channels)
                    .into_iter()
                    .map(|c| (c.id.clone(), c))
                    .collect();
            ui.channels = channels
                .into_iter()
                .map(|c| {
                    let members = old
                        .get(&c.channel_id)
                        .map(|o| o.members.clone())
                        .unwrap_or_default();
                    crate::ui::state::Channel {
                        id: c.channel_id,
                        name: c.channel_name,
                        kind: c.kind,
                        members,
                    }
                })
                .collect();
        }
        NetEvent::UserJoin {
            channel_id,
            user_id,
            nickname,
            ..
        } => {
            ui.user_names.insert(user_id, nickname.clone());
            if let Some(ch) = ui.channels.iter_mut().find(|c| c.id == channel_id)
                && !ch.members.contains(&nickname)
            {
                ch.members.push(nickname);
            }
        }
        NetEvent::UserLeave {
            channel_id,
            user_id,
        } => {
            if let Some(ch) = ui.channels.iter_mut().find(|c| c.id == channel_id) {
                let nickname = ui.user_names.remove(&user_id);
                ch.members.retain(|m| {
                    m != &user_id && nickname.as_deref().map(|n| m != n).unwrap_or(true)
                });
            }
        }
        NetEvent::DmStart {
            dm_id,
            other_nickname,
            messages,
            unread_count,
            ..
        } => {
            if !ui.dm_conversations.iter().any(|c| c.dm_id == dm_id) {
                ui.dm_conversations.push(crate::ui::state::DmConversation {
                    dm_id: dm_id.clone(),
                    other_nickname,
                    unread_count,
                    e2ee_enabled: false,
                    e2ee_peer_public_key: None,
                    e2ee_shared_secret: None,
                });
            }
            ui.dm_messages
                .insert(dm_id.clone(), messages.into_iter().map(into_msg).collect());
            ui.active_dm_id = Some(dm_id);
        }
        NetEvent::ChatMessage {
            message_id,
            channel_id,
            sender_id,
            content,
            timestamp,
            reply_to,
            ..
        } => {
            ui.messages
                .entry(channel_id)
                .or_default()
                .push(ChatMessage {
                    message_id,
                    sender_id,
                    content,
                    timestamp,
                    edited: false,
                    reactions: HashMap::new(),
                    reply_to,
                });
            ui.scroll_bottom = true;
        }
        NetEvent::ChatHistory {
            channel_id,
            messages,
        } => {
            ui.messages
                .insert(channel_id, messages.into_iter().map(into_msg).collect());
            ui.scroll_bottom = true;
        }
        NetEvent::Error { message, .. } => {
            ui.status_msg = Some(format!("error: {message}"));
        }
        NetEvent::ReactionAdded {
            channel_id,
            message_id,
            user_id,
            emoji,
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id)
                && let Some(msg) = msgs.iter_mut().find(|m| m.message_id == message_id)
            {
                msg.reactions.entry(emoji).or_default().push(user_id);
            }
        }
        NetEvent::ReactionRemoved {
            channel_id,
            message_id,
            user_id,
            emoji,
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id)
                && let Some(msg) = msgs.iter_mut().find(|m| m.message_id == message_id)
                && let Some(users) = msg.reactions.get_mut(&emoji)
            {
                users.retain(|u| u != &user_id);
                if users.is_empty() {
                    msg.reactions.remove(&emoji);
                }
            }
        }
        NetEvent::MessageEdited {
            channel_id,
            message_id,
            content,
            ..
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id)
                && let Some(msg) = msgs.iter_mut().find(|m| m.message_id == message_id)
            {
                msg.content = content;
                msg.edited = true;
            }
        }
        NetEvent::MessageDeleted {
            channel_id,
            message_id,
        } => {
            if let Some(msgs) = ui.messages.get_mut(&channel_id) {
                msgs.retain(|m| m.message_id != message_id);
            }
        }
        NetEvent::DmMessage {
            dm_id,
            sender_id,
            content,
            timestamp,
            ..
        } => {
            ui.dm_messages
                .entry(dm_id.clone())
                .or_default()
                .push(ChatMessage {
                    message_id: uuid::Uuid::new_v4().to_string(),
                    sender_id,
                    content,
                    timestamp,
                    edited: false,
                    reactions: HashMap::new(),
                    reply_to: None,
                });
            if ui.active_dm_id.as_deref() != Some(&dm_id)
                && let Some(c) = ui.dm_conversations.iter_mut().find(|c| c.dm_id == dm_id)
            {
                c.unread_count += 1;
            }
        }
        NetEvent::DmHistory { dm_id, messages } => {
            ui.dm_messages
                .insert(dm_id, messages.into_iter().map(into_msg).collect());
        }
        NetEvent::ReadReceiptBroadcast {
            channel_id,
            user_id,
            last_read_message_id,
        } => {
            ui.read_receipts
                .entry(channel_id)
                .or_default()
                .insert(user_id, last_read_message_id);
        }
        NetEvent::UserColor { user_id, color } => {
            if let Some(c) = parse_hex_color(&color) {
                ui.user_colors.insert(user_id, c);
            }
        }
        _ => {}
    }
}

fn social_handle(ui: &mut UiState, event: NetEvent) {
    match event {
        NetEvent::GuildAuditLog { .. } => {}
        NetEvent::GuildMemberList { .. } => {}
        NetEvent::GuildRoleList { .. } => {}
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
                .map(|g| crate::ui::state::GuildState {
                    guild_id: g.guild_id,
                    name: g.name,
                    member_count: g.member_count,
                })
                .collect();
        }
        NetEvent::GuildCreated {
            guild_id,
            name,
            ..
        } => {
            ui.guilds.push(crate::ui::state::GuildState {
                guild_id: guild_id.clone(),
                name,
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
        NetEvent::InviteCreated { .. } => {}
        NetEvent::InviteAccepted {
            guild_id,
            guild_name,
        } if !ui.guilds.iter().any(|g| g.guild_id == guild_id) => {
            ui.guilds.push(crate::ui::state::GuildState {
                guild_id,
                name: guild_name,
                member_count: 1,
            });
        }
        NetEvent::InviteDeleted { .. } => {}
        NetEvent::RoleCreated { .. } => {}
        NetEvent::RoleDeleted { .. } => {}
        NetEvent::FriendList { friends } => {
            ui.friends = friends
                .into_iter()
                .map(|f| crate::ui::state::FriendState {
                    user_id: f.user_id,
                    nickname: f.nickname,
                })
                .collect();
        }
        NetEvent::FriendRequested { user_id, nickname } => {
            ui.pending_friend_requests
                .push(crate::ui::state::FriendState {
                    user_id,
                    nickname,
                });
        }
        NetEvent::FriendAccepted { user_id, nickname } => {
            ui.pending_friend_requests.retain(|f| f.user_id != user_id);
            if !ui.friends.iter().any(|f| f.user_id == user_id) {
                ui.friends.push(crate::ui::state::FriendState {
                    user_id,
                    nickname,
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

fn status_handle(ui: &mut UiState, _net: &NetHandle, event: NetEvent) {
    match event {
        NetEvent::LatencyUpdate { rtt_ms } => {
            ui.rtt_ms = Some(rtt_ms);
        }
        NetEvent::TypingStart {
            user_id,
            nickname,
            channel_id,
        } => {
            let entry = ui.typing_users.entry(channel_id).or_default();
            entry.retain(|(u, _)| u != &user_id);
            entry.push((nickname, std::time::Instant::now()));
        }
        NetEvent::PresenceSync { presences } => {
            for p in presences {
                ui.presences.insert(p.user_id.clone(), p);
            }
        }
        NetEvent::PresenceUpdated {
            user_id,
            status,
            custom_status,
            activity,
        } => {
            ui.presences.insert(
                user_id.clone(),
                PresenceInfo {
                    user_id,
                    status,
                    custom_status,
                    activity,
                    activity_text: None,
                },
            );
        }
        _ => {}
    }
}

pub fn pipeline_config(channel_id: &str, state: &AppState) -> PipelineConfig {
    PipelineConfig {
        channel_key: vnox_client::net::voice::channel_key(channel_id),
        mic_device_index: state.ui.mic_device,
        output_device_index: state.ui.output_device,
        voice_bitrate_kbps: state.ui.voice_bitrate,
        packet_interval: state.ui.packet_interval,
    }
}

pub fn start_voice(state: &mut AppState, channel_id: &str) {
    let cfg = pipeline_config(channel_id, state);
    if state.voice_channel.as_deref() == Some(channel_id)
        && state.audio.is_some()
        && state.last_pipeline_config.as_ref() == Some(&cfg)
    {
        return;
    }
    state.audio = None;
    match vnox_client::audio::start(cfg.clone()) {
        Ok(pipeline) => {
            tracing::info!("voice active on channel {channel_id}");
            state.audio = Some(pipeline);
            state.voice_channel = Some(channel_id.to_string());
            state.last_pipeline_config = Some(cfg);
            state.ui.voice = VoiceUiState::Active {
                channel: channel_id.to_string(),
            };
            if state.ui.voice_joined_at.is_none() {
                state.ui.voice_joined_at = Some(std::time::Instant::now());
            }
            state.ui.status_msg = None;
        }
        Err(e) => {
            tracing::warn!("audio start failed: {e}");
            state.voice_channel = None;
            state.last_pipeline_config = None;
            state.ui.voice = VoiceUiState::Error(e.to_string());
            state.ui.status_msg = Some(format!("microphone error: {e}"));
        }
    }
}

pub fn stop_voice(state: &mut AppState) {
    state.audio = None;
    state.voice_channel = None;
    state.last_pipeline_config = None;
    state.ui.voice = VoiceUiState::Off;
    state.ui.voice_joined_at = None;
}

pub fn sync_voice_audio(state: &mut AppState) {
    if !matches!(state.ui.conn, ConnState::Connected { .. }) {
        stop_voice(state);
        return;
    }
    let active_voice = state
        .ui
        .active_channel
        .as_ref()
        .and_then(|id| state.ui.channels.iter().find(|c| &c.id == id))
        .filter(|c| c.kind == "voice")
        .map(|c| c.id.clone());
    match active_voice {
        Some(id) => start_voice(state, &id),
        None => stop_voice(state),
    }
}

pub fn flush_encoded_voice(state: &mut AppState) {
    let Some(pipeline) = state.audio.as_ref() else {
        return;
    };
    while let Some(frame) = pipeline.poll_encoded() {
        let net = state.net.clone();
        tokio::spawn(async move {
            net.send(NetCommand::SendVoice {
                channel_id: frame.channel_id,
                voice_seq: frame.voice_seq,
                timestamp: frame.timestamp,
                opus_data: frame.data,
            })
            .await;
        });
    }
}
