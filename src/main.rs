mod ui;


use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::ui::slint::MainWindow;
use anyhow::Result;
use slint::ComponentHandle;
use tracing::info;
use tracing_subscriber::EnvFilter;
use vnox_client::audio::AudioPipeline;
use vnox_client::audio::PipelineConfig;
use vnox_client::identity;
use vnox_client::net;
use vnox_client::net::NetCommand;
use vnox_client::net::NetHandle;

use crate::ui::state::{ConnState, UiState, VoiceUiState};

pub struct AppState {
    pub(crate) identity: identity::Identity,
    pub(crate) net: NetHandle,
    pub(crate) ui: UiState,
    pub(crate) audio: Option<AudioPipeline>,
    pub(crate) voice_channel: Option<String>,
    pub(crate) last_pipeline_config: Option<PipelineConfig>,
}

fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,wgpu=warn,wgpu_hal=warn,naga=warn"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let identity = identity::load_or_generate()?;
    info!("identity: {}", identity.short_id());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _guard = rt.enter();

    let net = net::spawn(identity.clone());

    let state = Rc::new(RefCell::new(AppState {
        identity,
        net,
        ui: UiState::default(),
        audio: None,
        voice_channel: None,
        last_pipeline_config: None,
    }));

    let window = MainWindow::new()?;

    wire_signals(&window, &state);

    let timer = slint::Timer::default();
    {
        let state = state.clone();
        let weak = window.as_weak();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(16),
            move || {
                if let Some(win) = weak.upgrade() {
                    let mut s = state.borrow_mut();
                    crate::ui::update::process_net_events(&mut s);
                    let own_id = s.identity.short_id().to_string();
                    crate::ui::bridge::sync_ui(&win, &s.ui, &own_id);
                }
            },
        );
    }

    window.run()?;
    Ok(())
}

fn wire_signals(window: &MainWindow, state: &Rc<RefCell<AppState>>) {
    {
        let state = state.clone();
        window.on_channel_selected(move |id| {
            let id = id.to_string();
            let mut s = state.borrow_mut();
            s.ui.active_channel = Some(id.clone());
            s.ui.active_dm_id = None;
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::JoinChannel { channel_id: id }).await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_guild_selected(move |idx| {
            let mut s = state.borrow_mut();
            s.ui.active_guild_id = if idx >= 0 && (idx as usize) < s.ui.guilds.len() {
                Some(s.ui.guilds[idx as usize].guild_id.clone())
            } else {
                None
            };
        });
    }

    {
        let state = state.clone();
        window.on_dm_selected(move |nickname| {
            let nick = nickname.to_string();
            let mut s = state.borrow_mut();
            if let Some(conv) =
                s.ui.dm_conversations
                    .iter()
                    .find(|c| c.other_nickname == nick)
            {
                s.ui.active_dm_id = Some(conv.dm_id.clone());
                s.ui.active_channel = None;
            }
        });
    }

    {
        let state = state.clone();
        window.on_connect(move |addr| {
            let addr = addr.to_string();
            let mut s = state.borrow_mut();
            s.ui.conn = ConnState::Connecting;
            s.ui.connect_input = addr.clone();
            s.ui.status_msg = Some("connecting...".into());
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::Connect {
                    address: addr,
                    auto_reconnect: true,
                })
                .await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_disconnect(move || {
            let s = state.borrow();
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::Disconnect).await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_send_message(move |ch_id, text| {
            let ch_id = ch_id.to_string();
            let text = text.to_string();
            if text.is_empty() {
                return;
            }
            let mut s = state.borrow_mut();
            s.ui.chat_input = String::new();
            let net = s.net.clone();
            drop(s);

            if !ch_id.is_empty() {
                tokio::spawn(async move {
                    net.send(NetCommand::SendChat {
                        channel_id: ch_id,
                        content: text,
                        reply_to: None,
                    })
                    .await;
                });
            } else {
                let s = state.borrow();
                if let Some(dm_id) = s.ui.active_dm_id.clone() {
                    let net = s.net.clone();
                    drop(s);
                    tokio::spawn(async move {
                        net.send(NetCommand::DmSend {
                            dm_id,
                            content: text,
                        })
                        .await;
                    });
                }
            }
        });
    }

    {
        let state = state.clone();
        window.on_add_friend(move |user_id| {
            let user_id = user_id.to_string();
            let s = state.borrow();
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::FriendRequest {
                    target_user_id: user_id,
                })
                .await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_accept_friend(move |user_id| {
            let user_id = user_id.to_string();
            let s = state.borrow();
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::FriendAccept { user_id }).await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_decline_friend(move |user_id| {
            let user_id = user_id.to_string();
            let s = state.borrow();
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::FriendDecline { user_id }).await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_remove_friend(move |user_id| {
            let user_id = user_id.to_string();
            let s = state.borrow();
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::FriendRemove { user_id }).await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_start_dm(move |user_id| {
            let user_id = user_id.to_string();
            let s = state.borrow();
            let net = s.net.clone();
            drop(s);
            tokio::spawn(async move {
                net.send(NetCommand::DmStart {
                    target_user_id: user_id,
                })
                .await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_toggle_mic(move || {
            let mut s = state.borrow_mut();
            s.ui.mic_enabled = !s.ui.mic_enabled;
        });
    }

    {
        let state = state.clone();
        window.on_toggle_deafen(move || {
            let mut s = state.borrow_mut();
            s.ui.deafen = !s.ui.deafen;
        });
    }

    {
        let state = state.clone();
        window.on_leave_voice(move || {
            let mut s = state.borrow_mut();
            s.ui.active_channel = None;
            s.voice_channel = None;
            s.audio = None;
            s.last_pipeline_config = None;
            s.ui.voice = VoiceUiState::Off;
            s.ui.voice_joined_at = None;
        });
    }

    {
        let state = state.clone();
        window.on_add_bookmark(move |_addr| {
            state.borrow_mut().ui.add_bookmark_from_input();
        });
    }

    {
        let state = state.clone();
        window.on_remove_bookmark(move |address| {
            let addr = address.to_string();
            let mut s = state.borrow_mut();
            if let Some(idx) = s.ui.bookmarks.iter().position(|b| b.address == addr) {
                s.ui.remove_bookmark(idx);
            }
        });
    }

    {
        let state = state.clone();
        window.on_load_more(move |ch_id| {
            let _ch_id = ch_id.to_string();
            let state = state.borrow();
            let net = state.net.clone();
            drop(state);
            tokio::spawn(async move {
                net.send(NetCommand::ChannelList).await;
            });
        });
    }

    {
        let state = state.clone();
        window.on_close_settings(move || {
            state.borrow_mut().ui.settings_open = false;
        });
    }

    {
        let _state = state.clone();
        window.on_set_passphrase(move |_pass| {});
    }

    {
        let _state = state.clone();
        window.on_export_keyfile(move || {});
    }

    {
        let _state = state.clone();
        window.on_import_keyfile(move || {});
    }
}
