//! Configuration handling for hypr-notch
//!
//! This file defines the configuration structure and provides
//! functionality to load and save configuration from/to files.
//! The NotchConfig struct contains all configurable parameters.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// Configuration for the notch appearance and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotchConfig {
    pub collapsed_width: u32,
    pub collapsed_height: u32,
    pub expanded_width: u32,
    pub expanded_height: u32,
    pub corner_radius: u32,
    pub background_color: [u8; 4], // BGRA format

    // New modules field with default
    #[serde(default)]
    pub modules: ModulesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModulesConfig {
    #[serde(default)]
    pub enabled: Vec<String>,

    #[serde(default)]
    pub module_configs: std::collections::HashMap<String, toml::Table>,
}

impl Default for NotchConfig {
    fn default() -> Self {
        Self {
            collapsed_width: 300,
            collapsed_height: 40,
            expanded_width: 800,
            expanded_height: 400,
            corner_radius: 20,
            background_color: [0, 0, 0, 255], // Black, fully opaque
            modules: ModulesConfig::default(),
        }
    }
}

impl NotchConfig {
    /// Get the path to the configuration file
    fn get_config_path() -> PathBuf {
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
}
