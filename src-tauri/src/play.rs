use crate::event::SerializableEvent;
use crate::config::{KeyMaps, Modifier};
use anyhow::Result;
// use cliclack::log;
use rdev::{listen, simulate, EventType, Key};
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use std::env;

pub fn run_play(input_path: PathBuf, speed: f64, repeat_count: u32, keymaps: KeyMaps, immediate: bool) -> Result<()> {
    println!("Preparing to play back from {:?}...", input_path);
    
    // Load events first to ensure file exists and is valid
    let file = File::open(&input_path)?;
    let events: Vec<SerializableEvent> = serde_json::from_reader(file)?;
    println!("Loaded {} events.", events.len());

    if speed != 1.0 {
        println!("Playback speed: {:.2}x", speed);
    }
    if repeat_count == 0 {
        println!("Repeat: Infinite");
    } else if repeat_count > 1 {
        println!("Repeat: {} times", repeat_count);
    }

    if immediate {
        println!("Starting playback immediately...");
        println!("Stop Playback: {:?} + {:?}", keymaps.stop_playback.modifiers, keymaps.stop_playback.trigger);
        
        // Shared flag to stop playback
        let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        
        // Spawn a thread for playback
        let events_for_thread = events.clone();
        let stop_flag_play = stop_flag.clone();
        thread::spawn(move || {
            do_playback(&events_for_thread, speed, repeat_count, stop_flag_play);
            // std::process::exit(0); // Removed exit
        });

        // Listen for stop hotkey
        let stop_flag_listen = stop_flag.clone();
        let keymaps_clone = keymaps.clone();
        
        struct StopState {
            cmd_pressed: bool,
            alt_pressed: bool,
            ctrl_pressed: bool,
            shift_pressed: bool,
        }

        let state = Arc::new(Mutex::new(StopState {
            cmd_pressed: false,
            alt_pressed: false,
            ctrl_pressed: false,
            shift_pressed: false,
        }));

        let state_clone = state.clone();
        
        if let Err(error) = listen(move |event| {
            let mut state = state_clone.lock().unwrap();

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

            // Check stop hotkey
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
                if key == keymaps_clone.stop_playback.trigger && check_modifiers(&keymaps_clone.stop_playback.modifiers) {
                    println!("Stop hotkey detected. Stopping playback...");
                    stop_flag_listen.store(true, std::sync::atomic::Ordering::SeqCst);
                    // std::process::exit(0); // Removed exit
                }
            }
        }) {
             eprintln!("Error: {:?}", error);
        }
        return Ok(());
    }

    println!("Waiting for start hotkey: {:?} + {:?}", keymaps.start_playback.modifiers, keymaps.start_playback.trigger);
    
    // For now, just return as we don't support the old hotkey wait loop in the same way yet
    Ok(())
}

pub fn do_playback(events: &[SerializableEvent], speed: f64, repeat_count: u32, stop_flag: Arc<std::sync::atomic::AtomicBool>) {
    let mut count = 0;
    loop {
        if repeat_count > 0 && count >= repeat_count {
            break;
        }
        if count > 0 {
             println!("Repeat #{}", count + 1);
        }

        for event in events {
            // Check if stop was requested
            if stop_flag.load(std::sync::atomic::Ordering::SeqCst) {
                println!("Playback stopped by user.");
                return;
            }
            
            // Adjust delay based on speed
            let delay = (event.delay_ms as f64 / speed) as u64;
            thread::sleep(Duration::from_millis(delay));
            let rdev_event_type = event.to_rdev();
            match simulate(&rdev_event_type) {
                Ok(()) => (),
                Err(e) => {
                    eprintln!("We could not send {:?}: {:?}", rdev_event_type, e);
                }
            }
        }
        count += 1;
    }
    println!("Playback complete.");
}
