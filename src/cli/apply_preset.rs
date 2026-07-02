use std::io::{self, Write};

use crate::core::{
    backup::BackupManager, config::Config, error::Error, manager::PresetManager,
    preset::StageResult,
};

pub fn apply_preset(path: &str, conf: &mut Config) -> Result<Vec<StageResult>, Error> {
    let preset = PresetManager::read_preset_file(path)?;

    if conf.has_preset(&preset.id) {
        println!(
            "[\x1b[33mWARN\x1b[0m] Preset '{}' has already been applied to the system.",
            preset.id
        );
        print!("Do you want to re-apply it anyway? (y/N): ");

        io::stdout()
            .flush()
            .map_err(|e| Error::OpenFile(e.to_string()))?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| Error::OpenFile(e.to_string()))?;

        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("[\x1b[34mINFO\x1b[0m] Operation cancelled by the user.");
            return Ok(vec![]);
        }
    } else {
        let preset_backup_path = BackupManager::create_preset_backup_dir(&preset.id)?;
        conf.save_preset(&preset.id, &preset.name, &preset_backup_path)?;
    }

    let result = preset.apply(&conf);
    if result.is_err() {
        return result;
    }

    result
}
