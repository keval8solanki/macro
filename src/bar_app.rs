use anyhow::Result;
use chrono::Local;
use dirs::document_dir;
use global_hotkey::GlobalHotKeyEvent;
use global_hotkey::hotkey::{HotKey, Modifiers, Code};

use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};

use tao::event_loop::{ControlFlow, EventLoopProxy};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, Submenu, CheckMenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIcon};

use self_update::cargo_crate_version;


#[derive(Debug)]
pub enum AppEvent {
    Menu(MenuEvent),
    Hotkey(GlobalHotKeyEvent),
}

pub struct AppState {
    pub is_recording: bool,
    pub recording_process: Option<Child>,
    pub playback_process: Option<Child>,
    pub playback_speed: f64,
    pub repeat_count: u32,
    pub pending_playback: Option<PathBuf>,
    pub current_recording_path: Option<PathBuf>,
    pub last_record_hotkey_pressed: bool,
    pub last_playback_hotkey_pressed: bool,
}



pub struct BarApp {
    pub state: Arc<Mutex<AppState>>,
    pub tray_icon: Option<TrayIcon>,
    pub recording_menu_item: MenuItem,
    pub playback_menu_item: MenuItem,
    pub load_menu_item: MenuItem,
    pub settings_menu: Submenu,
    pub speed_05: CheckMenuItem,
    pub speed_10: CheckMenuItem,
    pub speed_20: CheckMenuItem,
    pub repeat_1: CheckMenuItem,
    pub repeat_inf: CheckMenuItem,
    pub quit_i: MenuItem,
    pub icon_idle: Icon,
    pub icon_recording: Icon,
    pub icon_playing: Icon,
    pub icon_armed: Icon,
    pub record_hotkey: HotKey,
    pub playback_hotkey: HotKey,
    pub check_updates_item: MenuItem,
}

impl BarApp {
    pub fn new(proxy: EventLoopProxy<AppEvent>) -> Result<Self> {
        // Icons
        let icon_idle = create_icon(255, 255, 255, 255); // White
        let icon_recording = create_icon(255, 86, 86, 255); // #FF5656
        let icon_playing = create_icon(115, 175, 111, 255); // #73AF6F
        let icon_armed = create_icon(255, 162, 57, 255); // #FFA239

        // Menu
        let tray_menu = Menu::new();
        let app_title_item = MenuItem::new(concat!("Macro ", env!("CARGO_PKG_VERSION")), false, None);
        let recording_menu_item = MenuItem::new("Record", true, None);
        let playback_menu_item = MenuItem::new("Play", false, None); // Disabled by default
        let load_menu_item = MenuItem::new("Load", true, None);
        
        // Settings Menu
        let settings_menu = Submenu::new("Settings", false); // Disabled by default
        
        let speed_menu = Submenu::new("Speed", true);
        let speed_05 = CheckMenuItem::new("0.5x", true, false, None);
        let speed_10 = CheckMenuItem::new("1.0x", true, true, None); // Default
        let speed_20 = CheckMenuItem::new("2.0x", true, false, None);
        speed_menu.append(&speed_05)?;
        speed_menu.append(&speed_10)?;
        speed_menu.append(&speed_20)?;
        
        let repeat_menu = Submenu::new("Repeat", true);
        let repeat_1 = CheckMenuItem::new("1x", true, true, None); // Default
        let repeat_inf = CheckMenuItem::new("Infinite", true, false, None);
        repeat_menu.append(&repeat_1)?;
        repeat_menu.append(&repeat_inf)?;
        
        settings_menu.append(&speed_menu)?;
        settings_menu.append(&repeat_menu)?;

        let quit_i = MenuItem::new("Quit", true, None);
        let check_updates_item = MenuItem::new("Check for Updates...", true, None);

        tray_menu.append(&app_title_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&recording_menu_item)?;
        tray_menu.append(&playback_menu_item)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&load_menu_item)?;
        tray_menu.append(&settings_menu)?;
        tray_menu.append(&PredefinedMenuItem::separator())?;
        tray_menu.append(&check_updates_item)?;
        tray_menu.append(&quit_i)?;

