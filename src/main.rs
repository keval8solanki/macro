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
        /// Interval between repeats in seconds
        #[arg(long, default_value_t = 0.0)]
        repeat_interval: f64,
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
        let keymaps = config::KeyMaps::default();

        match command {
            Commands::Record { output, immediate } => {
                let final_path = if output.is_absolute() {
                    output
                } else {
                    std::env::current_dir()?.join(output)
                };

                // Ensure parent directory exists
                if let Some(parent) = final_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                record::run_record(final_path, keymaps, immediate)?;
            }
            Commands::Play {
                input,
                speed,
                repeat_count,
                repeat_interval,
                immediate,
            } => {
                play::run_play(input, speed, repeat_count, repeat_interval, keymaps, immediate)?;
            }
        }
    } else {
        // GUI Mode
        log::info!("Starting Macro...");

        let mut event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
        event_loop.set_activation_policy(ActivationPolicy::Accessory);

        let proxy = event_loop.create_proxy();

        // Global Hotkey Manager
        let hotkey_manager = GlobalHotKeyManager::new().unwrap();
        let (record_hotkey, playback_hotkey, load_hotkey) = bar_app::create_hotkeys();
        hotkey_manager.register(record_hotkey).unwrap();
        hotkey_manager.register(playback_hotkey).unwrap();
        hotkey_manager.register(load_hotkey).unwrap();

        // Initialize App
        let mut app = BarApp::new(proxy)?;

        event_loop.run(move |event, event_loop, control_flow| {
            // Poll every 100ms to check child process status
            *control_flow = ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(100));

            match event {
                tao::event::Event::UserEvent(app_event) => match app_event {
                    AppEvent::GlobalHotkeyEvent(event) => {
                        app.handle_hotkey(event, event_loop);
                    }
                    AppEvent::MenuEvent(event) => {
                        app.handle_menu_event(event, event_loop, control_flow);
                    }
                    AppEvent::SettingsApplied(settings) => {
                        app.handle_settings_applied(settings);
                    }
                },
                tao::event::Event::WindowEvent { event: tao::event::WindowEvent::CloseRequested, .. } => {
                    app.handle_window_close();
                }
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
