use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::Profile;
use crate::protocol::streamdeck::{DeckMessage, KeySlotConfig, PageLayoutUpdate, deck_message};

pub fn create_layout_message(profile_id: &str, profile: &Profile) -> DeckMessage {
    let mut sorted_keys: Vec<_> = profile.keys.iter().collect();
    sorted_keys.sort_by_key(|key| key.id);
    let mut keys = Vec::new();
    for key in sorted_keys {
        keys.push(KeySlotConfig {
            index: key.id as i32,
            title: key.label.clone(),
            icon_url: key.icon.clone(),
            icon_bytes: Vec::new(),
            background_color: "#1E88E5".to_string(), // default or map from action
            badge_text: "".to_string(),
            badge_color: String::new(),
            toggle_state: 0,
            is_dial: false,
        });
    }

    DeckMessage {
        timestamp: current_time_millis(),
        payload: Some(deck_message::Payload::LayoutUpdate(PageLayoutUpdate {
            profile_id: profile_id.to_string(),
            page_id: "main".to_string(),
            columns: 3,
            rows: 3,
            keys,
        })),
    }
}

pub fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn layout_uses_stable_slot_ids_even_when_keys_are_reordered() {
        let mut profile = AppConfig::default_config()
            .profiles
            .remove("Default")
            .unwrap();
        profile.keys.reverse();
        let message = create_layout_message("Default", &profile);
        let Some(deck_message::Payload::LayoutUpdate(layout)) = message.payload else {
            panic!("expected layout update");
        };
        for (index, key) in layout.keys.into_iter().enumerate() {
            assert_eq!(key.index, index as i32);
            assert_eq!(
                key.title,
                profile
                    .keys
                    .iter()
                    .find(|item| item.id == key.index as u32)
                    .unwrap()
                    .label
            );
        }
    }
}
