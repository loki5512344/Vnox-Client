use eframe::egui::Color32;

use crate::state::{ConnState, UiState};
use crate::theme as t;
use vnox_client::net::{NetCommand, NetHandle};

pub fn disconnect(s: &mut UiState, net: &NetHandle) {
    let net = net.clone();
    tokio::spawn(async move {
        net.send(NetCommand::Disconnect).await;
    });
    s.conn = ConnState::Disconnected;
}

pub fn connect_to(s: &mut UiState, net: &NetHandle, addr: &str) {
    if addr.is_empty() {
        return;
    }
    if let Some(i) = s.bookmarks.iter().position(|b| b.address == addr) {
        s.selected_bookmark = i;
    }
    s.connect_input = addr.to_string();
    s.conn = ConnState::Connecting;
    let net = net.clone();
    let address = addr.to_string();
    let auto_reconnect = s.auto_reconnect;
    tokio::spawn(async move {
        net.send(NetCommand::Connect {
            address,
            auto_reconnect,
        })
        .await;
    });
}

pub(crate) fn msg_color(msg: &str) -> Color32 {
    let l = msg.to_ascii_lowercase();
    if l.contains("error") || l.contains("fail") || l.contains("refuse") {
        t::DANGER
    } else if l.contains("connected") {
        t::SUCCESS
    } else {
        t::TEXT_MUTED
    }
}
