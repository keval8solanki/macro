use crate::event::SerializableEvent;
use anyhow::Result;
use rdev::simulate;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub fn run_play(input_path: PathBuf) -> Result<()> {
    println!("Playing back from {:?}...", input_path);

    let file = File::open(input_path)?;
    let events: Vec<SerializableEvent> = serde_json::from_reader(file)?;

    for event in events {
        thread::sleep(Duration::from_millis(event.delay_ms));
        let rdev_event_type = event.to_rdev();
        match simulate(&rdev_event_type) {
            Ok(()) => (),
            Err(e) => println!("We could not send {:?}: {:?}", rdev_event_type, e),
        }
    }

    println!("Playback complete.");
    Ok(())
}
