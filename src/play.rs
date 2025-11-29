use crate::event::SerializableEvent;
use anyhow::Result;
use cliclack::log;
use rdev::simulate;
use std::fs::File;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub fn run_play(input_path: PathBuf, speed: f64, repeat_count: u32) -> Result<()> {
    log::info(format!("Playing back from {:?}...", input_path))?;
    if speed != 1.0 {
        log::info(format!("Playback speed: {:.2}x", speed))?;
    }
    if repeat_count == 0 {
        log::info("Repeat: Infinite")?;
    } else if repeat_count > 1 {
        log::info(format!("Repeat: {} times", repeat_count))?;
    }

    let file = File::open(input_path)?;
    let events: Vec<SerializableEvent> = serde_json::from_reader(file)?;

    let mut count = 0;
    loop {
        if repeat_count > 0 && count >= repeat_count {
            break;
        }
        if count > 0 {
             log::info(format!("Repeat #{}", count + 1))?;
        }

        for event in &events {
            // Adjust delay based on speed
            // If speed is 2.0, delay should be half.
            // If speed is 0.5, delay should be double.
            let delay = (event.delay_ms as f64 / speed) as u64;
            thread::sleep(Duration::from_millis(delay));
            let rdev_event_type = event.to_rdev();
            match simulate(&rdev_event_type) {
                Ok(()) => (),
                Err(e) => log::error(format!("We could not send {:?}: {:?}", rdev_event_type, e))?,
            }
        }
        count += 1;
    }

    log::success("Playback complete.")?;
    Ok(())
}
