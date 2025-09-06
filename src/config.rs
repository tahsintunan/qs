use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default: Option<String>,
    pub profiles: HashMap<String, Profile>,
}

impl Config {
    pub fn load() -> Self {
        let path = Self::path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            toml::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::path();
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(path, toml::to_string_pretty(self).unwrap()).ok();
    }

    pub fn path() -> PathBuf {
        dirs::home_dir()
            .unwrap()
            .join(".config")
            .join("qs")
            .join("config.toml")
    }

    pub fn get_profile(&self, alias: &str) -> Option<&Profile> {
        self.profiles.get(alias).or_else(|| {
            if alias == "default" {
                self.default.as_ref().and_then(|n| self.profiles.get(n))
            } else {
                None
            }
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: None,
            profiles: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub host: String,
    pub user: String,
}
