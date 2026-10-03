// [LINE BUDGET AUDIT] 0/150
use std::collections::HashMap;
use tokio::task::JoinHandle;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub struct ProfileWatcher;

impl ProfileWatcher {
    pub fn spawn(tx: tokio::sync::mpsc::Sender<String>) -> JoinHandle<()> {
        tokio::task::spawn_blocking(move || {
            let mut map = HashMap::new();
            map.insert("chrome.exe".to_string(), "Browser".to_string());
            map.insert("code.exe".to_string(), "VSCode".to_string());
            map.insert("obs64.exe".to_string(), "OBS".to_string());
            map.insert("devenv.exe".to_string(), "VisualStudio".to_string());

            let mut last_profile = String::new();

            loop {
                let profile = Self::get_foreground_profile(&map).unwrap_or_else(|| "default".to_string());
                if profile != last_profile {
                    last_profile = profile.clone();
                    if tx.blocking_send(profile).is_err() {
                        break;
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        })
    }

    fn get_foreground_profile(map: &HashMap<String, String>) -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.0.is_null() {
                return None;
            }

            let mut pid = 0;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == 0 {
                return None;
            }

            let process = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
            
            let mut buf = [0u16; MAX_PATH as usize];
            let len = GetModuleFileNameExW(process, None, &mut buf);
            if len == 0 {
                return None;
            }

            let path = String::from_utf16_lossy(&buf[..len as usize]);
            let exe_name = path.split('\\').next_back()?.to_lowercase();
            
            map.get(&exe_name).cloned()
        }
    }
}
