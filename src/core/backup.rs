use std::{collections::HashMap, fs, path::Path};

use uuid::Uuid;

use crate::core::error::Error;

pub type BackupIndex = HashMap<String, String>;

pub struct BackupInfo<'a> {
    pub backup_dir: &'a Path,
    pub indexs: BackupIndex,
}

impl<'a> BackupInfo<'a> {
    pub fn backup_file(&mut self, original_file_path: &str) -> Result<(), Error> {
        let backup_file_name = format!("{}.backup", Uuid::new_v4().hyphenated());
        let target_backup_path = self.backup_dir.join(&backup_file_name);

        fs::copy(original_file_path, &target_backup_path).map_err(|e| {
            Error::OpenFile(format!(
                "Failed to copy file from '{}' to '{}': {}",
                original_file_path,
                target_backup_path.display(),
                e
            ))
        })?;

        self.indexs
            .insert(original_file_path.to_string(), backup_file_name);

        Ok(())
    }

    pub fn update_indexer(&self) -> Result<(), Error> {
        let updated_json = serde_json::to_string_pretty(&self.indexs)
            .map_err(|e| Error::OpenFile(format!("Failed to serialize updated index: {}", e)))?;
        let index_file_path = self.backup_dir.join("index.json");

        fs::write(&index_file_path, updated_json).map_err(|e| {
            Error::OpenFile(format!(
                "Failed to write updated index.json to '{}': {}",
                index_file_path.display(),
                e
            ))
        })?;

        Ok(())
    }

    pub fn restore_file(&self, original_file_path: &str) -> Result<(), Error> {
        let backup_file_name = self.indexs.get(original_file_path).ok_or_else(|| {
            Error::OpenFile(format!(
                "No backup reference found for file '{}' in the index.",
                original_file_path
            ))
        })?;

        let source_backup_path = self.backup_dir.join(backup_file_name);

        if !source_backup_path.exists() {
            return Err(Error::OpenFile(format!(
                "Backup file '{}' is missing from the backup directory.",
                source_backup_path.display()
            )));
        }

        fs::copy(&source_backup_path, original_file_path).map_err(|e| {
            Error::OpenFile(format!(
                "Failed to restore file from '{}' to '{}': {}",
                source_backup_path.display(),
                original_file_path,
                e
            ))
        })?;

        Ok(())
    }
}

pub struct BackupManager {}

impl BackupManager {
    pub fn get_backup_info(backup_dir_path: &str) -> Result<BackupInfo<'_>, Error> {
        let backup_dir = Path::new(backup_dir_path);
        let index_file_path = backup_dir.join("index.json");

        let index_content = fs::read_to_string(&index_file_path)
            .map_err(|e| Error::OpenFile(format!("Failed to read index.json: {}", e)))?;

        let index_map: BackupIndex = serde_json::from_str(&index_content)
            .map_err(|e| Error::OpenFile(format!("Failed to parse index.json: {}", e)))?;

        Ok(BackupInfo {
            indexs: index_map,
            backup_dir: backup_dir,
        })
    }

    pub fn create_preset_backup_dir(preset_id: &str) -> Result<String, Error> {
        let unique_id = Uuid::new_v4().hyphenated().to_string();

        let dir_path_str = format!(
            "/usr/share/bakeryos/preset/backups/{}+{}",
            preset_id, unique_id
        );
        let path = Path::new(&dir_path_str);
        fs::create_dir_all(path).map_err(|e| {
            Error::OpenFile(format!(
                "Failed to create backup directory '{}': {}",
                dir_path_str, e
            ))
        })?;

        let index_map: BackupIndex = HashMap::new();
        let json_string = serde_json::to_string_pretty(&index_map).map_err(|e| {
            Error::OpenFile(format!("Failed to serialize empty backup index: {}", e))
        })?;

        let index_file_path = path.join("index.json");
        fs::write(&index_file_path, json_string).map_err(|e| {
            Error::OpenFile(format!(
                "Failed to create index.json in '{}': {}",
                dir_path_str, e
            ))
        })?;

        Ok(dir_path_str)
    }
}
