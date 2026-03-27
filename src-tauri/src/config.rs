use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub autostart: bool,
    pub step: u32,
    pub resync_enabled: bool,
    pub resync_interval_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            autostart: false,
            step: 5,
            resync_enabled: false,
            resync_interval_ms: 2000,
        }
    }
}

pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("sun")
        .join("config.toml")
}

pub fn load() -> Config {
    let path = config_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|content| toml::from_str(&content).ok())
        .unwrap_or_default()
}

pub fn save(config: &Config) -> Result<(), String> {
    let path = config_path();
    fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}