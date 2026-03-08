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
    pub dim_background: bool,
    #[serde(default = "default_dim_intensity")]
    pub dim_intensity: String,
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
    #[serde(default = "default_excluded_apps")]
    pub excluded_apps: Vec<String>,
    #[serde(default)]
    pub per_app: HashMap<String, AppConfig>,
    #[serde(default)]
    pub onboarding_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeEntryConfigJson {
    #[serde(default = "default_mode_entry")]
    pub method: String,
    #[serde(default)]
    pub custom_sequence: Option<String>,
    #[serde(default)]
    pub double_escape_sends_real: bool,
    #[serde(default = "default_true")]
    pub smart_escape: bool,
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
fn default_dim_intensity() -> String { "light".to_string() }
fn default_strategy() -> String { "accessibility".to_string() }
fn default_excluded_apps() -> Vec<String> {
    vec![
        "com.apple.Terminal".to_string(),
        "com.googlecode.iterm2".to_string(),
        "io.alacritty".to_string(),
        "com.mitchellh.ghostty".to_string(),
        "net.kovidgoyal.kitty".to_string(),
        "dev.warp.Warp-Stable".to_string(),
        "com.github.wez.wezterm".to_string(),
        "co.zeit.hyper".to_string(),
    ]
}

impl Default for ModeEntryConfigJson {
    fn default() -> Self {
        Self {
            method: default_mode_entry(),
            custom_sequence: None,
            double_escape_sends_real: false,
            smart_escape: true,
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
            dim_background: true,
            dim_intensity: default_dim_intensity(),
            show_overlay: true,
            menu_bar_icon: true,
            launch_at_login: false,
            custom_mappings: vec![],
            disabled_motions: vec![],
            excluded_apps: default_excluded_apps(),
            per_app: HashMap::new(),
            onboarding_complete: false,
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

    #[test]
    fn default_config_new_fields() {
        let config = Config::default();
        assert!(config.focus_highlight);
        assert!(config.dim_background);
        assert_eq!(config.dim_intensity, "light");
        assert!(!config.onboarding_complete);
        assert!(!config.excluded_apps.is_empty());
        assert!(config.excluded_apps.contains(&"com.apple.Terminal".to_string()));
        assert!(config.excluded_apps.contains(&"net.kovidgoyal.kitty".to_string()));
    }

    #[test]
    fn default_mode_entry_smart_escape() {
        let me = ModeEntryConfigJson::default();
        assert_eq!(me.method, "escape");
        assert!(me.smart_escape);
        assert!(!me.double_escape_sends_real);
        assert!(me.custom_sequence.is_none());
    }

    #[test]
    fn deserialize_with_excluded_apps() {
        let json = r#"{"excluded_apps": ["com.example.app"]}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert_eq!(config.excluded_apps, vec!["com.example.app"]);
    }

    #[test]
    fn deserialize_with_onboarding() {
        let json = r#"{"onboarding_complete": true}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(config.onboarding_complete);
    }

    #[test]
    fn deserialize_with_dim_settings() {
        let json = r#"{"dim_background": false, "dim_intensity": "heavy"}"#;
        let config: Config = serde_json::from_str(json).unwrap();
        assert!(!config.dim_background);
        assert_eq!(config.dim_intensity, "heavy");
    }

    #[test]
    fn roundtrip_new_fields() {
        let mut config = Config::default();
        config.onboarding_complete = true;
        config.dim_background = false;
        config.dim_intensity = "medium".to_string();
        config.excluded_apps = vec!["test.app".to_string()];
        let json = serde_json::to_string(&config).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert!(parsed.onboarding_complete);
        assert!(!parsed.dim_background);
        assert_eq!(parsed.dim_intensity, "medium");
        assert_eq!(parsed.excluded_apps, vec!["test.app"]);
    }

    #[test]
    fn default_excluded_apps_count() {
        let config = Config::default();
        assert_eq!(config.excluded_apps.len(), 8);
    }
}
