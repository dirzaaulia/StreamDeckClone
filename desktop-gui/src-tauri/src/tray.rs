use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{App, AppHandle, Manager};

use crate::host_process::{self, HostProcess};

const OPEN: &str = "open";
const RESTART: &str = "restart";
const QUIT: &str = "quit";

pub fn show_main(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main")
        && let Err(error) = window
            .show()
            .and_then(|_| window.unminimize())
            .and_then(|_| window.set_focus())
    {
        eprintln!("Could not restore configurator: {error}");
    }
}

fn refresh_host_label(
    app: &AppHandle,
    label: &MenuItem<tauri::Wry>,
    restart: &MenuItem<tauri::Wry>,
) {
    let state = app.state::<HostProcess>();
    let (text, can_restart) = match host_process::current_status(&state) {
        Ok(status) if status.managed => ("Host: Running (managed)", true),
        Ok(status) if status.running => ("Host: Running (external)", false),
        Ok(_) => ("Host: Stopped", true),
        Err(error) => {
            eprintln!("Could not refresh tray host state: {error}");
            ("Host: Status unavailable", false)
        }
    };
    if let Err(error) = label
        .set_text(text)
        .and_then(|_| restart.set_enabled(can_restart))
    {
        eprintln!("Could not update tray menu: {error}");
    }
}

pub fn setup(app: &mut App) -> tauri::Result<()> {
    let handle = app.handle();
    let open = MenuItem::with_id(handle, OPEN, "Open Configurator", true, None::<&str>)?;
    let status = MenuItem::with_id(handle, "status", "Host: Stopped", false, None::<&str>)?;
    let restart = MenuItem::with_id(handle, RESTART, "Restart Host Engine", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(handle)?;
    let quit = MenuItem::with_id(handle, QUIT, "Quit StreamDeck", true, None::<&str>)?;
    let menu = Menu::with_items(handle, &[&open, &status, &restart, &separator, &quit])?;
    refresh_host_label(handle, &status, &restart);

    let status_for_menu = status.clone();
    let restart_for_menu = restart.clone();
    let status_for_tray = status.clone();
    let restart_for_tray = restart.clone();
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or(tauri::Error::WindowNotFound)?;
    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            OPEN => show_main(app),
            RESTART => {
                if let Err(error) = host_process::restart_managed_host(app) {
                    eprintln!("Could not restart host: {error}");
                    show_main(app);
                }
                refresh_host_label(app, &status_for_menu, &restart_for_menu);
            }
            QUIT => {
                host_process::stop_managed_host(app.state::<HostProcess>().inner());
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(move |tray, event| {
            if let TrayIconEvent::Click {
                button,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                if button == MouseButton::Left {
                    show_main(tray.app_handle());
                } else if button == MouseButton::Right {
                    refresh_host_label(tray.app_handle(), &status_for_tray, &restart_for_tray);
                }
            }
        })
        .build(handle)?;

    let running = Arc::new(AtomicBool::new(true));
    app.manage(TrayPoller(running.clone()));
    let app_handle = handle.clone();
    std::thread::spawn(move || {
        while running.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_secs(3));
            if !running.load(Ordering::Relaxed) {
                break;
            }
            let status = status.clone();
            let restart = restart.clone();
            let app = app_handle.clone();
            if let Err(error) =
                app_handle.run_on_main_thread(move || refresh_host_label(&app, &status, &restart))
            {
                eprintln!("Could not schedule tray refresh: {error}");
                break;
            }
        }
    });
    Ok(())
}

pub struct TrayPoller(Arc<AtomicBool>);

impl TrayPoller {
    pub fn stop(&self) {
        self.0.store(false, Ordering::Relaxed);
    }
}
