use super::{muted_label, pending, section_header, status_color, status_label};
use crate::state::{FriendState, UiState};
use crate::theme as t;
use eframe::egui::{self, FontId, Frame, RichText, ScrollArea};
use vnox_client::net::{NetCommand, NetHandle};

pub(super) fn render_content(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    ScrollArea::vertical()
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.add_space(4.0);

            match s.friends_tab.as_str() {
                "online" => render_online(ui, s, net),
                "pending" => render_pending(ui, s, net),
                "blocked" => render_blocked(ui, s, net),
                _ => render_all(ui, s, net),
            }
        });
}

fn render_online(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    section_header(ui, "ONLINE");
    let online: Vec<&FriendState> = s
        .friends
        .iter()
        .filter(|f| matches!(f.status.as_str(), "online" | "idle" | "dnd"))
        .collect();
    if online.is_empty() {
        muted_label(ui, "no friends online");
    } else {
        for f in &online {
            friend_row(ui, f, net);
        }
    }
}

fn render_all(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    section_header(ui, "ONLINE");
    let online: Vec<&FriendState> = s
        .friends
        .iter()
        .filter(|f| matches!(f.status.as_str(), "online" | "idle" | "dnd"))
        .collect();
    if online.is_empty() {
        muted_label(ui, "no friends online");
    } else {
        for f in &online {
            friend_row(ui, f, net);
        }
    }

    ui.add_space(16.0);

    section_header(ui, "OFFLINE");
    let offline: Vec<&FriendState> = s
        .friends
        .iter()
        .filter(|f| !matches!(f.status.as_str(), "online" | "idle" | "dnd"))
        .collect();
    if offline.is_empty() {
        if s.friends.is_empty() {
            muted_label(ui, "no friends yet — add some!");
        } else {
            muted_label(ui, "nobody is offline");
        }
    } else {
        for f in &offline {
            friend_row(ui, f, net);
        }
    }
}

fn render_pending(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    section_header(ui, "PENDING — INCOMING");
    if s.pending_friend_requests.is_empty() {
        muted_label(ui, "no pending requests");
    } else {
        for req in &s.pending_friend_requests {
            pending::pending_row(ui, req, net);
        }
    }
}

fn render_blocked(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    // Block-user input row
    Frame::NONE
        .inner_margin(egui::Margin::symmetric(12, 4))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut buf = s.block_input.clone().unwrap_or_default();
                let resp = ui.add(
                    egui::TextEdit::singleline(&mut buf)
                        .font(egui::FontId::monospace(11.0))
                        .hint_text("user id to block")
                        .desired_width(140.0)
                        .text_color(t::TEXT_PRIMARY),
                );
                let mut trigger = false;
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    trigger = true;
                }
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Block")
                                .font(FontId::proportional(10.0))
                                .color(t::ERROR),
                        )
                        .fill(t::BG_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                        .corner_radius(t::R_SM)
                        .min_size(egui::vec2(50.0, 18.0)),
                    )
                    .clicked()
                {
                    trigger = true;
                }
                s.block_input = if buf.is_empty() { None } else { Some(buf) };
                if trigger
                    && let Some(target) = s.block_input.clone().filter(|s| !s.trim().is_empty())
                {
                    let net = net.clone();
                    let target = target.trim().to_string();
                    tokio::spawn(async move {
                        net.send(NetCommand::BlockUser { user_id: target }).await;
                    });
                    s.block_input = None;
                }
            });
        });
    ui.add_space(8.0);

    section_header(ui, "BLOCKED");
    if s.blocked_users.is_empty() {
        muted_label(ui, "no blocked users — enter a user ID above to block");
        return;
    }

    // Fetch on first open of this tab.
    if s.block_list_fetched_at.is_none() {
        s.block_list_fetched_at = Some(std::time::Instant::now());
        let net = net.clone();
        tokio::spawn(async move {
            net.send(NetCommand::BlockList).await;
        });
    }

    for uid in s.blocked_users.clone() {
        Frame::NONE
            .fill(t::BG_SURFACE)
            .corner_radius(t::R_MD)
            .inner_margin(egui::Margin::symmetric(12, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let init: String = uid.chars().next().unwrap_or('?').to_uppercase().to_string();
                    t::avatar(ui, &init, t::TEXT_DISABLED, 28.0);
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(&uid)
                                .font(FontId::monospace(11.0))
                                .color(t::TEXT_PRIMARY),
                        );
                        ui.label(
                            RichText::new("blocked")
                                .font(FontId::proportional(10.0))
                                .color(t::TEXT_DISABLED),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("Unblock")
                                        .font(FontId::proportional(11.0))
                                        .color(t::SUCCESS),
                                )
                                .fill(egui::Color32::TRANSPARENT)
                                .frame(false),
                            )
                            .on_hover_text("unblock this user")
                            .clicked()
                        {
                            let net = net.clone();
                            let uidc = uid.clone();
                            tokio::spawn(async move {
                                net.send(NetCommand::UnblockUser { user_id: uidc }).await;
                            });
                        }
                    });
                });
            });
    }
}

fn friend_row(ui: &mut egui::Ui, f: &FriendState, net: &NetHandle) {
    Frame::NONE
        .fill(t::BG_SURFACE)
        .corner_radius(t::R_MD)
        .inner_margin(egui::Margin::symmetric(12, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let init = f
                    .nickname
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                t::avatar(ui, &init, status_color(&f.status), 28.0);

                ui.add_space(8.0);
                let (dot_r, _) =
                    ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter()
                    .circle_filled(dot_r.center(), 4.0, status_color(&f.status));

                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&f.nickname)
                            .font(FontId::proportional(13.0))
                            .color(t::TEXT_PRIMARY),
                    );
                    ui.label(
                        RichText::new(status_label(&f.status))
                            .font(FontId::proportional(10.0))
                            .color(t::TEXT_DISABLED),
                    );
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if t::icon_btn_drawn(
                        ui,
                        crate::theme::icons::envelope,
                        "send message",
                        None,
                        18.0,
                    )
                    .clicked()
                    {
                        let net = net.clone();
                        let uid = f.user_id.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::DmStart {
                                target_user_id: uid,
                            })
                            .await;
                        });
                    }
                    if t::icon_btn_drawn(
                        ui,
                        crate::theme::icons::close,
                        "remove friend",
                        None,
                        16.0,
                    )
                    .clicked()
                    {
                        let net = net.clone();
                        let uid = f.user_id.clone();
                        tokio::spawn(async move {
                            net.send(NetCommand::FriendRemove { user_id: uid }).await;
                        });
                    }
                });
            });
        });
}
