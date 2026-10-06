use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
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

        Self { profiles }
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<AppConfig>(&data) {
                // Validate and sanitize loaded config
                let mut valid = true;
                for profile in config.profiles.values() {
                    if !profile.validate() {
                        valid = false;
                        break;
                    }
                }
                if valid
                    && Self::default_config()
                        .profiles
                        .keys()
                        .all(|name| config.profiles.contains_key(name))
                {
                    return config;
                }
            }
            // Corrupt or invalid config: return default, do NOT overwrite the corrupt file
            return Self::default_config();
        }
        let default = Self::default_config();
        let _ = default.save(); // Save defaults if missing
        default
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let temp_path = path.with_extension("tmp");
        fs::write(&temp_path, data)?;
        fs::rename(temp_path, path)?;
        Ok(())
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
        let config_file = temp_dir.path().join("config.json");
        unsafe {
            std::env::set_var("STREAMDECK_TEST_CONFIG_PATH", config_file.to_str().unwrap());
        }

        // 1. Initial load should create defaults
        let config1 = AppConfig::load();
        assert!(config1.profiles.contains_key("Default"));
        assert!(config1.profiles.contains_key("VSCode"));

        let default_profile = config1.profiles.get("Default").unwrap();
        assert_eq!(default_profile.keys.len(), 9);
        assert_eq!(default_profile.keys[0].action, "vol_mute");

        // 2. Modify and save
        let mut config2 = AppConfig::load();
        config2.profiles.get_mut("Default").unwrap().keys[0].label = "Muted".to_string();
        let _ = config2.save();

        // 3. Reload and verify
        let config3 = AppConfig::load();
        assert_eq!(
            config3.profiles.get("Default").unwrap().keys[0].label,
            "Muted"
        );

        // 4. Test corruption handling
        fs::write(&config_file, "{ corrupt_json: 1").unwrap();
        let config_corrupt = AppConfig::load();
        // Should fall back to default but NOT overwrite the corrupt file
        assert_eq!(
            config_corrupt.profiles.get("Default").unwrap().keys[0].label,
            "Mute Audio"
        ); // Default value
        let file_contents = fs::read_to_string(&config_file).unwrap();
        assert_eq!(file_contents, "{ corrupt_json: 1"); // File left untouched

        unsafe {
            std::env::remove_var("STREAMDECK_TEST_CONFIG_PATH");
        }
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
