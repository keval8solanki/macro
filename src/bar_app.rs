
use chrono::Local;
use dirs::document_dir;
use eframe::egui;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use global_hotkey::hotkey::{HotKey, Modifiers, Code};
use macro_lib::config;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, TrayIconBuilder, TrayIcon};
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

#[derive(PartialEq)]
enum SettingsTab {
    Control,
    Hotkeys,
}

pub struct BarApp {
    pub state: Arc<Mutex<AppState>>,
    pub tray_icon: Option<TrayIcon>,
    pub recording_menu_item: MenuItem,
    pub playback_menu_item: MenuItem,
    pub load_menu_item: MenuItem,
    pub settings_menu_item: MenuItem,
    pub quit_i: MenuItem,
    pub icon_idle: Icon,
    pub icon_recording: Icon,
    pub icon_playing: Icon,
    pub icon_armed: Icon,
    pub record_hotkey: HotKey,
    pub playback_hotkey: HotKey,
    // UI State
    pub show_settings: bool,
    current_tab: SettingsTab,
    // Hotkey Manager
    _hotkey_manager: GlobalHotKeyManager,
    // System State
    is_quitting: bool,
    is_initialized: bool,
}

impl BarApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize the look and feel
        let mut style = egui::Style::default();
        style.visuals = egui::Visuals::dark();
        style.visuals.window_rounding = egui::Rounding::same(10.0);
        style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.inactive.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.hovered.rounding = egui::Rounding::same(6.0);
        style.visuals.widgets.active.rounding = egui::Rounding::same(6.0);
        style.visuals.selection.bg_fill = egui::Color32::from_rgb(0, 122, 255); // Mac Blue
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        
        cc.egui_ctx.set_style(style);

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
        let settings_menu_item = MenuItem::new("Settings...", true, None); // Disabled by default

        let quit_i = MenuItem::new("Quit", true, None);

        tray_menu.append(&app_title_item).unwrap();
        tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
        tray_menu.append(&recording_menu_item).unwrap();
        tray_menu.append(&playback_menu_item).unwrap();
        tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
        tray_menu.append(&load_menu_item).unwrap();
        tray_menu.append(&settings_menu_item).unwrap();
        tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
        tray_menu.append(&quit_i).unwrap();

        let tray_icon = Some(
            TrayIconBuilder::new()
                .with_menu(Box::new(tray_menu.clone()))
                .with_tooltip("Macro")
                .with_icon(icon_idle.clone())
                .build()
                .unwrap(),
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

        let hotkey_manager = GlobalHotKeyManager::new().unwrap();
        let (record_hotkey, playback_hotkey) = create_hotkeys();
        hotkey_manager.register(record_hotkey).unwrap();
        hotkey_manager.register(playback_hotkey).unwrap();

        Self {
            state,
            tray_icon,
            recording_menu_item,
            playback_menu_item,
            load_menu_item,
            settings_menu_item,
            quit_i,
            icon_idle,
            icon_recording,
            icon_playing,
            icon_armed,
            record_hotkey,
            playback_hotkey,
            show_settings: false,
            current_tab: SettingsTab::Control,
            _hotkey_manager: hotkey_manager,
            is_quitting: false,
            is_initialized: false,
        }
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

    pub fn process_menu_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == self.quit_i.id() {
                // Cleanup
                let mut state = self.state.lock().unwrap();
                if let Some(mut child) = state.recording_process.take() {
                    let _ = child.kill();
                }
                if let Some(mut child) = state.playback_process.take() {
                    let _ = child.kill();
                }
                self.is_quitting = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if event.id == self.recording_menu_item.id() {
                self.handle_toggle_recording();
            } else if event.id == self.playback_menu_item.id() {
                self.handle_toggle_playback();
            } else if event.id == self.load_menu_item.id() {
                // Check if we are loading or unloading
                // Lock state briefly
                let is_loaded = {
                    let state = self.state.lock().unwrap();
                    state.pending_playback.is_some()
                };

                if is_loaded {
                    // Unload Recording
                    let mut state = self.state.lock().unwrap();
                    log::info!("Unloading recording...");
                    state.pending_playback = None;
                    drop(state);
                    self.update_menu_state();
                } else {
                    // Load Recording
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
            } else if event.id == self.settings_menu_item.id() {
                self.show_settings = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
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
            
            // Settings always enabled now, or disabled during record? User said "enabled all the time"
            let _ = self.settings_menu_item.set_enabled(true);
            
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
            
            let _ = self.settings_menu_item.set_enabled(true);
            
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
            
            let _ = self.settings_menu_item.set_enabled(true);
            
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
            
            let _ = self.settings_menu_item.set_enabled(true);
            
            if let Some(tray) = &mut self.tray_icon {
                let _ = tray.set_icon(Some(self.icon_idle.clone()));
            }
        }
    }
}

impl eframe::App for BarApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll events
        self.process_menu_events(ctx);
        
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
             self.handle_hotkey(event);
        }

        self.check_playback_status();
        
        // Handle window close request
        if ctx.input(|i| i.viewport().close_requested()) && !self.is_quitting {
            self.show_settings = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        
        // Force hide on first frame
        if !self.is_initialized {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.is_initialized = true;
        }

        if self.show_settings {
             egui::CentralPanel::default().show(ctx, |ui| {
                 ui.add_space(10.0);
                 ui.heading("Settings");
                 ui.add_space(15.0);

                 // Tabs as buttons with better styling
                 ui.horizontal(|ui| {
                     let control_btn = ui.selectable_label(self.current_tab == SettingsTab::Control, "Control");
                     if control_btn.clicked() { self.current_tab = SettingsTab::Control; }
                     
                     let hotkeys_btn = ui.selectable_label(self.current_tab == SettingsTab::Hotkeys, "Hotkeys");
                     if hotkeys_btn.clicked() { self.current_tab = SettingsTab::Hotkeys; }
                 });
                 ui.separator();
                 ui.add_space(10.0);
                 
                 match self.current_tab {
                     SettingsTab::Control => {
                         let mut state = self.state.lock().unwrap();
                         
                         // Group: Playback Speed
                         egui::Grid::new("control_grid").num_columns(2).spacing([40.0, 20.0]).striped(true).show(ui, |ui| {
                             ui.label(egui::RichText::new("Playback Speed").strong());
                             ui.vertical(|ui| {
                                 ui.add(egui::Slider::new(&mut state.playback_speed, 0.1..=5.0).text("x").logarithmic(true));
                                 ui.horizontal(|ui| {
                                     if ui.small_button("0.5x").clicked() { state.playback_speed = 0.5; }
                                     if ui.small_button("1.0x").clicked() { state.playback_speed = 1.0; }
                                     if ui.small_button("2.0x").clicked() { state.playback_speed = 2.0; }
                                 });
                             });
                             ui.end_row();

                             ui.label(egui::RichText::new("Repeat Count").strong());
                             ui.vertical(|ui| {
                                let mut infinite = state.repeat_count == 0;
                                if ui.checkbox(&mut infinite, "Infinite Loop").changed() {
                                    if infinite { state.repeat_count = 0; } else { state.repeat_count = 1; }
                                }
                                
                                if !infinite {
                                    ui.add(egui::DragValue::new(&mut state.repeat_count).speed(1).range(1..=100));
                                }
                             });
                             ui.end_row();
                         });
                     }
                     SettingsTab::Hotkeys => {
                         ui.label(egui::RichText::new("Actions & Hotkeys").strong().size(16.0));
                         ui.add_space(10.0);
                         
                         egui::Grid::new("hotkey_grid").num_columns(2).spacing([40.0, 10.0]).striped(true).show(ui, |ui| {
                             ui.label("Record / Stop");
                             ui.label(egui::RichText::new("Cmd + Shift + 1").code());
                             ui.end_row();
                             
                             ui.label("Play / Stop");
                             ui.label(egui::RichText::new("Cmd + Shift + 2").code());
                             ui.end_row();
                         });
                         
                         ui.add_space(20.0);
                         ui.label(egui::RichText::new("Note: Hotkeys are currently fixed.").italics().weak());
                     }
                 }
             });
        }
        
        // Repaint periodically to poll events
        ctx.request_repaint_after(Duration::from_millis(100));
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
    if let Ok(Some(global_config)) = config::load_global_config() {
         if let Ok(workspace_config) = config::load_workspace_config(&global_config.workspace_path) {
             workspace_config.path.join("recording")
         } else {
             document_dir().unwrap_or(PathBuf::from(".")).join("Macros")
         }
    } else {
        document_dir().unwrap_or(PathBuf::from(".")).join("Macros")
    }
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
