mod config;
mod event;
mod play;
mod record;

use tauri::{Manager, State};
use std::sync::{Arc, Mutex};
use crate::record::RecorderState;

struct AppState {
    recorder: Arc<Mutex<RecorderState>>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn start_recording(state: State<AppState>, filename: String) -> Result<(), String> {
    let mut recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    if recorder.is_recording {
        return Err("Already recording".to_string());
    }
    recorder.is_recording = true;
    recorder.filename = Some(filename);
    recorder.events.clear();
    recorder.last_time = std::time::SystemTime::now();
    println!("Started recording");
    Ok(())
}

#[tauri::command]
fn stop_recording(state: State<AppState>) -> Result<String, String> {
    let mut recorder = state.recorder.lock().map_err(|e| e.to_string())?;
    if !recorder.is_recording {
        return Err("Not recording".to_string());
    }
    recorder.is_recording = false;
    
    let filename = recorder.filename.clone().unwrap_or_else(|| "default".to_string());
    // Save to "recordings" directory
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("recordings");
    if !path.exists() {
        std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }
    path.push(format!("{}.json", filename));
    
    let events = recorder.events.clone();
    
    let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    serde_json::to_writer_pretty(file, &events).map_err(|e| e.to_string())?;
    
    println!("Saved recording to {:?}", path);
    Ok(format!("Saved to {:?}", path))
}

#[tauri::command]
fn play_macro(filename: String) -> Result<(), String> {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("recordings");
    path.push(format!("{}.json", filename));
    
    let file = std::fs::File::open(&path).map_err(|e| e.to_string())?;
    let events: Vec<crate::event::SerializableEvent> = serde_json::from_reader(file).map_err(|e| e.to_string())?;
    
    let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    
    std::thread::spawn(move || {
        play::do_playback(&events, 1.0, 1, stop_flag);
    });
    
    Ok(())
}

#[tauri::command]
fn get_recordings() -> Result<Vec<String>, String> {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push("recordings");
    if !path.exists() {
        return Ok(Vec::new());
    }
    
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                files.push(stem.to_string());
            }
        }
    }
    Ok(files)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let recorder_state = Arc::new(Mutex::new(RecorderState::new()));
    let recorder_state_clone = recorder_state.clone();

    // Spawn listener thread
    std::thread::spawn(move || {
        if let Err(e) = record::listen_loop(recorder_state_clone) {
            eprintln!("Listener error: {:?}", e);
        }
    });

    tauri::Builder::default()
        .manage(AppState { recorder: recorder_state })
        .invoke_handler(tauri::generate_handler![greet, start_recording, stop_recording, play_macro, get_recordings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
