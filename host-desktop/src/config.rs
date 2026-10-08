use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct KeyConfig {
    pub id: u32,
    pub label: String,
    pub action: String,
    pub icon: String,
}

impl KeyConfig {
    pub fn is_valid(&self) -> bool {
        if self.label.chars().count() > 32
            || self.action.chars().count() > 32
            || self.icon.chars().count() > 64
        {
            return false;
        }
        let valid_actions = [
            "vol_mute",
            "vol_down",
            "vol_up",
            "media_play_pause",
            "media_prev",
            "media_next",
            "desktop",
            "taskmgr",
            "screenshot",
            "ctrl_c",
            "ctrl_v",
            "ctrl_z",
            "ctrl_s",
            "ctrl_w",
            "f5",
            "",
        ];
        valid_actions.contains(&self.action.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub keys: Vec<KeyConfig>,
}

impl Profile {
    pub fn validate(&self) -> bool {
        let mut seen_ids = HashSet::new();
        self.name.chars().count() <= 32
            && self.keys.len() == 9
            && self
                .keys
                .iter()
                .all(|key| key.id < 9 && seen_ids.insert(key.id) && key.is_valid())
    }
}

const CONFIG_VERSION: u32 = 1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyConfig {
    profiles: HashMap<String, Profile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub version: u32,
    pub profiles: HashMap<String, Profile>,
}

impl AppConfig {
    pub fn default_config() -> Self {
        let mut profiles = HashMap::new();
        let profile_names = vec!["Default", "Browser", "VSCode", "OBS", "VisualStudio"];

        for name in profile_names {
            let keys = if name == "Default" {
                vec![
                    KeyConfig {
                        id: 0,
                        label: "Mute Audio".to_string(),
                        action: "vol_mute".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 1,
                        label: "Vol Down".to_string(),
                        action: "vol_down".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 2,
                        label: "Vol Up".to_string(),
                        action: "vol_up".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 3,
                        label: "Play / Pause".to_string(),
                        action: "media_play_pause".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 4,
                        label: "Prev Track".to_string(),
                        action: "media_prev".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 5,
                        label: "Next Track".to_string(),
                        action: "media_next".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 6,
                        label: "Desktop".to_string(),
                        action: "desktop".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 7,
                        label: "Task Mgr".to_string(),
                        action: "taskmgr".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 8,
                        label: "Screenshot".to_string(),
                        action: "screenshot".to_string(),
                        icon: "".to_string(),
                    },
                ]
            } else {
                vec![
                    KeyConfig {
                        id: 0,
                        label: "Copy".to_string(),
                        action: "ctrl_c".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 1,
                        label: "Paste".to_string(),
                        action: "ctrl_v".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 2,
                        label: "Undo".to_string(),
                        action: "ctrl_z".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 3,
                        label: "Save".to_string(),
                        action: "ctrl_s".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 4,
                        label: "Close Tab".to_string(),
                        action: "ctrl_w".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 5,
                        label: "F5".to_string(),
                        action: "f5".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 6,
                        label: "".to_string(),
                        action: "".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 7,
                        label: "".to_string(),
                        action: "".to_string(),
                        icon: "".to_string(),
                    },
                    KeyConfig {
                        id: 8,
                        label: "".to_string(),
                        action: "".to_string(),
                        icon: "".to_string(),
                    },
                ]
            };

            profiles.insert(
                name.to_string(),
                Profile {
                    name: name.to_string(),
                    keys,
                },
            );
        }

        Self {
            version: CONFIG_VERSION,
            profiles,
        }
    }

    fn valid(&self) -> bool {
        self.version == CONFIG_VERSION
            && self
                .profiles
                .iter()
                .all(|(name, profile)| name == &profile.name && profile.validate())
            && Self::default_config()
                .profiles
                .keys()
                .all(|name| self.profiles.contains_key(name))
    }

    pub fn load() -> Result<Self, std::io::Error> {
        Self::load_from(&Self::config_path())
    }

    fn load_from(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let data = match fs::read_to_string(path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let default = Self::default_config();
                default.save_to(path)?;
                return Ok(default);
            }
            Err(error) => return Err(error),
        };
        let value: serde_json::Value = serde_json::from_str(&data)?;
        let legacy = value.get("version").is_none();
        if !legacy
            && value.get("version").and_then(serde_json::Value::as_u64)
                != Some(CONFIG_VERSION as u64)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported profile version",
            ));
        }
        let config: Self = if legacy {
            let old: LegacyConfig = serde_json::from_value(value)?;
            Self {
                version: CONFIG_VERSION,
                profiles: old.profiles,
            }
        } else {
            serde_json::from_value(value)?
        };
        if !config.valid() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid profile config",
            ));
        }
        if legacy {
            let backup = path.with_extension("legacy.json");
            match fs::read(&backup) {
                Ok(existing) if existing != data.as_bytes() => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::AlreadyExists,
                        "Different legacy backup exists",
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    let mut backup_file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&backup)?;
                    use std::io::Write;
                    backup_file.write_all(data.as_bytes())?;
                    backup_file.sync_all()?;
                }
                Err(error) => return Err(error),
            }
            config.save_to(path)?;
        }
        Ok(config)
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        self.save_to(&Self::config_path())
    }

    fn save_to(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        if !self.valid() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid profile config",
            ));
        }
        // Loading a damaged or future-version file must never turn a later save
        // into an overwrite of that file.
        match fs::read_to_string(path) {
            Ok(existing) => {
                let value: serde_json::Value = serde_json::from_str(&existing)?;
                let version = value.get("version").and_then(serde_json::Value::as_u64);
                if version != Some(CONFIG_VERSION as u64) && version.is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Unsupported profile version",
                    ));
                }
                let valid_existing = if version.is_none() {
                    let legacy: LegacyConfig = serde_json::from_value(value)?;
                    Self {
                        version: CONFIG_VERSION,
                        profiles: legacy.profiles,
                    }
                    .valid()
                } else {
                    let existing_config: Self = serde_json::from_value(value)?;
                    existing_config.valid()
                };
                if !valid_existing {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid existing profile config",
                    ));
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_vec_pretty(self)?;
        crate::storage::replace_file(path, &data)
    }

    pub fn config_path() -> PathBuf {
        if let Ok(p) = std::env::var("STREAMDECK_TEST_CONFIG_PATH") {
            return PathBuf::from(p);
        }
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("StreamDeckClone");
        path.push("config.json");
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_and_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("config.json");
        let mut config = AppConfig::load_from(&path).unwrap();
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.profiles["Default"].keys[0].action, "vol_mute");
        config.profiles.get_mut("Default").unwrap().keys[0].label = "Muted".into();
        config.save_to(&path).unwrap();
        assert_eq!(
            AppConfig::load_from(&path).unwrap().profiles["Default"].keys[0].label,
            "Muted"
        );
    }

    #[test]
    fn migration_keeps_original_backup_and_profiles() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut old = AppConfig::default_config();
        let keys = &mut old.profiles.get_mut("Default").unwrap().keys;
        keys[0].label = "Personal".into();
        keys[0].icon = "custom-icon".into();
        keys.swap(0, 8);
        let legacy =
            serde_json::to_string(&serde_json::json!({ "profiles": old.profiles })).unwrap();
        fs::write(&path, &legacy).unwrap();
        let migrated = AppConfig::load_from(&path).unwrap();
        assert_eq!(migrated.profiles["Default"].keys[8].label, "Personal");
        assert_eq!(migrated.profiles["Default"].keys[8].icon, "custom-icon");
        assert_eq!(migrated.profiles["Default"].keys[8].id, 0);
        assert_eq!(migrated.profiles["Default"].keys[0].id, 8);
        assert_eq!(migrated.version, CONFIG_VERSION);
        assert_eq!(
            fs::read_to_string(path.with_extension("legacy.json")).unwrap(),
            legacy
        );
        assert_eq!(
            AppConfig::load_from(&path).unwrap().profiles["Default"].keys[8].label,
            "Personal"
        );
    }

    #[test]
    fn interrupted_migration_reuses_matching_backup_and_preserves_conflicts() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let legacy = serde_json::to_string(&serde_json::json!({
            "profiles": AppConfig::default_config().profiles
        }))
        .unwrap();
        fs::write(&path, &legacy).unwrap();
        let backup = path.with_extension("legacy.json");
        fs::write(&backup, &legacy).unwrap();
        assert!(AppConfig::load_from(&path).is_ok());
        assert_eq!(fs::read_to_string(&backup).unwrap(), legacy);

        fs::write(&path, &legacy).unwrap();
        fs::write(&backup, "different backup").unwrap();
        assert!(AppConfig::load_from(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), legacy);
        assert_eq!(fs::read_to_string(&backup).unwrap(), "different backup");
    }

    #[test]
    fn invalid_and_future_configs_are_never_replaced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        for data in [
            "{ corrupt_json: 1",
            r#"{"version":99,"profiles":{}}"#,
            r#"{"version":1,"profiles":{}}"#,
        ] {
            fs::write(&path, data).unwrap();
            assert!(AppConfig::load_from(&path).is_err());
            assert!(AppConfig::default_config().save_to(&path).is_err());
            assert_eq!(fs::read_to_string(&path).unwrap(), data);
        }
    }

    #[test]
    fn unknown_fields_are_not_silently_discarded() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let mut value = serde_json::to_value(AppConfig::default_config()).unwrap();
        value["experimental"] = serde_json::json!(true);
        let data = serde_json::to_string(&value).unwrap();
        fs::write(&path, &data).unwrap();
        assert!(AppConfig::load_from(&path).is_err());
        assert!(AppConfig::default_config().save_to(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), data);
    }

    #[test]
    fn test_profile_validation() {
        let mut profile = Profile::default();
        for i in 0..9 {
            profile.keys.push(KeyConfig {
                id: i,
                label: "Test".to_string(),
                action: "vol_mute".to_string(),
                icon: "".to_string(),
            });
        }
        assert!(profile.validate());

        profile.keys[0].action = "invalid_action".to_string();
        assert!(!profile.validate());
        profile.keys[0].action = "vol_mute".to_string();

        profile.keys[1].id = 10;
        assert!(!profile.validate());
        profile.keys[1].id = 1;

        profile.keys[2].label = "a".repeat(100);
        assert!(!profile.validate());
    }
}
