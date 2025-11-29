use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use rdev::Key;
use anyhow::{Result, Context};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalConfig {
    pub workspace_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorkspaceConfig {
    pub path: PathBuf,
    pub keymaps: KeyMaps,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyMaps {
    pub start_recording: KeyCombo,
    pub stop_recording: KeyCombo,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyCombo {
    pub modifiers: Vec<Modifier>,
    pub trigger: Key,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Modifier {
    Cmd,
    Alt,
    Ctrl,
    Shift,
}

impl Default for KeyMaps {
    fn default() -> Self {
        Self {
            start_recording: KeyCombo {
                modifiers: vec![Modifier::Cmd, Modifier::Alt],
                trigger: Key::KeyR,
            },
            stop_recording: KeyCombo {
                modifiers: vec![Modifier::Cmd, Modifier::Alt],
                trigger: Key::Escape,
            },
        }
    }
}

pub fn get_global_config_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("com", "event-replay", "cli")
        .context("Could not determine config directory")?;
    Ok(dirs.config_dir().join("settings.json"))
}

pub fn load_global_config() -> Result<Option<GlobalConfig>> {
    let path = get_global_config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let config: GlobalConfig = serde_json::from_str(&content)?;
    Ok(Some(config))
}

pub fn save_global_config(config: &GlobalConfig) -> Result<()> {
    let path = get_global_config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

pub fn load_workspace_config(path: &PathBuf) -> Result<WorkspaceConfig> {
    let config_path = path.join("config.json");
    let content = fs::read_to_string(config_path)?;
    let config: WorkspaceConfig = serde_json::from_str(&content)?;
    Ok(config)
}

pub fn create_workspace(path: PathBuf) -> Result<WorkspaceConfig> {
    let config_path = path.join("config.json");
    let recording_path = path.join("recording");
    
    fs::create_dir_all(&recording_path)?;
    
    let config = WorkspaceConfig {
        path: path.clone(),
        keymaps: KeyMaps::default(),
    };
    
    let content = serde_json::to_string_pretty(&config)?;
    fs::write(config_path, content)?;
    
    Ok(config)
}
