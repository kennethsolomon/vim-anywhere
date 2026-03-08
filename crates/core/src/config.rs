use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub mode_entry: ModeEntryConfigJson,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_overlay_size")]
    pub overlay_size: String,
    #[serde(default = "default_overlay_position")]
    pub overlay_position: String,
    #[serde(default = "default_true")]
    pub focus_highlight: bool,
    #[serde(default = "default_true")]
    pub show_overlay: bool,
    #[serde(default = "default_true")]
    pub menu_bar_icon: bool,
    #[serde(default)]
    pub launch_at_login: bool,
    #[serde(default)]
    pub custom_mappings: Vec<CustomMapping>,
    #[serde(default)]
    pub disabled_motions: Vec<String>,
    #[serde(default)]
    pub per_app: HashMap<String, AppConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeEntryConfigJson {
    #[serde(default = "default_mode_entry")]
    pub method: String,
    #[serde(default)]
    pub custom_sequence: Option<String>,
    #[serde(default = "default_true")]
    pub double_escape_sends_real: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomMapping {
    pub mode: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_strategy")]
    pub strategy: String,
    #[serde(default)]
    pub custom_mappings: Vec<CustomMapping>,
}

fn default_theme() -> String { "dark".to_string() }
fn default_overlay_size() -> String { "medium".to_string() }
fn default_overlay_position() -> String { "bottom-right".to_string() }
fn default_true() -> bool { true }
fn default_mode_entry() -> String { "escape".to_string() }
fn default_strategy() -> String { "accessibility".to_string() }

impl Default for ModeEntryConfigJson {
    fn default() -> Self {
        Self {
            method: default_mode_entry(),
            custom_sequence: None,
            double_escape_sends_real: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode_entry: ModeEntryConfigJson::default(),
            theme: default_theme(),
            overlay_size: default_overlay_size(),
            overlay_position: default_overlay_position(),
            focus_highlight: true,
            show_overlay: true,
            menu_bar_icon: true,
            launch_at_login: false,
            custom_mappings: vec![],
            disabled_motions: vec![],
            per_app: HashMap::new(),
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        #[allow(deprecated)]
        let home = std::env::home_dir()
            .or_else(|| std::env::var("HOME").ok().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        home.join(".config")
            .join("vim-anywhere")
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("[vim-anywhere] warning: config parse error at {}: {}", path.display(), e);
                    Self::default()
                }
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => {
                eprintln!("[vim-anywhere] warning: failed to read config at {}: {}", path.display(), e);
                Self::default()
            }
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.theme, "dark");
        assert_eq!(config.overlay_size, "medium");
        assert!(config.focus_highlight);
        assert!(config.menu_bar_icon);
        assert!(!config.launch_at_login);
        assert!(config.custom_mappings.is_empty());
        assert!(config.per_app.is_empty());
    }

    #[test]
    fn serialize_deserialize() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.theme, "dark");
        assert_eq!(parsed.mode_entry.method, "escape");
    }

    #[test]
    fn deserialize_partial() {
        let json = r#"{"theme": "light"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.theme, "light");
        assert_eq!(config.overlay_size, "medium"); // default
    }
}
