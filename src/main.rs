mod event;
mod play;
mod record;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Record mouse and keyboard events
    Record {
        /// Output file path
        #[arg(default_value = "events.json")]
        output: PathBuf,
    },
    /// Play back recorded events
    Play {
        /// Input file path
        #[arg(default_value = "events.json")]
        input: PathBuf,
        /// Playback speed factor (e.g., 2.0 for 2x speed, 0.5 for half speed)
        #[arg(long, default_value_t = 1.0)]
        speed: f64,
        /// Number of times to repeat playback (0 for infinite)
        #[arg(long, default_value_t = 1)]
        repeat_count: u32,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Record { output } => {
            record::run_record(output)?;
        }
        Commands::Play { input, speed, repeat_count } => {
            play::run_play(input, speed, repeat_count)?;
        }
    }

    Ok(())
}
