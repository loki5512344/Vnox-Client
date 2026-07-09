mod list;
mod pending;

use eframe::egui::{self, FontId, Frame, RichText};
use vnox_client::net::{NetCommand, NetHandle};

use crate::state::UiState;
use crate::theme as t;

pub fn show(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
        Frame::NONE
            .fill(t::BG_SURFACE)
            .inner_margin(egui::Margin::symmetric(16, 0))
            .show(ui, |ui| {
                ui.set_height(44.0);
                ui.horizontal_centered(|ui| {
                    ui.label(
                        RichText::new("Friends")
                            .font(FontId::proportional(16.0))
                            .color(t::TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let add_btn = egui::Button::new(
                            RichText::new("Add Friend")
                                .font(FontId::proportional(11.0))
                                .color(t::ACCENT_SOFT),
                        )
                        .fill(t::ACCENT)
                        .stroke(egui::Stroke::NONE)
                        .corner_radius(t::R_SM)
                        .min_size(egui::vec2(80.0, 26.0));
                        if ui.add(add_btn).clicked() {
                            s.add_friend_open = true;
                            s.add_friend_input.clear();
                        }
                    });
                });
            });

        let sep = ui.max_rect();
        ui.painter().hline(
            sep.x_range(),
            sep.top() + 44.0,
            egui::Stroke::new(1.0, t::BORDER_SUBTLE),
        );

        ui.add_space(8.0);

        // Pending count badge for the Pending tab
        let pending_count = s.pending_friend_requests.len();

        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.spacing_mut().item_spacing.x = 4.0;
            for (tab_key, label) in [
                ("online", "Online"),
                ("all", "All"),
                ("pending", "Pending"),
                ("blocked", "Blocked"),
            ] {
                let selected = s.friends_tab == tab_key;
                let display = if tab_key == "pending" && pending_count > 0 {
                    format!("{label} ({pending_count})")
                } else {
                    label.to_string()
                };
                let btn = egui::Button::new(
                    RichText::new(display)
                        .font(FontId::proportional(11.0))
                        .color(if selected {
                            t::TEXT_PRIMARY
                        } else {
                            t::TEXT_MUTED
                        }),
                )
                .fill(if selected {
                    t::BG_ELEVATED
                } else {
                    egui::Color32::TRANSPARENT
                })
                .stroke(egui::Stroke::NONE)
                .corner_radius(t::R_SM);
                if ui.add(btn).clicked() {
                    s.friends_tab = tab_key.to_string();
                }
            }
        });

        ui.add_space(8.0);

        list::render_content(ui, s, net);
    });

    if s.add_friend_open {
        show_add_friend_popup(ui.ctx(), s, net);
    }
}

fn show_add_friend_popup(ctx: &egui::Context, s: &mut UiState, net: &NetHandle) {
    egui::Window::new("Add Friend")
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
            ui.set_width(320.0);
            ui.label(
                RichText::new("Enter a user ID to send a friend request.")
                    .font(FontId::proportional(12.0))
                    .color(t::TEXT_MUTED),
            );
            ui.add_space(8.0);
            let resp = ui.add(
                egui::TextEdit::singleline(&mut s.add_friend_input)
                    .font(FontId::monospace(13.0))
                    .text_color(t::TEXT_PRIMARY)
                    .hint_text(RichText::new("user id (pubkey hex)").color(t::TEXT_DISABLED))
                    .desired_width(280.0),
            );
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("Cancel")
                                .font(FontId::proportional(12.0))
                                .color(t::TEXT_MUTED),
                        )
                        .fill(egui::Color32::TRANSPARENT)
                        .frame(false),
                    )
                    .clicked()
                {
                    s.add_friend_open = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let send_btn = egui::Button::new(
                        RichText::new("Send Request")
                            .font(FontId::proportional(12.0))
                            .color(egui::Color32::WHITE),
                    )
                    .fill(t::ACCENT)
                    .stroke(egui::Stroke::NONE)
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(110.0, 28.0));
                    if ui.add(send_btn).clicked() && !s.add_friend_input.trim().is_empty() {
                        let net = net.clone();
                        let target = s.add_friend_input.trim().to_string();
                        tokio::spawn(async move {
                            net.send(NetCommand::FriendRequest {
                                target_user_id: target,
                            })
                            .await;
                        });
                        s.add_friend_open = false;
                    }
                });
            });
            if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                s.add_friend_open = false;
            }
        });
}

pub(super) fn section_header(ui: &mut egui::Ui, label: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(label)
            .font(FontId::proportional(10.0))
            .color(t::TEXT_MUTED)
            .strong(),
    );
    ui.add_space(2.0);
}

pub(super) fn muted_label(ui: &mut egui::Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(
        RichText::new(text)
            .font(FontId::proportional(12.0))
            .color(t::TEXT_DISABLED),
    );
}

pub(super) fn status_color(status: &str) -> egui::Color32 {
    match status {
        "online" => t::SUCCESS,
        "idle" => t::WARNING,
        "dnd" => t::ERROR,
        _ => t::TEXT_DISABLED,
    }
}

pub(super) fn status_label(status: &str) -> &str {
    match status {
        "online" => "online",
        "idle" => "idle",
        "dnd" => "do not disturb",
        "offline" => "offline",
        _ => status,
    }
}
