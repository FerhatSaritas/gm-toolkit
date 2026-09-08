use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::Manager;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub campaign_path: Option<PathBuf>,
}

fn config_path(app: &tauri::AppHandle) -> Result<PathBuf, std::io::Error> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(std::io::Error::other)?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join("config.json"))
}

pub fn load_config(app: &tauri::AppHandle) -> Result<Config, std::io::Error> {
    let path = config_path(app)?;
    match fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e),
    }
}

pub fn save_config(app: &tauri::AppHandle, config: &Config) -> Result<(), std::io::Error> {
    let path = config_path(app)?;
    let tmp = path.with_extension("json.tmp");
    let raw = serde_json::to_string_pretty(config)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&tmp, raw)?;
    fs::rename(&tmp, &path)
}
