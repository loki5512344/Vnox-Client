use std::rc::Rc;

use slint::{SharedString, VecModel};

use crate::ui::state::{ConnState, UiState};

type MainWindow = crate::ui::slint::MainWindow;
type BookmarkItem = crate::ui::slint::BookmarkItem;
type ChannelItem = crate::ui::slint::ChannelItem;
type GuildItem = crate::ui::slint::GuildItem;
type MessageItem = crate::ui::slint::MessageItem;
type MemberItem = crate::ui::slint::MemberItem;
type FriendItem = crate::ui::slint::FriendItem;
type VoiceMemberItem = crate::ui::slint::VoiceMemberItem;

fn fmt_ts(ts: i64) -> String {
    let secs = (ts / 1000).abs() % 86400;
    format!("{:02}:{:02}", secs / 3600, (secs % 3600) / 60)
}

fn set_messages(window: &MainWindow, msgs: &[crate::ui::state::ChatMessage], own_user_id: &str) {
    let items: Vec<MessageItem> = msgs
        .iter()
        .map(|m| MessageItem {
            message_id: SharedString::from(m.message_id.as_str()),
            sender_id: SharedString::from(m.sender_id.as_str()),
            sender_name: SharedString::from(m.sender_id.as_str()),
            content: SharedString::from(m.content.as_str()),
            timestamp: SharedString::from(fmt_ts(m.timestamp)),
            is_own: m.sender_id == own_user_id,
            edited: m.edited,
            reply_to: SharedString::from(m.reply_to.as_deref().unwrap_or("")),
        })
        .collect();
    window.set_messages(Rc::new(VecModel::from(items)).into());
}

fn set_members_state(window: &MainWindow, state: &UiState, ch_id: &str) {
    if let Some(ch) = state.channels.iter().find(|c| c.id == ch_id) {
        let items: Vec<MemberItem> = ch
            .members
            .iter()
            .map(|nick| {
                let user_id = state
                    .user_names
                    .iter()
                    .find(|(_, n)| *n == nick)
                    .map(|(id, _)| id.clone())
                    .unwrap_or_else(|| nick.clone());
                let status = state
                    .presences
                    .get(&user_id)
                    .map(|p| p.status.as_str())
                    .unwrap_or("offline");
                MemberItem {
                    user_id: SharedString::from(user_id),
                    nickname: SharedString::from(nick.as_str()),
                    status: SharedString::from(status),
                }
            })
            .collect();
        window.set_members(Rc::new(VecModel::from(items)).into());
    }
}

fn sync_sidebar(window: &MainWindow, state: &UiState, own_nickname: &str) {
    let guild_id = state.active_guild_id.as_deref();

    let channels: Vec<ChannelItem> = state
        .channels
        .iter()
        .map(|c| ChannelItem {
            channel_id: SharedString::from(c.id.as_str()),
            name: SharedString::from(c.name.as_str()),
            kind: SharedString::from(c.kind.as_str()),
        })
        .collect();
    window.set_channels(Rc::new(VecModel::from(channels)).into());

    let guilds: Vec<GuildItem> = state
        .guilds
        .iter()
        .map(|g| {
            let short = if g.name.len() >= 2 {
                g.name[..2].to_string()
            } else {
                g.name.clone()
            };
            GuildItem {
                guild_id: SharedString::from(g.guild_id.as_str()),
                name: SharedString::from(g.name.as_str()),
                short_name: SharedString::from(short),
                selected: Some(g.guild_id.as_str()) == guild_id,
            }
        })
        .collect();
    window.set_guilds(Rc::new(VecModel::from(guilds)).into());

    let dms: Vec<SharedString> = state
        .dm_conversations
        .iter()
        .map(|c| SharedString::from(c.other_nickname.as_str()))
        .collect();
    window.set_dm_conversations(Rc::new(VecModel::from(dms)).into());

    window.set_username(SharedString::from(own_nickname));
    window.set_user_status(SharedString::from("online"));
    window.set_show_dms(guild_id.is_none() && !state.dm_conversations.is_empty());
    window.set_replying_to_message(SharedString::from(
        state.replying_to_message.as_deref().unwrap_or(""),
    ));
    window.set_replying_to_sender(SharedString::from(state.replying_to_sender.as_str()));
    window.set_replying_to_content(SharedString::from(state.replying_to_content.as_str()));
    window.set_active_channel_id(SharedString::from(
        state.active_channel.as_deref().unwrap_or(""),
    ));
    window.set_active_guild_idx(state.active_guild_id.clone().map_or(-1, |id| {
        state
            .guilds
            .iter()
            .position(|g| g.guild_id == id)
            .map_or(-1, |i| i as i32)
    }));
}

