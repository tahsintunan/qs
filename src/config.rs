use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub default: Option<String>,
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    pub fn load() -> Result<Self, String> {
        Self::load_from(Self::path())
    }

    pub fn load_from(path: PathBuf) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content =
            fs::read_to_string(&path).map_err(|e| format!("Failed to read config file: {e}"))?;

        toml::from_str(&content).map_err(|e| format!("Invalid config file format: {e}"))
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(Self::path())
    }

    pub fn save_to(&self, path: PathBuf) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {e}"))?;

        Ok(())
    }

    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("qs")
            .join("config.toml")
    }

    pub fn get_profile(&self, alias: &str) -> Result<&Profile, String> {
        if alias == "default" {
            match &self.default {
                Some(default_alias) => self
                    .profiles
                    .get(default_alias)
                    .ok_or_else(|| format!("Default alias '{default_alias}' not found")),
                None => {
                    if self.profiles.is_empty() {
                        Err("No host found. Add a host: 'qs add <alias> --host <host> --user <user>'".to_string())
                    } else {
                        Err("No default set. Use 'qs set-default <alias>'".to_string())
                    }
                }
            }
        } else {
            self.profiles
                .get(alias)
                .ok_or_else(|| format!("Alias '{alias}' doesn't exist"))
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Profile {
    pub host: String,
    pub user: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    22
}
