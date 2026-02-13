use egui::Color32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtensionSetting {
    pub enabled: bool,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub extensions: HashMap<String, ExtensionSetting>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(rename = "themes_path")]
    pub themes_path: String,
    #[serde(rename = "save_prefix", default)]
    pub save_prefix: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            themes_path: dirs::home_dir()
                .map(|p| {
                    p.join(".config/omarchy/themes")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_else(|| "/home/your/.config/omarchy/themes".to_string()),
            save_prefix: "new-".to_string(),
        }
    }
}

pub fn get_default_extensions() -> HashMap<String, ExtensionSetting> {
    let mut extensions = HashMap::new();

    extensions.insert(
        ".css".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#2646dc".to_string(),
        },
    );
    extensions.insert(
        ".toml".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#ff9f43".to_string(),
        },
    );
    extensions.insert(
        ".theme".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#5f27cd".to_string(),
        },
    );
    extensions.insert(
        ".conf".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#1dd1a1".to_string(),
        },
    );
    extensions.insert(
        ".lua".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#22a6b3".to_string(),
        },
    );
    extensions.insert(
        ".json".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#f4b426".to_string(),
        },
    );
    extensions.insert(
        ".yaml".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#4ecdcd".to_string(),
        },
    );
    extensions.insert(
        ".ini".to_string(),
        ExtensionSetting {
            enabled: true,
            color: "#ff9ff3".to_string(),
        },
    );

    extensions
}

fn get_project_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

fn get_config_path() -> Option<PathBuf> {
    if let Some(config_dir) = dirs::config_dir() {
        let config_path = config_dir.join("omarchy-theme-maker").join("config.toml");
        if config_path.exists() {
            return Some(config_path);
        }
    }

    if let Some(project_dir) = get_project_dir() {
        let project_config = project_dir.join("config.toml");
        if project_config.exists() {
            return Some(project_config);
        }
    }

    None
}

fn get_config_dir() -> PathBuf {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("omarchy-theme-maker")
    } else if let Some(project_dir) = get_project_dir() {
        project_dir.join("config")
    } else {
        PathBuf::from(".")
    }
}

pub fn load_config() -> (AppConfig, Option<String>) {
    if let Some(config_path) = get_config_path() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str(&content) {
                return (config, Some(config_path.to_string_lossy().to_string()));
            }
        }
    }

    (
        AppConfig {
            general: GeneralConfig::default(),
            extensions: get_default_extensions(),
        },
        None,
    )
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_dir = get_config_dir();
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;

    let config_path = config_dir.join("config.toml");
    let content = toml::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, content).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn color_from_hex(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            Color32::from_rgb(r, g, b)
        }
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
            Color32::from_rgb(r, g, b)
        }
        _ => Color32::from_gray(128),
    }
}

pub fn color_to_hex(color: Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}
