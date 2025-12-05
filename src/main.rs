use anyhow::Result;
use clap::{Parser, Subcommand};
use global_hotkey::GlobalHotKeyManager;
use macro_lib::config;
use macro_lib::{play, record};
use std::path::PathBuf;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};

mod bar_app;
use bar_app::{AppEvent, BarApp};

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
        log::info!("Starting macro-bar...");

        let mut event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
        event_loop.set_activation_policy(ActivationPolicy::Accessory);

        let proxy = event_loop.create_proxy();

        // Global Hotkey Manager
        let hotkey_manager = GlobalHotKeyManager::new().unwrap();
        let (record_hotkey, playback_hotkey) = bar_app::create_hotkeys();
        hotkey_manager.register(record_hotkey).unwrap();
        hotkey_manager.register(playback_hotkey).unwrap();

        // Initialize App
        let mut app = BarApp::new(proxy)?;

        event_loop.run(move |event, _, control_flow| {
            // Poll every 100ms to check child process status
            *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(100));

            match event {
                tao::event::Event::UserEvent(app_event) => match app_event {
                    AppEvent::Hotkey(event) => {
                        app.handle_hotkey(event);
                    }
                    AppEvent::Menu(event) => {
                        app.handle_menu_event(event, control_flow);
                    }
                },
                tao::event::Event::MainEventsCleared => {
                    // Check if playback process has finished
                    app.check_playback_status();
                }
                _ => {}
            }
        });
    }

    Ok(())
}
