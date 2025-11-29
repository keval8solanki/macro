mod config;
mod event;
mod play;
mod record;

use anyhow::Result;
use clap::{Parser, Subcommand};
use cliclack::{confirm, intro, log, outro, select, input};
use config::{GlobalConfig, WorkspaceConfig};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;
use std::env;
use chrono::Local;

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
    let cli = Cli::parse();

    // If arguments are provided, run the command directly (used for child process or direct usage)
    if let Some(command) = cli.command {
        // We still need to load config for direct commands to get keymaps/paths
        let global_config = config::load_global_config()?;
        let workspace_config = if let Some(gc) = global_config {
             config::load_workspace_config(&gc.workspace_path)?
        } else {
             // If no config, we can't really run record/play effectively without workspace context
             // But for now, let's just error out if not configured for direct commands
             // Or we could try to run with defaults if that makes sense, but the requirement implies workspace usage.
             return Err(anyhow::anyhow!("CLI is not configured. Run without arguments to configure."));
        };

        match command {
            Commands::Record { output } => {
                // Ensure recording directory exists if we are using relative path
                let recording_dir = workspace_config.path.join("recording");
                std::fs::create_dir_all(&recording_dir)?;

                let final_path = if output.is_absolute() {
                    output
                } else {
                    recording_dir.join(output)
                };

                record::run_record(final_path, workspace_config.keymaps)?;
            }
            Commands::Play { input, speed, repeat_count } => {
                 play::run_play(input, speed, repeat_count, workspace_config.keymaps)?;
            }
        }
        return Ok(());
    }

    // Interactive Mode
    intro("Event Replay CLI")?;

    loop {
        // Load config each loop to ensure we have latest
        let global_config = config::load_global_config()?;
        
        let action = select("Select an option")
            .item("record", "Record", "")
            .item("play", "Play", "")
            .item("config", "Config", "")
            .item("exit", "Exit", "")
            .interact()?;

        match action {
            "record" => {
                if let Some(gc) = global_config {
                    let workspace_config = config::load_workspace_config(&gc.workspace_path)?;
                    handle_record(&workspace_config)?;
                } else {
                    log::error("Please configure the workspace first.")?;
                    handle_config()?;
                }
            }
            "play" => {
                 if let Some(gc) = global_config {
                    let workspace_config = config::load_workspace_config(&gc.workspace_path)?;
                    handle_play(&workspace_config)?;
                } else {
                    log::error("Please configure the workspace first.")?;
                    handle_config()?;
                }
            }
            "config" => {
                handle_config()?;
            }
            "exit" => {
                break;
            }
            _ => {}
        }
    }

    outro("Bye!")?;
    Ok(())
}

fn handle_record(workspace_config: &WorkspaceConfig) -> Result<()> {
    loop {
        let default_name = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        
        let mut filename: String = input("Enter recording name (without extension)")
            .placeholder(&default_name)
            .required(false)
            .interact()?;

        if filename.trim().is_empty() {
            filename = default_name;
        }

        if !filename.ends_with(".json") {
            filename.push_str(".json");
        }

        let recording_dir = workspace_config.path.join("recording");
        std::fs::create_dir_all(&recording_dir)?;
        let file_path = recording_dir.join(&filename);

        log::info(format!("Preparing to record to: {:?}", file_path))?;
        log::info("Recording will start in a separate process.")?;
        
        // Spawn child process
        let exe_path = env::current_exe()?;
        let mut child = ProcessCommand::new(exe_path)
            .arg("record")
            .arg(file_path.to_str().unwrap())
            .spawn()?;

        let status = child.wait()?;

        if status.success() {
            log::success("Recording saved successfully.")?;
        } else {
            log::error("Recording process failed or was interrupted.")?;
        }

        let record_again = confirm("Do you want to record another?").interact()?;
        if !record_again {
            break;
        }
    }
    Ok(())
}

fn handle_play(workspace_config: &WorkspaceConfig) -> Result<()> {
    let recording_dir = workspace_config.path.join("recording");
    if !recording_dir.exists() {
        std::fs::create_dir_all(&recording_dir)?;
    }

    loop {
        log::info("Opening file picker...")?;
        let file = rfd::FileDialog::new()
            .set_directory(&recording_dir)
            .add_filter("JSON", &["json"])
            .pick_file();

        if let Some(path) = file {
            log::info(format!("Selected: {:?}", path))?;
            
            let speed: f64 = input("Playback speed")
                .default_input("1.0")
                .interact()?;
                
            let repeat: u32 = input("Repeat count (0 for infinite)")
                .default_input("1")
                .interact()?;

            log::info("Playback will start in a separate process.")?;
            
            // Spawn child process
            let exe_path = env::current_exe()?;
            let mut child = ProcessCommand::new(exe_path)
                .arg("play")
                .arg(path.to_str().unwrap())
                .arg("--speed")
                .arg(speed.to_string())
                .arg("--repeat-count")
                .arg(repeat.to_string())
                .spawn()?;

            let status = child.wait()?;

            if status.success() {
                log::success("Playback finished.")?;
            } else {
                log::error("Playback process failed or was interrupted.")?;
            }
            
            let play_again = confirm("Do you want to play another?").interact()?;
            if !play_again {
                break;
            }
        } else {
            log::warning("No file selected.")?;
            break;
        }
    }

    Ok(())
}

fn handle_config() -> Result<()> {
    log::info("Opening folder picker for workspace...")?;
    let folder = rfd::FileDialog::new().pick_folder();

    if let Some(path) = folder {
        log::success(format!("Selected folder: {:?}", path))?;
        let _ = config::create_workspace(path.clone())?;
        config::save_global_config(&GlobalConfig {
            workspace_path: path,
        })?;
        log::success("Configuration saved.")?;
    } else {
        log::warning("No folder selected.")?;
    }
    Ok(())
}
