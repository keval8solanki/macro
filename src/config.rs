use rdev::Key;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyMaps {
    pub start_recording: KeyCombo,
    pub stop_recording: KeyCombo,
    pub start_playback: KeyCombo,
    pub stop_playback: KeyCombo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyCombo {
    pub modifiers: Vec<Modifier>,
    pub trigger: Key,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Cmd,
    Alt,
    Ctrl,
    Shift,
}

impl Default for KeyMaps {
    fn default() -> Self {
        Self {
            start_recording: KeyCombo {
                modifiers: vec![Modifier::Cmd, Modifier::Shift],
                trigger: Key::Num1,
            },
            stop_recording: KeyCombo {
                modifiers: vec![Modifier::Cmd, Modifier::Shift],
                trigger: Key::Num1,
            },
            start_playback: KeyCombo {
                modifiers: vec![Modifier::Cmd, Modifier::Shift],
                trigger: Key::Num2,
            },
            stop_playback: KeyCombo {
                modifiers: vec![Modifier::Cmd, Modifier::Shift],
                trigger: Key::Num2,
            },
        }
    }
}

