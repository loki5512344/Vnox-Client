use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{error, info};

use crate::identity::Identity;
use crate::net::types::{NetCommand, NetEvent};

use super::{
    handlers::session_loop,
    reconnect,
    switch::{ReconnectRequested, SessionTarget},
};

pub async fn net_task(
    identity: Identity,
    mut cmd_rx: mpsc::Receiver<NetCommand>,
    event_tx: mpsc::Sender<NetEvent>,
) {
    let mut target: Option<SessionTarget> = None;
    let mut failures: u32 = 0;

    loop {
        let Some(mut session) = target.take() else {
            match wait_for_connect(&mut cmd_rx).await {
                Some(t) => {
                    failures = 0;
                    target = Some(t);
                }
                None => return,
            }
            continue;
        };

        if failures > 0 {
            let delay = reconnect::backoff_delay(failures - 1);
            let _ = event_tx
                .send(NetEvent::Reconnecting {
                    attempt: failures,
                    delay_secs: delay.as_secs(),
                })
                .await;
            sleep(delay).await;
        }

        info!(
            "connecting to {} (attempt {})",
            session.address,
            failures + 1
        );

        let restored = failures > 0;
        match session_loop(
            &identity,
            &session.address,
            &mut session.channels,
            restored,
            &mut cmd_rx,
            &event_tx,
        )
        .await
        {
            Ok(()) => {
                failures = 0;
                let _ = event_tx
                    .send(NetEvent::Disconnected {
                        reason: "disconnected".into(),
                        will_retry: false,
                    })
                    .await;
            }
            Err(e) => {
                if let Some(req) = e.downcast_ref::<ReconnectRequested>() {
                    info!("switching node to {}", req.address);
                    session.address = req.address.clone();
                    session.auto_reconnect = req.auto_reconnect;
                    failures = 0;
                    target = Some(session);
                    continue;
                }
                error!("session ended: {e}");
                let retry = session.auto_reconnect;
                let _ = event_tx
                    .send(NetEvent::Disconnected {
                        reason: e.to_string(),
                        will_retry: retry,
                    })
                    .await;
                if retry {
                    failures = failures.saturating_add(1);
                    target = Some(session);
                } else {
                    failures = 0;
                }
            }
        }
    }
}

async fn wait_for_connect(cmd_rx: &mut mpsc::Receiver<NetCommand>) -> Option<SessionTarget> {
    loop {
        match cmd_rx.recv().await {
            Some(NetCommand::Connect {
                address,
                auto_reconnect,
            }) => {
                return Some(SessionTarget {
                    address,
                    auto_reconnect,
                    channels: vec!["general".into()],
                });
            }
            Some(NetCommand::Disconnect) | None => return None,
            _ => {}
        }
    }
}
