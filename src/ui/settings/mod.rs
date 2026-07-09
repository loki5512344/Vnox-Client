mod nav;
mod network;
mod tabs;
mod widgets;

use crate::state::{SettingsPage, UiState};
use crate::theme as t;
use eframe::egui::{self, Align, Context, Frame, Layout, ScrollArea, Vec2, Window};
use vnox_client::identity::Identity;

pub use nav::show_nav;

pub fn show_window(ctx: &Context, s: &mut UiState, identity: &Identity) {
    let open_id = egui::Id::new("settings_open_flag");
    if !s.settings_open {
        ctx.data_mut(|d| d.remove_temp::<bool>(open_id));
        return;
    }
    if ctx.data_mut(|d| d.get_temp::<bool>(open_id)) != Some(true) {
        ctx.data_mut(|d| d.insert_temp(open_id, true));
        s.force_refresh_audio_devices();
    }
    s.refresh_audio_devices();
    Window::new("Settings")
        .collapsible(false)
        .resizable(true)
        .default_size([720.0, 520.0])
        .min_size([520.0, 380.0])
        .frame(
            Frame::window(&ctx.style())
                .fill(t::BG_ELEVATED)
                .stroke(egui::Stroke::new(1.0, t::BORDER_DEFAULT))
                .corner_radius(t::R_LG)
                .shadow(egui::epaint::Shadow::NONE),
        )
        .show(ctx, |ui| {
            ui.visuals_mut().override_text_color = Some(t::TEXT_PRIMARY);
            let panel_h = ui.available_height().max(200.0);
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    Vec2::new(168.0, panel_h),
                    Layout::top_down(Align::LEFT),
                    |ui| nav::nav_panel(ui, s),
                );
                ui.separator();
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width().max(280.0), panel_h),
                    Layout::top_down(Align::LEFT),
                    |ui| {
                        ScrollArea::vertical()
                            .id_salt("settings_content")
                            .max_height(panel_h)
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.set_min_width(320.0);
                                ui.add_space(12.0);
                                show_content(ui, s, identity);
                                ui.add_space(12.0);
                            });
                    },
                );
            });
        });
}

pub fn show_content(ui: &mut egui::Ui, s: &mut UiState, id: &Identity) {
    match &s.settings_page.clone() {
        SettingsPage::Voice => tabs::voice(ui, s),
        SettingsPage::AudioOutput => tabs::output(ui, s),
        SettingsPage::Identity => tabs::show(ui, id, s),
        SettingsPage::Network => network::network(ui, s),
        SettingsPage::Overlay => network::overlay(ui, s),
        SettingsPage::Appearance => tabs::appearance(ui, s),
        SettingsPage::Keybinds => tabs::keybinds(ui, s),
        SettingsPage::Plugins => tabs::plugins(ui),
        SettingsPage::Advanced => tabs::advanced(ui, s),
    }
}
