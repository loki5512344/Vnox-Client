pub mod crypto;
pub mod framing;
pub mod handshake;
mod internal;
pub mod payloads;
mod session;
mod types;
pub mod voice;

pub use types::{ChatMsg, FriendInfo, GuildInfo, MemberInfo, NetCommand, NetEvent, PresenceInfo};

use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::identity::Identity;

// ─── Handle ──────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct NetHandle {
    cmd_tx: mpsc::Sender<NetCommand>,
    event_rx: Arc<Mutex<mpsc::Receiver<NetEvent>>>,
}

impl NetHandle {
    pub async fn send(&self, cmd: NetCommand) {
        let _ = self.cmd_tx.send(cmd).await;
    }
    pub fn try_recv(&self) -> Option<NetEvent> {
        self.event_rx.try_lock().ok()?.try_recv().ok()
    }
}

pub fn spawn(identity: Identity) -> NetHandle {
    let (cmd_tx, cmd_rx) = mpsc::channel::<NetCommand>(64);
    let (event_tx, event_rx) = mpsc::channel::<NetEvent>(256);
    tokio::spawn(session::net_task(identity, cmd_rx, event_tx));
    NetHandle {
        cmd_tx,
        event_rx: Arc::new(Mutex::new(event_rx)),
    }
}
