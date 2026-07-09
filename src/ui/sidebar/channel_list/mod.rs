mod items;

use eframe::egui::{self, Frame};
use vnox_client::net::{NetCommand, NetHandle};

use crate::state::{Channel, UiState, VoiceUiState};
use crate::theme as t;

pub fn show(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    if matches!(s.conn, crate::state::ConnState::Connected { .. }) && s.channels.is_empty() {
        s.seed_default_channels();
        if s.active_channel.is_none() {
            s.active_channel = Some("general".into());
        }
    }

    if let Some(guild_id) = &s.active_guild_id.clone()
        && let Some(g) = s.guilds.iter().find(|g| g.guild_id == *guild_id)
    {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(&g.name)
                    .font(egui::FontId::proportional(14.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                Frame::NONE
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!("{} members", g.member_count))
                                .font(egui::FontId::proportional(10.0))
                                .color(t::TEXT_MUTED),
                        );
                    });
                // Create channel button ("+").
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("+")
                                .font(egui::FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(t::BG_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                        .corner_radius(t::R_SM)
                        .min_size(egui::vec2(20.0, 18.0)),
                    )
                    .on_hover_text("create a channel")
                    .clicked()
                {
                    s.create_channel_open = true;
                    s.create_channel_input.clear();
                }
                let guild_id = guild_id.clone();
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Leave")
                                .font(egui::FontId::proportional(10.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(t::BG_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                        .corner_radius(t::R_SM),
                    )
                    .on_hover_text("leave this guild")
                    .clicked()
                {
                    s.active_guild_id = None;
                    let net = net.clone();
                    tokio::spawn(async move {
                        net.send(NetCommand::GuildLeave { guild_id }).await;
                    });
                }
            });
        });
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(4.0);
    } else {
        // Home (no guild) — show a "Create Channel" button in the channel header.
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new("CHANNELS")
                    .font(egui::FontId::proportional(10.0))
                    .color(t::TEXT_MUTED)
                    .strong(),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("+")
                                .font(egui::FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(t::BG_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                        .corner_radius(t::R_SM)
                        .min_size(egui::vec2(20.0, 18.0)),
                    )
                    .on_hover_text("create a channel")
                    .clicked()
                {
                    s.create_channel_open = true;
                    s.create_channel_input.clear();
                }
            });
        });
        ui.add_space(2.0);
        ui.separator();
        ui.add_space(4.0);
    }

    if s.create_channel_open {
        show_create_channel_popup(ui.ctx(), s, net);
    }

    let text_chs: Vec<Channel> = s
        .channels
        .iter()
        .filter(|c| c.kind == "text")
        .cloned()
        .collect();
    let voice_chs: Vec<Channel> = s
        .channels
        .iter()
        .filter(|c| c.kind == "voice")
        .cloned()
        .collect();

    if !text_chs.is_empty() {
        cat_label(ui, s, "text");
        if !s.collapsed_categories.contains("text") {
            for ch in &text_chs {
                let active = s.active_channel.as_deref() == Some(&ch.id);
                if items::ch_row_with_menu(ui, "#", &ch.name, active, None, false, Some(net)) {
                    join(s, net, &ch.id);
                }
            }
        }
    }

    if !voice_chs.is_empty() {
        ui.add_space(4.0);
        cat_label(ui, s, "voice");
        if !s.collapsed_categories.contains("voice") {
            for ch in &voice_chs {
                let active = s.active_channel.as_deref() == Some(&ch.id);
                let live =
                    matches!(&s.voice, VoiceUiState::Active { channel } if channel == &ch.id);
                let badge = (!ch.members.is_empty()).then(|| ch.members.len().to_string());
                if items::ch_row_with_menu(
                    ui,
                    "~",
                    &ch.name,
                    active,
                    badge.as_deref(),
                    live,
                    Some(net),
                ) {
                    join(s, net, &ch.id);
                }
                if live || ch.name == "lobby" {
                    for member in &ch.members {
                        items::member_row(ui, member);
                    }
                }
            }
        }
    }

    if s.channels.is_empty() {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("waiting for channels…")
                .font(egui::FontId::proportional(11.0))
                .color(t::TEXT_DISABLED),
        );
    }
}