        let tray_icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu.clone()))
                .with_tooltip("Macro")
                .with_icon(icon_idle.clone())
                .build()?,
        );

        // Shared state
        let state = Arc::new(Mutex::new(AppState {
            is_recording: false,
            recording_process: None,
            playback_process: None,
            playback_speed: 1.0,
            repeat_count: 1,
            pending_playback: None,
            current_recording_path: None,
            last_record_hotkey_pressed: false,
            last_playback_hotkey_pressed: false,
        }));

        // Listen for menu events in a separate thread
        let proxy_menu = proxy.clone();
        std::thread::spawn(move || {
            while let Ok(event) = MenuEvent::receiver().recv() {
                let _ = proxy_menu.send_event(AppEvent::Menu(event));
            }
        });

        // Listen for hotkey events in a separate thread
        let proxy_hotkey = proxy.clone();
        std::thread::spawn(move || {
            while let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
                let _ = proxy_hotkey.send_event(AppEvent::Hotkey(event));
            }
        });

        let (record_hotkey, playback_hotkey) = create_hotkeys();

        Ok(Self {
            state,
            tray_icon,
            recording_menu_item,
            playback_menu_item,
            load_menu_item,
            settings_menu,
            speed_05,
            speed_10,
            speed_20,
            repeat_1,
            repeat_inf,
            quit_i,
            icon_idle,
            icon_recording,
            icon_playing,
            icon_armed,
            record_hotkey,
            playback_hotkey,
            check_updates_item,
        })
    }

    pub fn handle_hotkey(&mut self, event: GlobalHotKeyEvent) {
        let mut state = self.state.lock().unwrap();
        
        // Check if this is a press event (state change from not pressed to pressed)
        if event.id == self.record_hotkey.id() {
            // Event state: HotKeyState::Pressed or HotKeyState::Released
            let is_pressed = event.state == global_hotkey::HotKeyState::Pressed;
            
            // Only trigger on press event (transition from not pressed to pressed)
            if is_pressed && !state.last_record_hotkey_pressed {
                state.last_record_hotkey_pressed = true;
                drop(state); // Release lock before calling handler
                self.handle_toggle_recording();
            } else if !is_pressed {
                state.last_record_hotkey_pressed = false;
            }
        } else if event.id == self.playback_hotkey.id() {
            let is_pressed = event.state == global_hotkey::HotKeyState::Pressed;
            
            // Only trigger on press event (transition from not pressed to pressed)
            if is_pressed && !state.last_playback_hotkey_pressed {
                state.last_playback_hotkey_pressed = true;
                drop(state); // Release lock before calling handler
                self.handle_toggle_playback();
            } else if !is_pressed {
                state.last_playback_hotkey_pressed = false;
            }
        }
    }

    pub fn handle_file_selected(&mut self, path: PathBuf) {
        let mut state = self.state.lock().unwrap();
        state.pending_playback = Some(path);
        drop(state);
        
        self.update_menu_state();
    }


    pub fn handle_toggle_playback(&mut self) {
        let mut state = self.state.lock().unwrap();

        // If playback is running, stop it
        if let Some(mut child) = state.playback_process.take() {
            log::info!("Stopping playback...");
            let _ = child.kill();
            let _ = child.wait();
            
            // Reset icon and menu text
            drop(state);
            self.update_menu_state();
            return;
        }

        // If no playback running, check if we have a pending playback to start
        if let Some(path) = &state.pending_playback {
            log::info!("Starting playback of: {:?}", path);
            
            // Spawn `macro play` (self)
            let macro_bin = std::env::current_exe().unwrap();
            
            let (speed, repeat) = (state.playback_speed, state.repeat_count);

            let child = Command::new(macro_bin)
                .arg("play")
                .arg(path)
                .arg("--speed")
                .arg(speed.to_string())
                .arg("--repeat-count")
                .arg(repeat.to_string())
                .arg("--immediate")
                .spawn();
            
            log::info!("Spawned playback process: {:?}", child);
                
            if let Ok(child) = child {
                state.playback_process = Some(child);
                drop(state);
                self.update_menu_state();
            } else {
                drop(state);
            }
        } else {
            log::warn!("No recording selected for playback.");
        }
    }

    pub fn handle_toggle_recording(&mut self) {
        let mut state = self.state.lock().unwrap();
        
        // If playback is running, we cannot record
        if state.playback_process.is_some() {
            log::warn!("Cannot start recording while playback is active.");
            return;
        }

        // If we are recording, stop it
        if state.is_recording {
            log::info!("Stopping recording...");
            state.is_recording = false;
            
            // Kill the child process gracefully
            if let Some(mut child) = state.recording_process.take() {
                let pid = child.id();
                
                // Check if it has already exited (it should have if it caught the hotkey)
                match child.try_wait() {
                    Ok(Some(status)) => {
                        log::info!("Child process already exited with: {:?}", status);
                    }
                    Ok(None) => {
                        log::info!("Child process still running. Waiting for it to exit...");
                        // Wait a bit for it to exit on its own
                        let start = std::time::Instant::now();
                        let mut exited = false;
                        while start.elapsed() < std::time::Duration::from_millis(1000) {
                            if let Ok(Some(status)) = child.try_wait() {
                                log::info!("Child process exited gracefully with: {:?}", status);
                                exited = true;
                                break;
                            }
                            std::thread::sleep(std::time::Duration::from_millis(50));
                        }
                        
                        if !exited {
                             log::info!("Child process did not exit. Sending SIGTERM...");
                             // Send SIGTERM (15) to allow graceful shutdown and saving
                            let kill_output = Command::new("kill")
                                .arg("-15")
                                .arg(pid.to_string())
                                .output();
                            
                            match kill_output {
                                Ok(output) => log::info!("Kill command output: {:?}", output),
                                Err(e) => log::error!("Failed to execute kill command: {}", e),
                            }
                            
                            // Wait for it to finish
                            let exit_status = child.wait();
                            log::info!("Child process exited with: {:?}", exit_status);
                        }
                    }
                    Err(e) => {
                        log::error!("Error waiting for child process: {}", e);
                         let _ = child.kill();
                    }
                }
                
                // Give the process a moment to flush and close the file
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            // Handle file saving - extract path before releasing the lock
            let temp_path = state.current_recording_path.take();
            
            // Release the lock before opening the file picker
            drop(state);
            
            // Update UI state
            self.update_menu_state();

            // Handle file saving after releasing the lock
            if let Some(temp_path) = temp_path {
                // Verify the temp file exists
                if !temp_path.exists() {
                    log::error!("Temp recording file not found at: {:?}", temp_path);
                    return;
                }
                
                // Run file picker on the main thread
                let recording_dir = get_recordings_dir();
                let default_name = format!("recording_{}.json", Local::now().format("%Y%m%d_%H%M%S"));
                
                log::info!("Opening file picker to save recording...");
                
                let file_handle = rfd::FileDialog::new()
                    .set_directory(&recording_dir)
                    .set_file_name(&default_name)
                    .add_filter("JSON", &["json"])
                    .save_file();
                
                if let Some(target_path) = file_handle {
                    log::info!("Saving recording to: {:?}", target_path);
                    if let Err(e) = fs::rename(&temp_path, &target_path) {
                        log::error!("Failed to save recording (rename failed): {}", e);
                        // Try copying if rename fails (cross-device link error)
                        if let Err(e) = fs::copy(&temp_path, &target_path) {
                             log::error!("Failed to save recording (copy failed): {}", e);
                        } else {
                             let _ = fs::remove_file(&temp_path);
                             log::info!("Recording saved successfully (copied)");
                             
                             // Do not auto-load. Just update UI.
                             self.update_menu_state();
                        }
                    } else {
                        log::info!("Recording saved successfully");
                        
                        // Do not auto-load. Just update UI.
                        self.update_menu_state();
                    }
                } else {
                    log::info!("Save canceled. Discarding recording.");
                    let _ = fs::remove_file(&temp_path);
                }
            }

        } else {
            // Start Recording
            log::info!("Starting recording...");
            state.is_recording = true;
            // Clear any pending playback so we don't return to "loaded" state after this recording
            state.pending_playback = None;
            
            // Use a temporary file for recording
            let temp_dir = std::env::temp_dir();
            let filename = format!("macro_recording_{}.json", Local::now().format("%Y%m%d_%H%M%S"));
            let path = temp_dir.join(filename);
            
            log::info!("Recording to temp file: {:?}", path);
            state.current_recording_path = Some(path.clone());

            // Spawn `macro record` (self)
            let macro_bin = std::env::current_exe().unwrap();

            let child = Command::new(macro_bin)
                .arg("record")
                .arg(path)
                .arg("--immediate")
                .spawn();

            log::info!("Spawned recording process: {:?}", child);

            match child {
                Ok(child) => {
                    state.recording_process = Some(child);
                    drop(state);
                    self.update_menu_state();
                }
                Err(e) => {
                    log::error!("Failed to spawn macro record: {}", e);
                    state.is_recording = false;
                    state.current_recording_path = None;
                    drop(state);
                    self.update_menu_state();
                }
            }
        }
    }

    pub fn handle_menu_event(&mut self, event: MenuEvent, control_flow: &mut ControlFlow) {
        if event.id == self.quit_i.id() {
            // Cleanup
            let mut state = self.state.lock().unwrap();
            if let Some(mut child) = state.recording_process.take() {
                let _ = child.kill();
            }
            if let Some(mut child) = state.playback_process.take() {
                let _ = child.kill();
            }
            *control_flow = ControlFlow::Exit;
        } else if event.id == self.recording_menu_item.id() {
            self.handle_toggle_recording();
        } else if event.id == self.playback_menu_item.id() {
            self.handle_toggle_playback();
        } else if event.id == self.load_menu_item.id() {
            // Check if we are loading or unloading
            let mut state = self.state.lock().unwrap();
            if state.pending_playback.is_some() {
                // Unload Recording
                log::info!("Unloading recording...");
                state.pending_playback = None;
                drop(state);
                self.update_menu_state();
            } else {
                // Load Recording
                drop(state);
                // Open File Picker - run on main thread
                let recording_dir = get_recordings_dir();
                
                log::info!("Opening file picker to load recording...");
                
                let file_handle = rfd::FileDialog::new()
                    .set_directory(&recording_dir)
                    .add_filter("JSON", &["json"])
                    .pick_file();
                
                if let Some(path) = file_handle {
                    log::info!("Selected recording: {:?}", path);
                    self.handle_file_selected(path);
                }
            }
        } else if event.id == self.speed_05.id() {
            let mut state = self.state.lock().unwrap();
            state.playback_speed = 0.5;
            drop(state);
            let _ = self.speed_05.set_checked(true);
            let _ = self.speed_10.set_checked(false);
            let _ = self.speed_20.set_checked(false);
        } else if event.id == self.speed_10.id() {
            let mut state = self.state.lock().unwrap();
            state.playback_speed = 1.0;
            drop(state);
            let _ = self.speed_05.set_checked(false);
            let _ = self.speed_10.set_checked(true);
            let _ = self.speed_20.set_checked(false);
        } else if event.id == self.speed_20.id() {
            let mut state = self.state.lock().unwrap();
            state.playback_speed = 2.0;
            drop(state);
            let _ = self.speed_05.set_checked(false);
            let _ = self.speed_10.set_checked(false);
            let _ = self.speed_20.set_checked(true);
        } else if event.id == self.repeat_1.id() {
            let mut state = self.state.lock().unwrap();
            state.repeat_count = 1;
            drop(state);
            let _ = self.repeat_1.set_checked(true);
            let _ = self.repeat_inf.set_checked(false);
        } else if event.id == self.repeat_inf.id() {
            let mut state = self.state.lock().unwrap();
            state.repeat_count = 0; // 0 means infinite
            drop(state);
            let _ = self.repeat_1.set_checked(false);
            let _ = self.repeat_inf.set_checked(true);
        } else if event.id == self.check_updates_item.id() {
             std::thread::spawn(|| {
                 check_and_update();
             });
        }
    }

    pub fn check_playback_status(&mut self) {
        let mut state = self.state.lock().unwrap();
        
        if let Some(mut child) = state.playback_process.take() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    log::info!("Playback finished with status: {:?}", status);
                    // Playback finished, reset UI
                    drop(state);
                    self.update_menu_state();
                }
                Ok(None) => {
                    // Still running, put it back
                    state.playback_process = Some(child);
                }
                Err(e) => {
                    log::error!("Error waiting for playback process: {}", e);
                    // Assume it's gone or broken, reset UI
                    drop(state);
                    self.update_menu_state();
                }
            }
        }
    }

    pub fn update_menu_state(&mut self) {
        let state = self.state.lock().unwrap();
        let is_recording = state.is_recording;
        let is_playing = state.playback_process.is_some();
        let has_recording = state.pending_playback.is_some();
        drop(state);

        if is_recording {
            // Recording Started
            let _ = self.recording_menu_item.set_text("Stop");
            let _ = self.recording_menu_item.set_enabled(true);
            
            let _ = self.playback_menu_item.set_text("Play");
            let _ = self.playback_menu_item.set_enabled(false);
            
            let _ = self.load_menu_item.set_text("Load");
            let _ = self.load_menu_item.set_enabled(false);
            
            let _ = self.settings_menu.set_enabled(false);
            
            if let Some(tray) = &mut self.tray_icon {
                let _ = tray.set_icon(Some(self.icon_recording.clone()));
            }
        } else if is_playing {
            // Playback Started
            let _ = self.recording_menu_item.set_text("Record");
            let _ = self.recording_menu_item.set_enabled(false);
            
            let _ = self.playback_menu_item.set_text("Stop");
            let _ = self.playback_menu_item.set_enabled(true);
            
            let _ = self.load_menu_item.set_text("Load");
            let _ = self.load_menu_item.set_enabled(false);
            
            let _ = self.settings_menu.set_enabled(false);
            
            if let Some(tray) = &mut self.tray_icon {
                let _ = tray.set_icon(Some(self.icon_playing.clone()));
            }
        } else if has_recording {
            // Recording Loaded
            let _ = self.recording_menu_item.set_text("Record");
            let _ = self.recording_menu_item.set_enabled(false);
            
            let _ = self.playback_menu_item.set_text("Play");
            let _ = self.playback_menu_item.set_enabled(true);
            
            let _ = self.load_menu_item.set_text("Unload");
            let _ = self.load_menu_item.set_enabled(true);
            
            let _ = self.settings_menu.set_enabled(true);
            
            if let Some(tray) = &mut self.tray_icon {
                let _ = tray.set_icon(Some(self.icon_armed.clone()));
            }
        } else {
            // Initial State / Unloaded
            let _ = self.recording_menu_item.set_text("Record");
            let _ = self.recording_menu_item.set_enabled(true);
            
            let _ = self.playback_menu_item.set_text("Play");
            let _ = self.playback_menu_item.set_enabled(false);
            
            let _ = self.load_menu_item.set_text("Load");
            let _ = self.load_menu_item.set_enabled(true);
            
            let _ = self.settings_menu.set_enabled(false);
            
            if let Some(tray) = &mut self.tray_icon {
                let _ = tray.set_icon(Some(self.icon_idle.clone()));
            }
        }
    }
}

