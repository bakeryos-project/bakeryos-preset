use std::{
    fs::File,
    io::Read,
    path::Path,
};


use crate::core::{error::Error, preset::Preset};

pub struct PresetManager {}

impl PresetManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn read_preset_file<P: AsRef<Path>>(path: P) -> Result<Preset, Error> {
        let path_ref = path.as_ref();

        let mut file = File::open(path_ref)
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, path_ref.display())))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| Error::OpenFile(format!("{}, path: {}", e, path_ref.display())))?;

        let config: Preset = serde_yaml::from_str(&contents)
            .map_err(|e| Error::InvalidConfiguration(e.to_string()))?;

        Ok(config)
    }
}
