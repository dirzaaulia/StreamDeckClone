// [LINE BUDGET AUDIT] 55/100
use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::collections::HashMap;
use std::net::{IpAddr, UdpSocket};
use tokio::task::JoinHandle;

pub fn get_local_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    Some(socket.local_addr().ok()?.ip())
}

pub fn spawn_mdns_broadcaster(port: u16) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mdns = match ServiceDaemon::new() {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to create mDNS daemon: {}", e);
                return;
            }
        };

        let hostname =
            std::env::var("COMPUTERNAME").unwrap_or_else(|_| "streamdeck-host".to_string());
        let host_name_fqdn = format!("{}.local.", hostname.to_lowercase());
        let instance_name = format!("StreamDeck-{}", hostname);
        let service_type = "_streamdeck._tcp.local.";
        let ip_str = get_local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let properties: HashMap<String, String> = HashMap::new();

        let service_info = match ServiceInfo::new(
            service_type,
            &instance_name,
            &host_name_fqdn,
            &ip_str,
            port,
            Some(properties),
        ) {
            Ok(s) => s,
            Err(e) => {
                tracing::error!("Failed to create mDNS service info: {}", e);
                return;
            }
        };

        if let Err(e) = mdns.register(service_info) {
            tracing::error!("Failed to register mDNS service: {}", e);
        } else {
            tracing::info!(
                "Started mDNS broadcaster on port {} (IP: {}, Host: {})",
                port,
                ip_str,
                instance_name
            );
        }

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
        }
    })
}
