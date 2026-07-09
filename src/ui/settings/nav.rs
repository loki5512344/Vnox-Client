use crate::state::{SettingsPage, UiState};
use crate::theme as t;
use eframe::egui::{self, Align, Context, FontId, Frame, Layout, ScrollArea, Vec2};

use super::widgets;

pub(super) fn nav_panel(ui: &mut egui::Ui, s: &mut UiState) {
    Frame::NONE
        .fill(t::BG_SURFACE)
        .inner_margin(egui::Margin::symmetric(8, 12))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ScrollArea::vertical()
                    .id_salt("settings_nav")
                    .max_height((ui.available_height() - 30.0).max(80.0))
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        nav_group(ui, s, "account", &[(SettingsPage::Identity, "identity")]);
                        nav_group(
                            ui,
                            s,
                            "audio",
                            &[
                                (SettingsPage::Voice, "voice"),
                                (SettingsPage::AudioOutput, "audio output"),
                            ],
                        );
                        nav_group(
                            ui,
                            s,
                            "network",
                            &[
                                (SettingsPage::Network, "network"),
                                (SettingsPage::Overlay, "overlay"),
                            ],
                        );
                        nav_group(
                            ui,
                            s,
                            "app",
                            &[
                                (SettingsPage::Appearance, "appearance"),
                                (SettingsPage::Keybinds, "keybinds"),
                                (SettingsPage::Plugins, "plugins"),
                            ],
                        );
                        nav_group(ui, s, "debug", &[(SettingsPage::Advanced, "advanced")]);
                    });
                if back_btn(ui) {
                    s.settings_open = false;
                }
            });
        });
}

pub fn show_nav(ctx: &Context, s: &mut UiState) {
    egui::SidePanel::left("settings_nav")
        .exact_width(160.0)
        .resizable(false)
        .frame(Frame::new().fill(t::BG_STRIP))
        .show(ctx, |ui| {
            ui.style_mut().spacing.item_spacing = Vec2::new(0.0, 1.0);
            ui.add_space(10.0);
            nav_group(ui, s, "account", &[(SettingsPage::Identity, "identity")]);
            nav_group(
                ui,
                s,
                "audio",
                &[
                    (SettingsPage::Voice, "voice"),
                    (SettingsPage::AudioOutput, "audio output"),
                ],
            );
            nav_group(
                ui,
                s,
                "network",
                &[
                    (SettingsPage::Network, "network"),
                    (SettingsPage::Overlay, "overlay"),
                ],
            );
            nav_group(
                ui,
                s,
                "app",
                &[
                    (SettingsPage::Appearance, "appearance"),
                    (SettingsPage::Keybinds, "keybinds"),
                    (SettingsPage::Plugins, "plugins"),
                ],
            );
            nav_group(ui, s, "debug", &[(SettingsPage::Advanced, "advanced")]);
            ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                ui.add_space(8.0);
                if back_btn(ui) {
                    s.settings_open = false;
                }
            });
        });
}

fn nav_group(ui: &mut egui::Ui, s: &mut UiState, section: &str, items: &[(SettingsPage, &str)]) {
    ui.add_space(4.0);
    widgets::nav_section(ui, section);
    for (page, label) in items {
        if widgets::nav_item(ui, &s.settings_page == page, label) {
            s.settings_page = page.clone();
        }
    }
}

fn back_btn(ui: &mut egui::Ui) -> bool {
    ui.add_sized(
        [ui.available_width().max(120.0), 28.0],
        egui::Button::new(
            egui::RichText::new("← back")
                .font(FontId::monospace(11.0))
                .color(t::TEXT_GHOST),
        )
        .fill(egui::Color32::TRANSPARENT)
        .frame(false),
    )
    .clicked()
}
