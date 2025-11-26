use crate::event::SerializableEvent;
use anyhow::Result;
use rdev::{listen, Event, EventType, Key};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

struct RecorderState {
    is_recording: bool,
    cmd_pressed: bool,
    alt_pressed: bool,
    events: Vec<SerializableEvent>,
    last_time: SystemTime,
}

pub fn run_record(output_path: PathBuf) -> Result<()> {
    println!("Running in background.");
    println!("Start Recording: Cmd + Option + R");
    println!("Stop Recording: Cmd + Option + Esc");

    let state = Arc::new(Mutex::new(RecorderState {
        is_recording: false,
        cmd_pressed: false,
        alt_pressed: false,
        events: Vec::new(),
        last_time: SystemTime::now(),
    }));

    let state_clone = state.clone();
    let output_path_clone = output_path.clone();

    let callback = move |event: Event| {
        let mut state = state_clone.lock().unwrap();
        
        // Update modifier keys
        match event.event_type {
            EventType::KeyPress(Key::MetaLeft) | EventType::KeyPress(Key::MetaRight) => state.cmd_pressed = true,
            EventType::KeyRelease(Key::MetaLeft) | EventType::KeyRelease(Key::MetaRight) => state.cmd_pressed = false,
            EventType::KeyPress(Key::Alt) | EventType::KeyPress(Key::AltGr) => state.alt_pressed = true,
            EventType::KeyRelease(Key::Alt) | EventType::KeyRelease(Key::AltGr) => state.alt_pressed = false,
            _ => {}
        }

        // Check for Hotkeys
        if state.cmd_pressed && state.alt_pressed {
            if let EventType::KeyPress(Key::KeyR) = event.event_type {
                if !state.is_recording {
                    println!("Recording started...");
                    state.is_recording = true;
                    state.events.clear();
                    state.last_time = SystemTime::now();
                    return; // Don't record the hotkey itself
                }
            }
            if let EventType::KeyPress(Key::Escape) = event.event_type {
                if state.is_recording {
                    println!("Recording stopped.");
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
        println!("Error: {:?}", error);
        return Err(anyhow::anyhow!("Listen error: {:?}", error));
    }

    Ok(())
}

fn save_events(events: &[SerializableEvent], path: &PathBuf) {
    let file = File::create(path).expect("Failed to create file");
    serde_json::to_writer_pretty(file, events).expect("Failed to write to file");
    println!("Saved to {:?}", path);
}
