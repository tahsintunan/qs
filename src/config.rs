use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub default: Option<String>,
    pub hosts: HashMap<String, Host>,
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

    pub fn get_host(&self, name: &str) -> Option<&Host> {
        self.hosts.get(name).or_else(|| {
            if name == "default" {
                self.default.as_ref().and_then(|n| self.hosts.get(n))
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
            hosts: HashMap::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Host {
    pub host: String,
    pub user: Option<String>,
}
