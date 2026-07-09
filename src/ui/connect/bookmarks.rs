mod add;

use eframe::egui::{self, Frame};
use vnox_client::net::NetHandle;

use crate::state::{ConnState, UiState};
use crate::theme as t;

use super::action::connect_to;

pub fn server_list(ui: &mut egui::Ui, s: &mut UiState, net: &NetHandle) {
    Frame::NONE
        .fill(t::BG_ELEVATED)
        .corner_radius(t::R_LG)
        .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("SERVERS")
                        .font(egui::FontId::proportional(11.0))
                        .color(t::TEXT_MUTED),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let add_btn = egui::Button::new(
                        egui::RichText::new("+")
                            .font(egui::FontId::proportional(16.0))
                            .color(t::ACCENT_SOFT),
                    )
                    .fill(t::ACCENT_10)
                    .stroke(egui::Stroke::new(1.0, t::BORDER_ACCENT))
                    .corner_radius(t::R_SM)
                    .min_size(egui::vec2(28.0, 28.0));
                    if ui.add(add_btn).on_hover_text("add server").clicked() {
                        s.add_server_open = !s.add_server_open;
                        if s.add_server_open {
                            s.add_server_input = String::new();
                        }
                    }
                });
            });
            if s.add_server_open {
                add::add_form(ui, s);
            }
            ui.add_space(4.0);
            let busy = matches!(
                s.conn,
                ConnState::Connecting | ConnState::Reconnecting { .. }
            );

            if s.bookmarks.is_empty() {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("no servers yet — click + to add one")
                            .font(egui::FontId::proportional(12.0))
                            .color(t::TEXT_DISABLED),
                    );
                });
                ui.add_space(12.0);
            } else {
                for (i, bm) in s.bookmarks.clone().into_iter().enumerate() {
                    let active = s.connect_input == bm.address;
                    let bg = if active {
                        t::ACCENT_10
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    let stroke = if active {
                        egui::Stroke::new(1.0, t::BORDER_ACCENT)
                    } else {
                        egui::Stroke::NONE
                    };

                    let resp = Frame::NONE
                        .fill(bg)
                        .stroke(stroke)
                        .corner_radius(t::R_SM)
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("🖥").font(egui::FontId::proportional(13.0)),
                                );
                                ui.add_space(6.0);
                                ui.vertical(|ui| {
                                    ui.label(
                                        egui::RichText::new(&bm.label)
                                            .font(egui::FontId::proportional(13.0))
                                            .color(t::TEXT_PRIMARY),
                                    );
                                    ui.label(
                                        egui::RichText::new(&bm.address)
                                            .font(egui::FontId::monospace(10.0))
                                            .color(t::TEXT_MUTED),
                                    );
                                });
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if t::icon_btn_drawn(
                                            ui,
                                            crate::theme::icons::close,
                                            "remove bookmark",
                                            None,
                                            16.0,
                                        )
                                        .clicked()
                                        {
                                            s.remove_bookmark(i);
                                        }
                                    },
                                );
                            });
                        })
                        .response
                        .interact(egui::Sense::click());

                    if resp.clicked() && !busy {
                        s.connect_input = bm.address.clone();
                        s.selected_bookmark = i;
                        if matches!(s.conn, ConnState::Disconnected) {
                            connect_to(s, net, &bm.address);
                        }
                    }
                }
            }
        });
}
