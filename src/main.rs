use anyhow::Result;
use clap::{Parser, Subcommand};
use macro_lib::config;
use macro_lib::{play, record};
use std::path::PathBuf;

mod bar_app;
use bar_app::BarApp;

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
        /// Internal flag to start recording immediately without waiting for hotkey
        #[arg(long, default_value_t = false, hide = true)]
        immediate: bool,
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
        /// Internal flag to start playback immediately without waiting for hotkey
        #[arg(long, default_value_t = false, hide = true)]
        immediate: bool,
    },
}

fn main() -> Result<()> {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    log::info!("Launched with args: {:?}", args);
    let cli = Cli::parse();

    if let Some(command) = cli.command {
        // CLI / Worker Mode
        // We still need to load config for direct commands to get keymaps/paths
        let global_config = config::load_global_config()?;
        let workspace_config = if let Some(gc) = global_config {
            config::load_workspace_config(&gc.workspace_path).unwrap_or_else(|_| {
                config::WorkspaceConfig {
                    path: std::env::current_dir().unwrap_or_default(),
                    keymaps: config::KeyMaps::default(),
                }
            })
        } else {
            config::WorkspaceConfig {
                path: std::env::current_dir().unwrap_or_default(),
                keymaps: config::KeyMaps::default(),
            }
        };

        match command {
            Commands::Record { output, immediate } => {
                // Ensure recording directory exists if we are using relative path
                let recording_dir = workspace_config.path.join("recording");
                std::fs::create_dir_all(&recording_dir)?;

                let final_path = if output.is_absolute() {
                    output
                } else {
                    recording_dir.join(output)
                };

                record::run_record(final_path, workspace_config.keymaps, immediate)?;
            }
            Commands::Play {
                input,
                speed,
                repeat_count,
                immediate,
            } => {
                play::run_play(input, speed, repeat_count, workspace_config.keymaps, immediate)?;
            }
        }
    } else {
        // GUI Mode
        log::info!("Starting macro-bar (eframe)...");

        let options = eframe::NativeOptions {
             viewport: eframe::egui::ViewportBuilder::default()
                 .with_visible(false) // Start hidden (tray only initially)
                 .with_inner_size([400.0, 300.0]),
             event_loop_builder: Some(Box::new(|builder| {
                 #[cfg(target_os = "macos")]
                 {
                     use winit::platform::macos::EventLoopBuilderExtMacOS;
                     builder.with_activation_policy(winit::platform::macos::ActivationPolicy::Accessory);
                 }
             })),
             ..Default::default()
        };

        eframe::run_native(
            "Macro",
            options,
            Box::new(|cc| Ok(Box::new(BarApp::new(cc)))),
        ).map_err(|e| anyhow::anyhow!("Eframe error: {}", e))?;
    }

    Ok(())
}
