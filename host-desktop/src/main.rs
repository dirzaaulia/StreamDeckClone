// [LINE BUDGET AUDIT] 38/250
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod audio;
mod config;
mod input;
mod layout;
mod mdns;
mod pairing;
mod peer_throttle;
mod profiles;
mod protocol;
mod server;
mod storage;
mod tls;

use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "host_desktop=info,tokio=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("===============================================");
    info!("   StreamDeck Clone: Windows Host Engine v0.1  ");
    info!("   Transport: WSS (Protobuf) on :4455          ");
    info!("   Control: WS (JSON) on :4456                 ");
    info!("   Automation: Win32 Input Native Driver       ");
    info!("===============================================");

    mdns::spawn_mdns_broadcaster(4455);
    if let Some(ip) = mdns::get_local_ip() {
        info!("   Local Wi-Fi IP: wss://{}:4455", ip);
    }

    let server = server::DeckServer::new()?;
    server.run().await?;

    Ok(())
}