pub fn create_hotkeys() -> (HotKey, HotKey) {
    let record_hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::Digit1);
    // We need to set the ID manually if possible, but HotKey::new generates a random ID or hashes it.
    // Actually GlobalHotKeyManager uses the ID from the HotKey struct.
    // We can't easily force an ID on `HotKey` struct from `global_hotkey` crate as fields are private or it's constructed via new.
    // Wait, `HotKey` struct in `global_hotkey` 0.5.0 might not allow setting ID directly if it's not exposed.
    // Let's check how we can identify them.
    // Ah, `HotKey` implements `PartialEq` and `Hash`. We can store the created hotkeys in `BarApp` and compare `event.id` with `hotkey.id()`.
    
    let playback_hotkey = HotKey::new(Some(Modifiers::META | Modifiers::SHIFT), Code::Digit2);
    
    (record_hotkey, playback_hotkey)
}

fn get_recordings_dir() -> PathBuf {
    document_dir().unwrap_or(PathBuf::from(".")).join("Macros")
}



fn create_icon(r: u8, g: u8, b: u8, a: u8) -> Icon {
    let width = 22;
    let height = 22;
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    let radius = (width as f32 / 2.0) - 3.0; // Smaller circle

    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x + 0.5; // +0.5 to center in pixel
            let dy = y as f32 - center_y + 0.5;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance <= radius {
                rgba.push(r);
                rgba.push(g);
                rgba.push(b);
                rgba.push(a);
            } else {
                // Transparent
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }
    Icon::from_rgba(rgba, width, height).expect("Failed to create icon")
}

