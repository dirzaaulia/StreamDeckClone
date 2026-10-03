use std::time::{SystemTime, UNIX_EPOCH};

use crate::protocol::streamdeck::{
    deck_message, DeckMessage, KeySlotConfig, PageLayoutUpdate,
};

pub fn create_default_layout() -> DeckMessage {
    let keys = vec![
        make_slot(0, "Mute Audio", "#E53935", ""),
        make_slot(1, "Vol Down", "#1E88E5", ""),
        make_slot(2, "Vol Up", "#1E88E5", ""),
        make_slot(3, "Play / Pause", "#43A047", ""),
        make_slot(4, "Prev Track", "#3949AB", ""),
        make_slot(5, "Next Track", "#3949AB", ""),
        make_slot(6, "Desktop", "#8E24AA", "Win+D"),
        make_slot(7, "Task Mgr", "#FB8C00", "Esc"),
        make_slot(8, "Screenshot", "#00ACC1", "Win+Shift+S"),
    ];

    DeckMessage {
        timestamp: current_time_millis(),
        payload: Some(deck_message::Payload::LayoutUpdate(PageLayoutUpdate {
            profile_id: "default".to_string(),
            page_id: "main".to_string(),
            columns: 3,
            rows: 3,
            keys,
        })),
    }
}

fn make_slot(index: i32, title: &str, bg_color: &str, badge_text: &str) -> KeySlotConfig {
    KeySlotConfig {
        index,
        title: title.to_string(),
        icon_url: String::new(),
        icon_bytes: Vec::new(),
        background_color: bg_color.to_string(),
        badge_text: badge_text.to_string(),
        badge_color: String::new(),
        toggle_state: 0,
        is_dial: false,
    }
}

pub fn current_time_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
