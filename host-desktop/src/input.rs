// [LINE BUDGET AUDIT] 124/250
use std::sync::{Arc, Mutex};
use tracing::{error, info};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
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

    /// Execute predefined action mapped to a key index
    pub fn trigger_key_index(&self, key_index: i32) {
        match key_index {
            0 => self.action_toggle_mute(),
            1 => self.action_volume_down(),
            2 => self.action_volume_up(),
            3 => {
                info!("Slot 3: Media Play/Pause");
                self.press_single_key(VK_MEDIA_PLAY_PAUSE);
            }
            4 => {
                info!("Slot 4: Media Previous");
                self.press_single_key(VK_MEDIA_PREV_TRACK);
            }
            5 => {
                info!("Slot 5: Media Next");
                self.press_single_key(VK_MEDIA_NEXT_TRACK);
            }
            6 => {
                info!("Slot 6: Show Desktop (Win + D)");
                self.press_hotkey(&[VK_LWIN, VK_D]);
            }
            7 => {
                info!("Slot 7: Task Manager (Ctrl + Shift + Esc)");
                self.press_hotkey(&[VK_CONTROL, VK_SHIFT, VK_ESCAPE]);
            }
            8 => {
                info!("Slot 8: Take Screenshot (Win + Shift + S)");
                self.press_hotkey(&[VK_LWIN, VK_SHIFT, VIRTUAL_KEY(0x53)]);
            }
            _ => {
                info!("Unhandled slot index: {}", key_index);
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
                error!("Failed to send hotkey sequence, sent: {} of {}", sent, inputs.len());
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