fn cat_label(ui: &mut egui::Ui, s: &mut UiState, text: &str) {
    ui.add_space(2.0);
    let collapsed = s.collapsed_categories.contains(text);
    let arrow = if collapsed { "▶" } else { "▼" };
    let resp = Frame::NONE
        .inner_margin(egui::Margin {
            left: 12,
            right: 8,
            top: 2,
            bottom: 2,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(arrow)
                        .font(egui::FontId::proportional(8.0))
                        .color(t::TEXT_MUTED),
                );
                ui.label(
                    egui::RichText::new(text.to_uppercase())
                        .font(egui::FontId::proportional(10.0))
                        .color(t::TEXT_MUTED),
                );
            });
        })
        .response
        .interact(egui::Sense::click());
    if resp.clicked() {
        if collapsed {
            s.collapsed_categories.remove(text);
        } else {
            s.collapsed_categories.insert(text.to_string());
        }
    }
}

fn join(s: &mut UiState, net: &NetHandle, channel_id: &str) {
    s.active_channel = Some(channel_id.to_string());
    s.server_channel = Some(channel_id.to_string());
    let net = net.clone();
    let id = channel_id.to_string();
    tokio::spawn(async move {
        net.send(NetCommand::JoinChannel { channel_id: id }).await;
    });
}

fn show_create_channel_popup(ctx: &egui::Context, s: &mut UiState, net: &NetHandle) {
    egui::Window::new("Create a Channel")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG),
        )
        .show(ctx, |ui| {
            ui.set_width(300.0);
            ui.label(
                egui::RichText::new("Channel Name")
                    .font(egui::FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.add_space(8.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut s.create_channel_input)
                    .font(egui::FontId::proportional(14.0))
                    .text_color(t::TEXT_PRIMARY)
                    .hint_text(egui::RichText::new("e.g. dev-talk").color(t::TEXT_DISABLED))
                    .desired_width(260.0),
            );
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Type")
                    .font(egui::FontId::proportional(12.0))
                    .color(t::TEXT_PRIMARY),
            );
            ui.add_space(4.0);
            let kind_text = match s.create_channel_kind.as_str() {
                "voice" => "Voice",
                _ => "Text",
            };
            egui::ComboBox::from_id_salt("channel_kind")
                .selected_text(kind_text)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut s.create_channel_kind, "text".into(), "Text");
                    ui.selectable_value(&mut s.create_channel_kind, "voice".into(), "Voice");
                });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            egui::RichText::new("Cancel")
                                .font(egui::FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    s.create_channel_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let create_btn = egui::Button::new(
                        egui::RichText::new("Create")
                            .font(egui::FontId::proportional(12.0))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(t::ACCENT)
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(80.0, 28.0));
                    if ui.add(create_btn).clicked() && !s.create_channel_input.trim().is_empty() {
                        let name = s.create_channel_input.trim().to_string();
                        let kind = s.create_channel_kind.clone();
                        // Send server-side ChannelCreate — gateway will broadcast
                        // back a ChannelCreated event that updates the sidebar.
                        let net_clone = net.clone();
                        let ch_id = name.clone();
                        let ch_name = name.clone();
                        let ch_kind = kind.clone();
                        tokio::spawn(async move {
                            net_clone
                                .send(NetCommand::ChannelCreate {
                                    channel_id: ch_id,
                                    channel_name: ch_name,
                                    kind: ch_kind,
                                })
                                .await;
                        });
                        s.create_channel_open = false;
                    }
                });
            });
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                s.create_channel_open = false;
            }
        });
}
