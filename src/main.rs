mod config;
mod event;
mod play;
mod record;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cliclack::{confirm, intro, log, outro, spinner};
use config::GlobalConfig;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
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
    // Initialize cliclack
    intro("Event Replay CLI")?;

    let cli = Cli::parse();

    // Check configuration
    let global_config = config::load_global_config()?;

    let workspace_config = match global_config {
        Some(gc) => {
            if !gc.workspace_path.exists() {
                log::warning(format!(
                    "Configured workspace path {:?} does not exist.",
                    gc.workspace_path
                ))?;
            }
            config::load_workspace_config(&gc.workspace_path)?
        }
        None => {
            log::info("CLI is not configured.")?;
            let should_configure = confirm("Do you want to select a workspace folder?").interact()?;

            if !should_configure {
                log::error("Configuration required to proceed.")?;
                return Ok(());
            }

            let spinner = spinner();
            spinner.start("Waiting for folder selection...");

            // We need to ensure we are on the main thread for some OSs, but for simple dialogs it might be fine.
            // On Mac, rfd should work.
            let folder = rfd::FileDialog::new().pick_folder();
            
            spinner.stop("Folder selection completed.");

            if let Some(path) = folder {
                log::success(format!("Selected folder: {:?}", path))?;
                let config = config::create_workspace(path.clone())?;
                config::save_global_config(&GlobalConfig {
                    workspace_path: path,
                })?;
                config
            } else {
                log::error("No folder selected. Exiting.")?;
                return Ok(());
            }
        }
    };

    match cli.command {
        Some(Commands::Record { output }) => {
            let recording_dir = workspace_config.path.join("recording");
            std::fs::create_dir_all(&recording_dir)?;

            let final_path = if output.is_absolute() {
                output
            } else {
                recording_dir.join(output)
            };

            record::run_record(final_path, workspace_config.keymaps)?;
        }
        Some(Commands::Play {
            input,
            speed,
            repeat_count,
        }) => {
            play::run_play(input, speed, repeat_count)?;
        }
        None => {
            log::info(format!(
                "Workspace configured at: {:?}",
                workspace_config.path
            ))?;
            log::info("Use 'record' to start recording or 'play' to play back events.")?;
        }
    }

    outro("Done")?;
    Ok(())
}
