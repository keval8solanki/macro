use crate::event::SerializableEvent;
use crate::config::{KeyMaps, Modifier};
use anyhow::Result;
use cliclack::log;
use rdev::{listen, Event, EventType, Key};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

struct RecorderState {
    is_recording: bool,
    cmd_pressed: bool,
    alt_pressed: bool,
    ctrl_pressed: bool,
    shift_pressed: bool,
    events: Vec<SerializableEvent>,
    last_time: SystemTime,
}

pub fn run_record(output_path: PathBuf, keymaps: KeyMaps) -> Result<()> {
    log::info("Running in background.")?;
    log::info(format!("Start Recording: {:?} + {:?}", keymaps.start_recording.modifiers, keymaps.start_recording.trigger))?;
    log::info(format!("Stop Recording: {:?} + {:?}", keymaps.stop_recording.modifiers, keymaps.stop_recording.trigger))?;

    let state = Arc::new(Mutex::new(RecorderState {
        is_recording: false,
        cmd_pressed: false,
        alt_pressed: false,
        ctrl_pressed: false,
        shift_pressed: false,
        events: Vec::new(),
        last_time: SystemTime::now(),
    }));

    let state_clone = state.clone();
    let output_path_clone = output_path.clone();
    let keymaps = keymaps.clone();

    let callback = move |event: Event| {
        let mut state = state_clone.lock().unwrap();
        
        // Update modifier keys
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

        // Check for Hotkeys
        let check_modifiers = |modifiers: &[Modifier]| -> bool {
            for m in modifiers {
                match m {
                    Modifier::Cmd => if !state.cmd_pressed { return false; },
                    Modifier::Alt => if !state.alt_pressed { return false; },
                    Modifier::Ctrl => if !state.ctrl_pressed { return false; },
                    Modifier::Shift => if !state.shift_pressed { return false; },
                }
            }
            true
        };

        if let EventType::KeyPress(key) = event.event_type {
            // Start Recording
            if key == keymaps.start_recording.trigger && check_modifiers(&keymaps.start_recording.modifiers) {
                if !state.is_recording {
                    let _ = log::info("Recording started...");
                    state.is_recording = true;
                    state.events.clear();
                    state.last_time = SystemTime::now();
                    return; // Don't record the hotkey itself
                }
            }
            // Stop Recording
            if key == keymaps.stop_recording.trigger && check_modifiers(&keymaps.stop_recording.modifiers) {
                if state.is_recording {
                    let _ = log::info("Recording stopped.");
                    state.is_recording = false;
                    save_events(&state.events, &output_path_clone);
                    return; // Don't record the hotkey itself
                }
            }
        }

        if state.is_recording {
             let now = SystemTime::now();
             let delay = now.duration_since(state.last_time).unwrap().as_millis() as u64;
             state.last_time = now;

             if let Some(serializable_event) = SerializableEvent::from_rdev(event.clone(), delay) {
                 state.events.push(serializable_event);
             }
        }
    };

    if let Err(error) = listen(callback) {
        log::error(format!("Error: {:?}", error))?;
        return Err(anyhow::anyhow!("Listen error: {:?}", error));
    }

    Ok(())
}

fn save_events(events: &[SerializableEvent], path: &PathBuf) {
    let file = File::create(path).expect("Failed to create file");
    serde_json::to_writer_pretty(file, events).expect("Failed to write to file");
    let _ = log::success(format!("Saved to {:?}", path));
    std::process::exit(0);
}