fn check_and_update() {
    log::info!("Checking for updates...");
    
    let status = self_update::backends::github::Update::configure()
        .repo_owner("keval8solanki")
        .repo_name("macro")
        .bin_name("macro")
        .target("macos") 
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build();

    let updater = match status {
        Ok(s) => s,
        Err(e) => {
            log::error!("Failed to configure update: {}", e);
             rfd::MessageDialog::new()
                .set_title("Update Error")
                .set_description(&format!("Failed to configure update: {}", e))
                .show();
            return;
        }
    };
    
    match updater.get_latest_release() {
        Ok(release) => {
             let latest_version = release.version;
             let current_version = cargo_crate_version!();
             
             if self_update::version::bump_is_greater(current_version, &latest_version).unwrap_or(false) {
                 let confirm = rfd::MessageDialog::new()
                    .set_title("Update Available")
                    .set_description(&format!("New version {} is available (current: {}).\nUpdate now?", latest_version, current_version))
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show();
                    
                 if confirm == rfd::MessageDialogResult::Yes {
                     // Perform update
                     match updater.update() {
                         Ok(_) => {
                             rfd::MessageDialog::new()
                                .set_title("Update Successful")
                                .set_description("Application updated successfully. Please restart the application.")
                                .show();
                         }
                         Err(e) => {
                             rfd::MessageDialog::new()
                                .set_title("Update Failed")
                                .set_description(&format!("Failed to update: {}", e))
                                .show();
                         }
                     }
                 }
             } else {
                 rfd::MessageDialog::new()
                    .set_title("No Update")
                    .set_description("You are on the latest version.")
                    .show();
             }
        }
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            rfd::MessageDialog::new()
                .set_title("Update Check Failed")
                .set_description(&format!("Failed to check for updates: {}", e))
                .show();
        }
    }
}
