// [LINE BUDGET AUDIT] 0/200
use std::sync::{Arc, Mutex};
use thiserror::Error;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator, eMultimedia, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx,
};

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Windows API Error: {0}")]
    Windows(#[from] windows::core::Error),
}

pub struct AudioController {
    endpoint_volume: Option<IAudioEndpointVolume>,
}

unsafe impl Send for AudioController {}
unsafe impl Sync for AudioController {}

impl AudioController {
    pub fn new() -> Result<Arc<Mutex<Self>>, AudioError> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia)?;
            let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

            Ok(Arc::new(Mutex::new(Self {
                endpoint_volume: Some(endpoint_volume),
            })))
        }
    }

    #[allow(dead_code)]
    pub fn stub() -> Arc<Mutex<Self>> {
        tracing::warn!("WASAPI init failed, using AudioController stub");
        Arc::new(Mutex::new(Self {
            endpoint_volume: None,
        }))
    }

    #[allow(dead_code)]
    pub fn set_master_volume(&self, level: f32) {
        if let Some(vol) = &self.endpoint_volume {
            let level = level.clamp(0.0, 1.0);
            unsafe {
                let _ = vol.SetMasterVolumeLevelScalar(level, std::ptr::null());
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_master_volume(&self) -> f32 {
        if let Some(vol) = &self.endpoint_volume {
            unsafe {
                if let Ok(level) = vol.GetMasterVolumeLevelScalar() {
                    return level;
                }
            }
        }
        0.0
    }

    #[allow(dead_code)]
    pub fn toggle_mute(&self) {
        if let Some(vol) = &self.endpoint_volume {
            unsafe {
                if let Ok(muted) = vol.GetMute() {
                    let new_mute = !muted.as_bool();
                    let _ = vol.SetMute(new_mute, std::ptr::null());
                }
            }
        }
    }
}
