// [LINE BUDGET AUDIT] 124/250
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, SendInput,
    VIRTUAL_KEY, VK_CONTROL, VK_D, VK_ESCAPE, VK_LWIN, VK_MEDIA_NEXT_TRACK, VK_MEDIA_PLAY_PAUSE,
    VK_MEDIA_PREV_TRACK, VK_SHIFT, VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP,
};

use crate::audio::AudioController;

pub struct InputExecutor {
    audio: Arc<Mutex<AudioController>>,
}

impl InputExecutor {
    pub fn new(audio: Arc<Mutex<AudioController>>) -> Self {
        Self { audio }
    }

    pub fn trigger_action(&self, action: &str) {
        match action {
            "vol_mute" => self.action_toggle_mute(),
            "vol_down" => self.action_volume_down(),
            "vol_up" => self.action_volume_up(),
            "media_play_pause" => {
                info!("Action: Media Play/Pause");
                self.press_single_key(VK_MEDIA_PLAY_PAUSE);
            }
            "media_prev" => {
                info!("Action: Media Previous");
                self.press_single_key(VK_MEDIA_PREV_TRACK);
            }
            "media_next" => {
                info!("Action: Media Next");
                self.press_single_key(VK_MEDIA_NEXT_TRACK);
            }
            "desktop" => {
                info!("Action: Show Desktop (Win + D)");
                self.press_hotkey(&[VK_LWIN, VK_D]);
            }
            "taskmgr" => {
                info!("Action: Task Manager (Ctrl + Shift + Esc)");
                self.press_hotkey(&[VK_CONTROL, VK_SHIFT, VK_ESCAPE]);
            }
            "screenshot" => {
                info!("Action: Take Screenshot (Win + Shift + S)");
                self.press_hotkey(&[VK_LWIN, VK_SHIFT, VIRTUAL_KEY(0x53)]);
            }
            "ctrl_c" => {
                info!("Action: Copy (Ctrl + C)");
                self.press_hotkey(&[VK_CONTROL, VIRTUAL_KEY(0x43)]);
            }
            "ctrl_v" => {
                info!("Action: Paste (Ctrl + V)");
                self.press_hotkey(&[VK_CONTROL, VIRTUAL_KEY(0x56)]);
            }
            "ctrl_z" => {
                info!("Action: Undo (Ctrl + Z)");
                self.press_hotkey(&[VK_CONTROL, VIRTUAL_KEY(0x5A)]);
            }
            "ctrl_s" => {
                info!("Action: Save (Ctrl + S)");
                self.press_hotkey(&[VK_CONTROL, VIRTUAL_KEY(0x53)]);
            }
            "ctrl_w" => {
                info!("Action: Close Tab (Ctrl + W)");
                self.press_hotkey(&[VK_CONTROL, VIRTUAL_KEY(0x57)]);
            }
            "f5" => {
                info!("Action: Refresh (F5)");
                self.press_single_key(VIRTUAL_KEY(0x74));
            }
            _ => {
                info!("Unhandled action: {}", action);
            }
        }
    }

    fn action_toggle_mute(&self) {
        info!("Slot 0: Mute/Unmute Audio (WASAPI + Key)");
        if let Ok(audio) = self.audio.lock() {
            audio.toggle_mute();
        }
        self.press_single_key(VK_VOLUME_MUTE);
    }

    fn action_volume_down(&self) {
        info!("Slot 1: Volume Down (WASAPI + Key)");
        if let Ok(audio) = self.audio.lock() {
            let current = audio.get_master_volume();
            audio.set_master_volume((current - 0.05).max(0.0));
        }
        self.press_single_key(VK_VOLUME_DOWN);
    }

    fn action_volume_up(&self) {
        info!("Slot 2: Volume Up (WASAPI + Key)");
        if let Ok(audio) = self.audio.lock() {
            let current = audio.get_master_volume();
            audio.set_master_volume((current + 0.05).min(1.0));
        }
        self.press_single_key(VK_VOLUME_UP);
    }

    /// Press and release a single virtual key
    pub fn press_single_key(&self, vk: VIRTUAL_KEY) {
        let down = Self::create_key_input(vk, false);
        let up = Self::create_key_input(vk, true);

        unsafe {
            let inputs = [down, up];
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                error!("Failed to send input for VK {:?}, sent: {}", vk, sent);
            }
        }
    }

    /// Press keys in sequence, then release in reverse order
    pub fn press_hotkey(&self, keys: &[VIRTUAL_KEY]) {
        let mut inputs = Vec::with_capacity(keys.len() * 2);

        // Press down all modifiers and keys
        for &vk in keys {
            inputs.push(Self::create_key_input(vk, false));
        }

        // Release in reverse order
        for &vk in keys.iter().rev() {
            inputs.push(Self::create_key_input(vk, true));
        }

        unsafe {
            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                error!(
                    "Failed to send hotkey sequence, sent: {} of {}",
                    sent,
                    inputs.len()
                );
            }
        }
    }

    fn create_key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
        let flags = if key_up {
            KEYEVENTF_KEYUP
        } else {
            KEYBD_EVENT_FLAGS(0)
        };

        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }
}
