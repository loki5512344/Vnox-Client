use eframe::egui::{self, FontId, Frame, Rect, RichText};

use crate::state::UiState;
use crate::theme as t;
use crate::theme::icons;

pub(super) fn member_card(
    ui: &mut egui::Ui,
    nick: &str,
    user_id: &str,
    volumes: &mut std::collections::HashMap<String, f32>,
    speaking: bool,
) {
    let vol = volumes.entry(user_id.to_string()).or_insert(80.0);
    let avatar_color = if speaking {
        t::SUCCESS
    } else {
        nick_color(nick)
    };
    Frame::NONE
        .fill(if speaking {
            egui::Color32::from_rgba_premultiplied(
                t::SUCCESS.r(),
                t::SUCCESS.g(),
                t::SUCCESS.b(),
                15,
            )
        } else {
            t::BG_ELEVATED
        })
        .corner_radius(t::R_MD)
        .stroke(if speaking {
            egui::Stroke::new(1.0, t::SUCCESS)
        } else {
            egui::Stroke::new(1.0, t::BORDER_DEFAULT)
        })
        .inner_margin(egui::Margin::symmetric(12, 8))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let init = nick
                    .chars()
                    .next()
                    .unwrap_or('?')
                    .to_uppercase()
                    .to_string();
                t::avatar(ui, &init, avatar_color, 28.0);
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(nick)
                                .font(FontId::proportional(13.0))
                                .color(if speaking {
                                    t::SUCCESS
                                } else {
                                    t::TEXT_PRIMARY
                                })
                                .strong(),
                        );
                        if speaking {
                            let (sr, _) = ui
                                .allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                            icons::speaker(ui.painter(), sr, t::SUCCESS);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("vol")
                                .font(FontId::proportional(9.0))
                                .color(t::TEXT_DISABLED),
                        );
                        ui.add(
                            egui::Slider::new(vol, 0.0..=100.0)
                                .show_value(false)
                                .smallest_positive(0.0),
                        );
                    });
                });
            });
        });
}

pub(super) fn voice_controls(ui: &mut egui::Ui, s: &mut UiState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        // Microphone button — drawn icon.
        let mic_color = if s.mic_enabled {
            t::TEXT_SECONDARY
        } else {
            t::ERROR
        };
        let mic_fill = if s.mic_enabled {
            t::BG_ELEVATED
        } else {
            egui::Color32::from_rgba_premultiplied(217, 96, 74, 20)
        };
        let mic_stroke = if s.mic_enabled {
            egui::Stroke::new(1.0, t::BORDER_DEFAULT)
        } else {
            egui::Stroke::new(1.0, egui::Color32::from_rgba_premultiplied(217, 96, 74, 80))
        };
        let (mic_rect, mic_resp) =
            ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
        if ui.is_rect_visible(mic_rect) {
            let painter = ui.painter();
            painter.rect_filled(mic_rect, t::R_FULL, mic_fill);
            painter.rect_stroke(mic_rect, t::R_FULL, mic_stroke, egui::StrokeKind::Middle);
            let mic_icon = if s.mic_enabled {
                icons::microphone
            } else {
                icons::microphone_off
            };
            let inner = inset(mic_rect, 10.0);
            mic_icon(painter, inner, mic_color);
        }
        if mic_resp.clicked() {
            s.mic_enabled = !s.mic_enabled;
        }
        mic_resp.on_hover_text("microphone");

        // Deafen button — drawn icon.
        let deaf_color = if s.deafen {
            t::ERROR
        } else {
            t::TEXT_SECONDARY
        };
        let deaf_fill = if s.deafen {
            egui::Color32::from_rgba_premultiplied(217, 96, 74, 20)
        } else {
            t::BG_ELEVATED
        };
        let (deaf_rect, deaf_resp) =
            ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
        if ui.is_rect_visible(deaf_rect) {
            let painter = ui.painter();
            painter.rect_filled(deaf_rect, t::R_FULL, deaf_fill);
            painter.rect_stroke(
                deaf_rect,
                t::R_FULL,
                egui::Stroke::new(1.0, t::BORDER_DEFAULT),
                egui::StrokeKind::Middle,
            );
            let deaf_icon = if s.deafen {
                icons::headphones_off
            } else {
                icons::headphones
            };
            let inner = inset(deaf_rect, 10.0);
            deaf_icon(painter, inner, deaf_color);
        }
        if deaf_resp.clicked() {
            s.deafen = !s.deafen;
        }
        deaf_resp.on_hover_text("deafen");

        // Leave button — drawn X on red background.
        let (leave_rect, leave_resp) =
            ui.allocate_exact_size(egui::vec2(44.0, 44.0), egui::Sense::click());
        if ui.is_rect_visible(leave_rect) {
            let painter = ui.painter();
            painter.rect_filled(leave_rect, t::R_FULL, t::ERROR);
            let inner = inset(leave_rect, 12.0);
            icons::close(painter, inner, egui::Color32::from_rgb(26, 16, 8));
        }
        if leave_resp.clicked()
            && let Some(id) = s.active_channel.clone()
        {
            if s.channels.iter().any(|c| c.id == id && c.kind == "text") {
                s.active_channel = Some("general".into());
            } else {
                s.active_channel = s
                    .channels
                    .iter()
                    .find(|c| c.kind == "text")
                    .map(|c| c.id.clone());
            }
        }
        leave_resp.on_hover_text("leave voice channel");
    });
}

fn inset(rect: Rect, pad: f32) -> Rect {
    Rect::from_min_max(
        (rect.min.x + pad, rect.min.y + pad).into(),
        (rect.max.x - pad, rect.max.y - pad).into(),
    )
}

fn nick_color(nick: &str) -> egui::Color32 {
    let colors = [
        t::ACCENT,
        t::SUCCESS,
        t::WARNING,
        t::INFO,
        egui::Color32::from_rgb(200, 100, 180),
    ];
    colors[nick.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len()]
}
