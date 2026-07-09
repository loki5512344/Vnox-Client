use eframe::egui::{self, Frame};

use crate::connect;
use crate::state::ConnState;
use crate::{chat, members, settings, sidebar, theme, titlebar};

use super::VnoxApp;

impl eframe::App for VnoxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_voice_keybinds(ctx);
        self.sync_audio_controls(ctx);
        self.poll_net();

        if matches!(
            self.ui.conn,
            ConnState::Connecting | ConnState::Reconnecting { .. }
        ) {
            ctx.request_repaint_after(std::time::Duration::from_millis(50));
        }
        if matches!(self.ui.conn, ConnState::Connected { .. }) {
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }
        if self.audio.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(20));
        }

        titlebar::show_titlebar(ctx, &mut self.ui);

        if matches!(self.ui.conn, ConnState::Connected { .. }) {
            sidebar::guild_bar(ctx, &mut self.ui, &self.net);
        }
        sidebar::show(ctx, &mut self.ui, &self.identity, &self.net);
        if matches!(self.ui.conn, ConnState::Connected { .. }) {
            members::show(ctx, &mut self.ui);
        }
        egui::CentralPanel::default()
            .frame(Frame::NONE.fill(theme::BG_BASE))
            .show(ctx, |ui| match &self.ui.conn.clone() {
                ConnState::Disconnected
                | ConnState::Connecting
                | ConnState::Reconnecting { .. } => {
                    connect::show(ui, &mut self.ui, &self.identity, &self.net)
                }
                ConnState::Connected { .. } => {
                    chat::show(ui, ctx, &mut self.ui, &self.identity, &self.net)
                }
            });
        settings::show_window(ctx, &mut self.ui, &self.identity);
    }
}
