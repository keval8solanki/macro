use crate::event::SerializableEvent;
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub struct RecorderState {
    pub is_recording: bool,
    pub events: Vec<SerializableEvent>,
    pub filename: Option<String>,
    pub last_time: SystemTime,
    pub cmd_pressed: bool,
    pub alt_pressed: bool,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
}

impl RecorderState {
    pub fn new() -> Self {
        Self {
            is_recording: false,
            events: Vec::new(),
            filename: None,
            last_time: SystemTime::now(),
            cmd_pressed: false,
            alt_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
        }
    }
}

pub fn listen_loop(state: Arc<Mutex<RecorderState>>) -> anyhow::Result<()> {
    listen(move |event| {
        let mut state = state.lock().unwrap();
        
        // Update modifiers
        match event.event_type {
            EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => state.cmd_pressed = true,
            EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => state.cmd_pressed = false,
            EventType::KeyPress(Key::Alt) | EventType::KeyPress(Key::AltGr) => state.alt_pressed = true,
            EventType::KeyRelease(Key::Alt) | EventType::KeyRelease(Key::AltGr) => state.alt_pressed = false,
            EventType::KeyPress(Key::ControlLeft) | EventType::KeyPress(Key::ControlRight) => state.ctrl_pressed = true,
            EventType::KeyRelease(Key::ControlLeft) | EventType::KeyRelease(Key::ControlRight) => state.ctrl_pressed = false,
            EventType::KeyPress(Key::ShiftLeft) | EventType::KeyPress(Key::ShiftRight) => state.shift_pressed = true,
            EventType::KeyRelease(Key::ShiftLeft) | EventType::KeyRelease(Key::ShiftRight) => state.shift_pressed = false,
            _ => {}
        }

        if state.is_recording {
             let now = SystemTime::now();
             let delay = now.duration_since(state.last_time).unwrap_or(std::time::Duration::from_millis(0)).as_millis() as u64;
             state.last_time = now;

             if let Some(serializable_event) = SerializableEvent::from_rdev(event.clone(), delay) {
                 state.events.push(serializable_event);
             }
        }
    }).map_err(|e| anyhow::anyhow!("Listen error: {:?}", e))
}