fn sync_chat(window: &MainWindow, state: &UiState, own_user_id: &str) {
    if let Some(ch_id) = &state.active_channel {
        if let Some(ch) = state.channels.iter().find(|c| &c.id == ch_id) {
            window.set_active_channel_name(SharedString::from(ch.name.as_str()));
            if ch.kind == "voice" {
                window.set_show_chat(false);
                window.set_show_voice(true);
                window.set_show_friends(false);
            } else {
                window.set_show_chat(true);
                window.set_show_voice(false);
                window.set_show_friends(false);
                if let Some(msgs) = state.messages.get(ch_id) {
                    set_messages(window, msgs, own_user_id);
                } else {
                    window.set_messages(Rc::new(VecModel::from(vec![])).into());
                }
                set_members_state(window, state, ch_id);
            }
        }
    } else {
        window.set_show_chat(false);
        window.set_show_voice(false);
        window.set_show_friends(true);
    }
    window.set_input_text(SharedString::from(state.chat_input.as_str()));
}

fn sync_friends(window: &MainWindow, state: &UiState) {
    let friends: Vec<FriendItem> = state
        .friends
        .iter()
        .map(|f| FriendItem {
            user_id: SharedString::from(f.user_id.as_str()),
            nickname: SharedString::from(f.nickname.as_str()),
            status: SharedString::from(
                state
                    .presences
                    .get(&f.user_id)
                    .map(|p| p.status.as_str())
                    .unwrap_or("offline"),
            ),
        })
        .collect();
    window.set_friends(Rc::new(VecModel::from(friends)).into());

    let pending: Vec<FriendItem> = state
        .pending_friend_requests
        .iter()
        .map(|f| FriendItem {
            user_id: SharedString::from(f.user_id.as_str()),
            nickname: SharedString::from(f.nickname.as_str()),
            status: SharedString::from("pending"),
        })
        .collect();
    window.set_pending_requests(Rc::new(VecModel::from(pending)).into());

    let blocked: Vec<SharedString> = state
        .blocked_users
        .iter()
        .map(|u| SharedString::from(u.as_str()))
        .collect();
    window.set_blocked_users(Rc::new(VecModel::from(blocked)).into());

    window.set_friends_active_tab(SharedString::from(state.friends_tab.as_str()));
}

pub fn sync_voice(window: &MainWindow, state: &UiState) {
    let members: Vec<VoiceMemberItem> = state
        .channels
        .iter()
        .filter(|c| {
            matches!(
                &state.voice,
                crate::ui::state::VoiceUiState::Active { channel } if channel == &c.id
            )
        })
        .flat_map(|c| c.members.iter())
        .map(|nick| {
            let user_id = state
                .user_names
                .iter()
                .find(|(_, n)| *n == nick)
                .map(|(id, _)| id.clone())
                .unwrap_or_else(|| nick.clone());
            VoiceMemberItem {
                user_id: SharedString::from(user_id.clone()),
                nickname: SharedString::from(nick.as_str()),
                speaking: state.last_remote_speaker_id == user_id,
            }
        })
        .collect();
    window.set_voice_members(Rc::new(VecModel::from(members)).into());
    window.set_mic_enabled(state.mic_enabled);
    window.set_deafened(state.deafen);

    if let crate::ui::state::VoiceUiState::Active { channel } = &state.voice {
        if let Some(ch) = state.channels.iter().find(|c| &c.id == channel) {
            window.set_voice_channel_name(SharedString::from(ch.name.as_str()));
        }
    }

    let banner = match (state.local_speaking, state.remote_speaking) {
        (true, true) => "you and others are talking".to_string(),
        (true, false) => "you are talking".to_string(),
        (false, true) => format!(
            "{} is talking",
            state.nick_for(&state.last_remote_speaker_id)
        ),
        _ => String::new(),
    };
    window.set_activity_banner(SharedString::from(banner));
}

