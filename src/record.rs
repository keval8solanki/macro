use crate::event::SerializableEvent;
use anyhow::Result;
use rdev::{listen, Event, Key};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

pub fn run_record(output_path: PathBuf) -> Result<()> {
    println!("Recording... Press Esc to stop.");

    let events = Arc::new(Mutex::new(Vec::new()));
    let last_time = Arc::new(Mutex::new(SystemTime::now()));
    
    let events_clone = events.clone();
    let last_time_clone = last_time.clone();
    let output_path_clone = output_path.clone();

    let callback = move |event: Event| {
        let mut last_time_guard = last_time_clone.lock().unwrap();
        let now = SystemTime::now();
        let delay = now.duration_since(*last_time_guard).unwrap().as_millis() as u64;
        *last_time_guard = now;

        if let Some(serializable_event) = SerializableEvent::from_rdev(event.clone(), delay) {
            events_clone.lock().unwrap().push(serializable_event);
        }

        if let rdev::EventType::KeyPress(Key::Escape) = event.event_type {
            println!("Stopping recording...");
            let file = File::create(&output_path_clone).expect("Failed to create file");
            let events_guard = events_clone.lock().unwrap();
            serde_json::to_writer_pretty(file, &*events_guard).expect("Failed to write to file");
            println!("Saved to {:?}", output_path_clone);
            std::process::exit(0);
        }
    };

    if let Err(error) = listen(callback) {
        println!("Error: {:?}", error);
        return Err(anyhow::anyhow!("Listen error: {:?}", error));
    }

    Ok(())
}
