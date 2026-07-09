mod ui;

pub use ui::VnoxApp;
pub use ui::{chat, connect, keybind, members, settings, sidebar, state, theme, titlebar};

use anyhow::Result;
use tracing::info;
use tracing_subscriber::EnvFilter;
use vnox_client::{identity, net};

fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("info,wgpu=warn,wgpu_hal=warn,egui_wgpu=warn,naga=warn")
    });
    tracing_subscriber::fmt().with_env_filter(filter).init();

    let identity = identity::load_or_generate()?;
    info!("identity: {}", identity.short_id());

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _guard = rt.enter();

    let net = net::spawn(identity.clone());

    eframe::run_native(
        "Vnox",
        eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title("Vnox")
                .with_inner_size([900.0, 560.0])
                .with_min_inner_size([600.0, 400.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(ui::VnoxApp::new(cc, identity, net)))),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))
}
