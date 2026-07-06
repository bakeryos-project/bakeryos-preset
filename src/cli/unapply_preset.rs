use crate::core::{config::Config, error::Error, manager::PresetManager, preset::StageResult};

pub fn unapply_preset(path: &str, conf: &mut Config) -> Result<Vec<StageResult>, Error> {
    let preset = PresetManager::read_preset_file(path)?;

    let result = preset.unapply(&conf);
    if result.is_err() {
        return result;
    }

    result
}
