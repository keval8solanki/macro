use crate::event::SerializableEvent;
use crate::config::{KeyMaps, Modifier};
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
    ctrl_pressed: bool,
    shift_pressed: bool,
    events: Vec<SerializableEvent>,
    last_time: SystemTime,
}

pub fn run_record(output_path: PathBuf, keymaps: KeyMaps, immediate: bool) -> Result<()> {
    log::info!("Running in background.");
    log::info!("Start Recording: {:?} + {:?}", keymaps.start_recording.modifiers, keymaps.start_recording.trigger);
    log::info!("Stop Recording: {:?} + {:?}", keymaps.stop_recording.modifiers, keymaps.stop_recording.trigger);

    // Create file immediately to ensure it exists
    save_events(&[], &output_path)?;

    let state = Arc::new(Mutex::new(RecorderState {
        is_recording: immediate,
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

    // Handle Ctrl+C / SIGTERM
    let state_ctrlc = state.clone();
    let output_path_ctrlc = output_path.clone();
    ctrlc::set_handler(move || {
        log::info!("Ctrl+C / SIGTERM handler triggered");
        let state = state_ctrlc.lock().unwrap();
        if state.is_recording {
            log::info!("Received termination signal. Saving recording...");
            if let Err(e) = save_events(&state.events, &output_path_ctrlc) {
                log::error!("Failed to save events: {}", e);
            }
        } else {
            log::info!("Not recording, exiting without save.");
        }
        std::process::exit(0);
    })?;

    let callback = move |event: Event| {
        // log::trace!("Received event: {:?}", event.event_type); // Too noisy for info level, but good for debug
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
                    log::info!("Recording started...");
                    state.is_recording = true;
                    state.events.clear();
                    state.last_time = SystemTime::now();
                    return; // Don't record the hotkey itself
                }
            }
            // Stop Recording
            if key == keymaps.stop_recording.trigger && check_modifiers(&keymaps.stop_recording.modifiers) {
                if state.is_recording {
                    log::info!("Recording stopped.");
                    state.is_recording = false;
                    if let Err(e) = save_events(&state.events, &output_path_clone) {
                        log::error!("Failed to save events: {}", e);
                    }
                    std::process::exit(0);
                }
            }
        }

        if state.is_recording {
             let now = SystemTime::now();
             let delay = now.duration_since(state.last_time).unwrap().as_millis() as u64;
             state.last_time = now;

             if let Some(serializable_event) = SerializableEvent::from_rdev(event.clone(), delay) {
                 log::info!("Recorded event: {:?}", serializable_event);
                 state.events.push(serializable_event);
                 
                 // Save immediately to ensure data persistence
                 if let Err(e) = save_events(&state.events, &output_path_clone) {
                     log::error!("Failed to save events: {}", e);
                 }
             }
        }
    };

    if let Err(error) = listen(callback) {
        log::error!("Error: {:?}", error);
        return Err(anyhow::anyhow!("Listen error: {:?}", error));
    }

    Ok(())
}

pub fn save_events(events: &[SerializableEvent], path: &PathBuf) -> Result<()> {
    if events.is_empty() {
        log::warn!("No events captured! This usually means the application does not have Accessibility Permissions.");
        log::warn!("Please check System Settings -> Privacy & Security -> Accessibility.");
    }
    log::info!("Saving {} events to {:?}", events.len(), path);
    let file = File::create(path)?;
    serde_json::to_writer(&file, events)?;
    // Ensure data is flushed to disk before returning
    file.sync_all()?;
    log::info!("Saved to {:?}", path);
    Ok(())
}
