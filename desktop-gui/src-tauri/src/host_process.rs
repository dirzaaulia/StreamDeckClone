// [LINE BUDGET AUDIT] 130/250
use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

#[derive(Default)]
pub struct HostProcess(Mutex<Option<Child>>);

#[derive(serde::Serialize)]
pub struct HostStatus {
    pub running: bool,
    pub managed: bool,
}

fn external_host_pid() -> Option<u32> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let output = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq host-desktop.exe", "/NH", "/FO", "CSV"])
            .creation_flags(0x08000000)
            .output()
            .ok()?;
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter(|line| line.starts_with('"'))
            .find_map(|line| line.split(',').nth(1)?.trim_matches('"').parse().ok())
    }
    #[cfg(not(windows))]
    {
        None
    }
}

pub fn current_status(process: &HostProcess) -> Result<HostStatus, String> {
    let mut child = process.0.lock().map_err(|e| e.to_string())?;
    if let Some(running) = child.as_mut() {
        if running.try_wait().map_err(|e| e.to_string())?.is_none() {
            return Ok(HostStatus {
                running: true,
                managed: true,
            });
        }
        *child = None;
    }
    Ok(HostStatus {
        running: external_host_pid().is_some(),
        managed: false,
    })
}

fn find_host_exe(app: &AppHandle) -> Option<std::path::PathBuf> {
    let resource_dir = app.path().resource_dir().ok();
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));
    let dev_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()?
        .parent()?
        .join("host-desktop")
        .join("target");
    [
        resource_dir.map(|dir| dir.join("host-desktop.exe")),
        exe_dir.map(|dir| dir.join("host-desktop.exe")),
        Some(dev_dir.join("release").join("host-desktop.exe")),
        Some(dev_dir.join("debug").join("host-desktop.exe")),
    ]
    .into_iter()
    .flatten()
    .find(|path| path.is_file())
}

pub fn ensure_host_running(app: &AppHandle) -> Result<HostStatus, String> {
    let process = app.state::<HostProcess>();
    let current = current_status(&process)?;
    if current.running {
        return Ok(current);
    }
    launch_host(app.clone(), process)
}

#[tauri::command]
pub fn host_status(process: State<'_, HostProcess>) -> Result<HostStatus, String> {
    current_status(&process)
}

#[tauri::command]
pub fn launch_host(app: AppHandle, process: State<'_, HostProcess>) -> Result<HostStatus, String> {
    let current = current_status(&process)?;
    if current.running {
        return Err("Host is already running. Stop it before starting another.".into());
    }
    let exe = find_host_exe(&app).ok_or("Build the Windows host before starting it.")?;
    let mut command = Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("Could not start host: {e}"))?;
    std::thread::sleep(std::time::Duration::from_millis(300));
    if let Some(exit) = child.try_wait().map_err(|e| e.to_string())? {
        return Err(format!(
            "Host exited during startup ({exit}). Check host logs or start it from a development terminal."
        ));
    }
    *process.0.lock().map_err(|e| e.to_string())? = Some(child);
    current_status(&process)
}

#[tauri::command]
pub fn stop_host(process: State<'_, HostProcess>) -> Result<HostStatus, String> {
    let mut child = process.0.lock().map_err(|e| e.to_string())?;
    if let Some(running) = child.as_mut() {
        if running.try_wait().map_err(|e| e.to_string())?.is_none() {
            running
                .kill()
                .map_err(|e| format!("Could not stop host: {e}"))?;
            running
                .wait()
                .map_err(|e| format!("Could not wait for host: {e}"))?;
        }
        *child = None;
    } else if external_host_pid().is_some() {
        return Err("This host was started outside the desktop app. Close its window or stop it as Administrator, then start it here.".into());
    }
    drop(child);
    current_status(&process)
}

pub fn restart_managed_host(app: &AppHandle) -> Result<HostStatus, String> {
    let process = app.state::<HostProcess>();
    let current = current_status(&process)?;
    if current.running && !current.managed {
        return Err("Host was started outside this app; restart it from its owner.".into());
    }
    if current.managed {
        stop_host(process.clone())?;
    }
    launch_host(app.clone(), process)
}

pub fn stop_managed_host(process: &HostProcess) {
    if let Ok(mut child) = process.0.lock()
        && let Some(mut running) = child.take()
    {
        let _ = running.kill();
        let _ = running.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_process_default_unmanaged_status() {
        let process = HostProcess::default();
        let current = current_status(&process).unwrap();
        assert!(!current.managed);
    }
}
