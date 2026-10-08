use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use uuid::Uuid;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PairingStore {
    #[serde(default)]
    devices: HashMap<String, String>,
    #[serde(skip)]
    pending: Option<PendingCode>,
    #[serde(skip)]
    failed_attempts: HashMap<String, Instant>,
}

#[derive(Debug)]
struct PendingCode {
    code: String,
    expires_at: Instant,
    attempts_left: u8,
}

fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    hex::encode(hasher.finalize())
}

impl PairingStore {
    pub fn load(path: &Path) -> io::Result<Self> {
        let mut store = match fs::read_to_string(path) {
            Ok(data) => serde_json::from_str::<Self>(&data)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Self::default(),
            Err(error) => return Err(error),
        };

        let mut modified = false;
        let mut new_devices = HashMap::new();
        for (k, v) in &store.devices {
            if v.len() != 64 || hex::decode(v).is_err() {
                // Legacy plaintext, rejecting
                modified = true;
            } else {
                new_devices.insert(k.clone(), v.clone());
            }
        }
        if modified {
            store.devices = new_devices;
            store.persist(&store.devices, path)?;
        }

        Ok(store)
    }

    pub fn start(&mut self) -> String {
        let random = Uuid::new_v4();
        let code = format!(
            "{:06}",
            u64::from_le_bytes(random.as_bytes()[..8].try_into().unwrap()) % 1_000_000
        );
        self.pending = Some(PendingCode {
            code: code.clone(),
            expires_at: Instant::now() + Duration::from_secs(300),
            attempts_left: 10,
        });
        self.failed_attempts.clear();
        code
    }

    pub fn check_throttle(&mut self, device_id: &str) -> bool {
        if device_id.len() > 128 {
            return false;
        }
        if let Some(&last_failed) = self.failed_attempts.get(device_id)
            && last_failed.elapsed() < Duration::from_secs(2)
        {
            return false;
        }
        true
    }

    pub fn record_failure(&mut self, device_id: &str) {
        if device_id.len() > 128 {
            return;
        }
        self.failed_attempts
            .retain(|_, failed| failed.elapsed() < Duration::from_secs(2));
        if self.failed_attempts.len() >= 256 && !self.failed_attempts.contains_key(device_id) {
            return;
        }
        self.failed_attempts
            .insert(device_id.to_string(), Instant::now());
    }

    pub fn pair(&mut self, code: &str, device_id: &str, path: &Path) -> io::Result<Option<String>> {
        if !self.check_throttle(device_id) {
            return Ok(None);
        }

        let Some(pending) = self.pending.as_mut() else {
            self.record_failure(device_id);
            return Ok(None);
        };
        if Instant::now() >= pending.expires_at || pending.attempts_left == 0 {
            self.pending = None;
            self.record_failure(device_id);
            return Ok(None);
        }
        if pending.code != code || device_id.trim().is_empty() || device_id.len() > 128 {
            pending.attempts_left -= 1;
            self.record_failure(device_id);
            return Ok(None);
        }
        let token = Uuid::new_v4().simple().to_string();
        let mut devices = self.devices.clone();
        devices.insert(device_id.to_string(), hash_token(&token));
        self.persist(&devices, path)?;
        self.devices = devices;
        self.pending = None;
        Ok(Some(token))
    }

    pub fn is_authorized(&mut self, device_id: &str, token: &str) -> bool {
        if !self.check_throttle(device_id) {
            return false;
        }

        if token.is_empty() {
            self.record_failure(device_id);
            return false;
        }
        let hashed = hash_token(token);
        let authorized = self
            .devices
            .get(device_id)
            .is_some_and(|saved| saved.as_bytes().ct_eq(hashed.as_bytes()).unwrap_u8() == 1);
        if !authorized {
            self.record_failure(device_id);
        }
        authorized
    }

    pub fn revoke(&mut self, device_id: &str, path: &Path) -> io::Result<bool> {
        if !self.devices.contains_key(device_id) {
            return Ok(false);
        }
        let mut devices = self.devices.clone();
        devices.remove(device_id);
        self.persist(&devices, path)?;
        self.devices = devices;
        Ok(true)
    }

    pub fn devices(&self) -> Vec<String> {
        let mut devices: Vec<_> = self.devices.keys().cloned().collect();
        devices.sort();
        devices
    }

    fn persist(&self, devices: &HashMap<String, String>, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_vec_pretty(&Self {
            devices: devices.clone(),
            pending: None,
            failed_attempts: HashMap::new(),
        })?;
        crate::storage::replace_file(path, &data)
    }
}

pub fn pairing_path() -> PathBuf {
    let mut path = crate::config::AppConfig::config_path();
    path.set_file_name("paired-devices.json");
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_plaintext_credentials_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("devices.json");
        fs::write(&path, r#"{"devices":{"phone":"legacy-plaintext-token"}}"#).unwrap();
        let mut store = PairingStore::load(&path).unwrap();
        assert!(!store.is_authorized("phone", "legacy-plaintext-token"));
        assert!(store.devices().is_empty());
        assert!(
            !fs::read_to_string(&path)
                .unwrap()
                .contains("legacy-plaintext-token")
        );
    }

    #[test]
    fn failed_attempt_tracking_is_bounded() {
        let mut store = PairingStore::default();
        for id in 0..1_000 {
            store.record_failure(&format!("device-{id}"));
        }
        assert!(store.failed_attempts.len() <= 256);
        store.record_failure(&"x".repeat(129));
        assert!(store.failed_attempts.len() <= 256);
        assert!(!store.check_throttle(&"x".repeat(129)));
    }

    #[test]
    fn pairing_survives_restart_and_revoke() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("devices.json");
        let mut store = PairingStore::load(&path).unwrap();
        assert!(!store.is_authorized("phone", ""));
        store.failed_attempts.clear();
        let code = store.start();
        assert_eq!(code.len(), 6);
        assert!(code.bytes().all(|digit| digit.is_ascii_digit()));
        assert!(store.pair("wrong", "phone", &path).unwrap().is_none());
        store.failed_attempts.clear();
        for _ in 0..9 {
            assert!(store.pair("wrong", "phone", &path).unwrap().is_none());
            store.failed_attempts.clear();
        }
        assert!(store.pair(&code, "phone", &path).unwrap().is_none());
        store.failed_attempts.clear();
        let code = store.start();
        let token = store.pair(&code, "phone", &path).unwrap().unwrap();
        let mut loaded = PairingStore::load(&path).unwrap();
        assert!(!std::fs::read_to_string(&path).unwrap().contains(&token));
        assert!(loaded.is_authorized("phone", &token));
        assert!(!loaded.is_authorized("phone", "wrong"));
        assert!(loaded.pair(&code, "other", &path).unwrap().is_none());
        loaded.failed_attempts.clear();
        let replacement_code = loaded.start();
        let new_token = loaded
            .pair(&replacement_code, "phone", &path)
            .unwrap()
            .unwrap();
        assert_ne!(token, new_token);
        assert!(
            !PairingStore::load(&path)
                .unwrap()
                .is_authorized("phone", &token)
        );
        assert!(loaded.revoke("phone", &path).unwrap());
        assert!(
            !PairingStore::load(&path)
                .unwrap()
                .is_authorized("phone", &token)
        );
    }
}