pub fn sync_connect(window: &MainWindow, state: &UiState) {
    let bookmarks: Vec<BookmarkItem> = state
        .bookmarks
        .iter()
        .map(|bm| BookmarkItem {
            name: SharedString::from(bm.label.as_str()),
            address: SharedString::from(bm.address.as_str()),
        })
        .collect();
    window.set_bookmarks(Rc::new(VecModel::from(bookmarks)).into());
    window.set_address_input(SharedString::from(state.connect_input.as_str()));
    window.set_status_text(SharedString::from(
        state.status_msg.as_deref().unwrap_or(""),
    ));
    window.set_connecting(matches!(
        state.conn,
        ConnState::Connecting | ConnState::Reconnecting { .. }
    ));
    window.set_auto_reconnect(state.auto_reconnect);
    if let ConnState::Connected { ref node_name, .. } = state.conn {
        window.set_node_name(SharedString::from(node_name.as_str()));
    } else {
        window.set_node_name(SharedString::from(""));
    }
}

pub fn sync_settings(window: &MainWindow, state: &UiState, own_pubkey: &str) {
    window.set_identity_pubkey(SharedString::from(own_pubkey));
    window.set_vault_enabled(false);
    window.set_voice_input_device(SharedString::from(""));
    window.set_voice_output_device(SharedString::from(""));
    window.set_voice_bitrate(state.voice_bitrate);
    window.set_vad_mode(state.vad_mode as i32);
    window.set_volume(state.output_volume);
    window.set_active_tab(SharedString::from("audio"));
    window.set_settings_open(state.settings_open);

    window.set_passphrase_modal_open(state.settings_passphrase_open);
    window.set_passphrase_input(SharedString::from(state.settings_passphrase_input.as_str()));
    window.set_passphrase_confirm(SharedString::from(
        state.settings_passphrase_confirm.as_str(),
    ));
    window.set_passphrase_error(SharedString::from(
        state.settings_passphrase_error.as_deref().unwrap_or(""),
    ));

    window.set_export_modal_open(state.settings_export_open);
    window.set_export_pass_input(SharedString::from(state.settings_export_pass.as_str()));
    window.set_export_pass_confirm(SharedString::from(state.settings_export_confirm.as_str()));
    window.set_export_output(SharedString::from(
        state.settings_export_output.as_deref().unwrap_or(""),
    ));
    window.set_export_error(SharedString::from(
        state.settings_export_error.as_deref().unwrap_or(""),
    ));

    window.set_import_modal_open(state.settings_import_open);
    window.set_import_keyfile_input(SharedString::from(state.settings_import_input.as_str()));
    window.set_import_pass_input(SharedString::from(state.settings_import_pass.as_str()));
    window.set_import_error(SharedString::from(
        state.settings_import_error.as_deref().unwrap_or(""),
    ));
}

pub fn sync_ui(
    window: &MainWindow,
    state: &UiState,
    own_user_id: &str,
    own_nickname: &str,
    own_pubkey: &str,
) {
    sync_sidebar(window, state, own_nickname);
    sync_chat(window, state, own_user_id);
    sync_friends(window, state);
    sync_voice(window, state);
    sync_connect(window, state);
    sync_settings(window, state, own_pubkey);
}
