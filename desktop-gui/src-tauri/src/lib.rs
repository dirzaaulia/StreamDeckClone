// [LINE BUDGET AUDIT] 134/250
use tauri::Manager;

mod host_process;
mod control;
mod tray;

#[tauri::command]
fn get_local_ip() -> String {
    if let Ok(ip) = std::env::var("STREAMDECK_HOST_IP") {
        let trimmed = ip.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok();
    sock.and_then(|s| {
        s.connect("8.8.8.8:80").ok()?;
        s.local_addr().ok().map(|a| a.ip().to_string())
    })
    .unwrap_or_else(|| "127.0.0.1".to_string())
}

#[derive(serde::Serialize)]
pub struct NetworkInfo {
    pub wifi_name: Option<String>,
    pub ip: String,
    pub port: u16,
}

#[cfg(windows)]
fn get_wifi_ssid() -> Option<String> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    let mut cmd = Command::new("netsh");
    cmd.args(["wlan", "show", "interfaces"]);
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    let out = cmd.output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("SSID")
            && !trimmed.starts_with("BSSID")
            && let Some((_, val)) = trimmed.split_once(':')
        {
            let name = val.trim().to_string();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

#[cfg(not(windows))]
fn get_wifi_ssid() -> Option<String> {
    None
}

#[tauri::command]
fn get_network_info() -> NetworkInfo {
    let ip = get_local_ip();
    let wifi_name = if ip.starts_with("100.") {
        Some("Tailscale Network".to_string())
    } else {
        get_wifi_ssid()
    };
    NetworkInfo {
        wifi_name,
        ip,
        port: 4455,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();
    #[cfg(windows)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            tray::show_main(app)
        }));
    }
    builder
        .manage(host_process::HostProcess::default())
        .manage(control::ControlConnection::default())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            #[cfg(windows)]
            if let Some(window) = app.get_webview_window("main")
                && let Err(error) = window_vibrancy::apply_mica(&window, Some(true))
            {
                eprintln!("Mica unavailable; using solid theme: {error}");
            }
            tray::setup(app)?;
            if let Err(error) = host_process::ensure_host_running(app.handle()) {
                eprintln!("Could not start host on launch: {error}");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "main"
                && let tauri::WindowEvent::CloseRequested { api, .. } = event
            {
                api.prevent_close();
                host_process::stop_managed_host(
                    window
                        .app_handle()
                        .state::<host_process::HostProcess>()
                        .inner(),
                );
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            host_process::launch_host,
            host_process::stop_host,
            host_process::host_status,
            get_local_ip,
            get_network_info,
            control::host_fingerprint,
            control::control_request,
            control::control_disconnect
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application")
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<tray::TrayPoller>().stop();
                host_process::stop_managed_host(app.state::<host_process::HostProcess>().inner());
            }
        });
}
