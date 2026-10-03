// [LINE BUDGET AUDIT] 145/250
use tauri::Manager;

fn find_host_exe(app: &tauri::AppHandle) -> Option<std::path::PathBuf> {
    if let Ok(dir) = app.path().resource_dir() {
        let p = dir.join("host-desktop.exe");
        if p.exists() {
            return Some(p);
        }
    }
    if let Ok(current_exe) = std::env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        let p = parent.join("host-desktop.exe");
        if p.exists() {
            return Some(p);
        }
    }
    let release_path = std::path::PathBuf::from(
        r"d:\Android\Projects\StreamDeckClone\host-desktop\target\release\host-desktop.exe",
    );
    if release_path.exists() {
        return Some(release_path);
    }
    let dev_path = std::path::PathBuf::from(
        r"d:\Android\Projects\StreamDeckClone\host-desktop\target\debug\host-desktop.exe",
    );
    if dev_path.exists() {
        return Some(dev_path);
    }
    None
}

/// Return the running host-desktop PID if found, else 0.
#[tauri::command]
fn host_pid() -> u32 {
    #[cfg(windows)]
    {
        use std::process::Command;
        let s = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq host-desktop.exe", "/NH", "/FO", "CSV"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
            .unwrap_or_default();
        s.lines()
            .next()
            .and_then(|l| l.split(',').nth(1))
            .and_then(|p| p.trim_matches('"').parse().ok())
            .unwrap_or(0)
    }
    #[cfg(not(windows))]
    0
}

/// Launch the host-desktop engine in the background silently.
#[tauri::command]
fn launch_host(app: tauri::AppHandle) -> Result<String, String> {
    if host_pid() > 0 {
        return Ok("Host engine already running".to_string());
    }
    let exe = find_host_exe(&app).ok_or_else(|| "host-desktop.exe not found".to_string())?;
    let mut cmd = std::process::Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    cmd.spawn()
        .map(|_| "Host engine started successfully".to_string())
        .map_err(|e| format!("Failed to launch host: {e}"))
}

#[tauri::command]
fn get_local_ip() -> String {
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
    NetworkInfo {
        wifi_name: get_wifi_ssid(),
        ip: get_local_ip(),
        port: 4455,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if host_pid() == 0 {
                let handle = app.handle().clone();
                let _ = launch_host(handle);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            launch_host,
            host_pid,
            get_local_ip,
            get_network_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

