use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    path::Path,
};

use serde::{Deserialize, Serialize};

use crate::core::error::Error;

const CONFIG_PATH: &str = "/usr/share/bakeryos/preset/config.json";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PresetConfig {
    pub name: String,
    pub backup: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub presets: HashMap<String, PresetConfig>,
}

impl Config {
    pub fn get_backup_path_by_id(&self, id: &str) -> Option<String> {
        self.presets
            .get(id)
            .and_then(|preset| preset.backup.clone())
    }

    pub fn has_preset(&self, id: &str) -> bool {
        self.presets.contains_key(id)
    }

    pub fn save_preset(&mut self, id: &str, name: &str, backup_path: &str) -> Result<(), Error> {
        if self.presets.contains_key(id) {
            println!(
                "[\x1b[34mINFO\x1b[0m] Preset ID '{}' already exists. Updating configuration...",
                id
            );
        }

        self.presets.insert(
            id.to_owned(),
            PresetConfig {
                name: name.to_owned(),
                backup: Some(backup_path.to_owned()),
            },
        );

        Ok(())
    }
}
pub struct ConfigManager {}

impl ConfigManager {
    pub fn read_config() -> Result<Config, Error> {
        let path = Path::new(CONFIG_PATH);

        let mut file = File::open(path)
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, CONFIG_PATH)))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, CONFIG_PATH)))?;

        let config: Config = serde_json::from_str(&contents)
            .map_err(|e| Error::InvalidConfiguration(e.to_string()))?;

        Ok(config)
    }

    pub fn write_config(config: &Config) -> Result<(), Error> {
        let path = Path::new(CONFIG_PATH);

        let json_string = serde_json::to_string_pretty(config)
            .map_err(|e| Error::InvalidConfiguration(e.to_string()))?;

        let mut file = File::create(path)
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, CONFIG_PATH)))?;

        file.write_all(json_string.as_bytes())
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, CONFIG_PATH)))?;

        file.flush()
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, CONFIG_PATH)))?;

        Ok(())
    }
}
