// filepath: src/config.rs
//! Configuration handling for hypr-notch
//!
//! This file defines the configuration structure and provides
//! functionality to load and save configuration from/to files.
//! The NotchConfig struct contains all configurable parameters.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutRow {
    pub alignment: Option<String>, // "left", "center", "right"
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutState {
    pub rows: Vec<LayoutRow>,
    pub row_spacing: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleStateConfig {
    pub visible: Option<bool>,
    pub alignment: Option<String>, // Optional per-module alignment override
}

/// Style properties for the notch (collapsed/expanded/main)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotchStyle {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub corner_radius: Option<u32>,
    pub background_color: Option<[u8; 4]>,
}

/// Configuration for the notch appearance and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotchConfig {
    #[serde(flatten)]
    pub main: NotchStyle,

    #[serde(default)]
    pub collapsed: NotchStyle,

    #[serde(default)]
    pub expanded: NotchStyle,

    #[serde(default)]
    pub modules: ModulesConfig,

    #[serde(default)]
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutConfig {
    #[serde(default)]
    pub expanded: LayoutState,
    #[serde(default)]
    pub collapsed: LayoutState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulesConfig {
    #[serde(default)]
    pub enabled: Vec<String>,

    #[serde(default)]
    pub module_configs: HashMap<String, toml::Table>,

    #[serde(default)]
    pub state: HashMap<String, ModuleStateConfigSet>,

    #[serde(default)]
    pub aliases: HashMap<String, String>, // alias -> path
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModuleStateConfigSet {
    #[serde(default)]
    pub expanded: ModuleStateConfig,
    #[serde(default)]
    pub collapsed: ModuleStateConfig,
}

/// Resolved style with no Option fields
#[derive(Debug, Clone)]
pub struct NotchStyleResolved {
    pub width: u32,
    pub height: u32,
    pub corner_radius: u32,
    pub background_color: [u8; 4],
}

impl Default for NotchConfig {
    fn default() -> Self {
        Self {
            main: NotchStyle {
                width: Some(300),
                height: Some(40),
                corner_radius: Some(10),
                background_color: Some([0, 0, 0, 255]),
            },
            collapsed: NotchStyle::default(),
            expanded: NotchStyle::default(),
            modules: ModulesConfig::default(),
            layout: LayoutConfig::default(),
        }
    }
}

impl NotchConfig {
    /// Get the path to the configuration file
    pub fn get_config_path() -> PathBuf {
        let config_dir = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("hypr-notch")
        } else {
            PathBuf::from(".config/hypr-notch")
        };

        config_dir.join("config.toml")
    }

    /// Load configuration from file, returning default if not found
    pub fn load_from_file() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        // Try to read the configuration file
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                // Parse the TOML content
                let config: Self = toml::from_str(&content)?;
                Ok(config)
            }
            Err(e) if e.kind() == ErrorKind::NotFound => {
                // If the file doesn't exist, create it with default values
                let default_config = Self::default();
                default_config.save_to_file()?;
                Ok(default_config)
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Save configuration to file
    pub fn save_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();

        // Create the directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            if !Path::exists(parent) {
                fs::create_dir_all(parent)?;
            }
        }

        // Serialize and write the configuration
        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;

        Ok(())
    }

    /// Get the effective style for the current state (expanded/collapsed)
    pub fn style_for(&self, expanded: bool) -> NotchStyleResolved {
        let (section, fallback) = if expanded {
            (&self.expanded, &self.main)
        } else {
            (&self.collapsed, &self.main)
        };
        NotchStyleResolved {
            width: section.width.or(fallback.width).unwrap_or(300),
            height: section.height.or(fallback.height).unwrap_or(40),
            corner_radius: section
                .corner_radius
                .or(fallback.corner_radius)
                .unwrap_or(10),
            background_color: section
                .background_color
                .or(fallback.background_color)
                .unwrap_or([0, 0, 0, 255]),
        }
    }
}
