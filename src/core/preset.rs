use serde::{Deserialize, Serialize};
use std::{
    process::{Command, Stdio},
    time::Instant,
};

use crate::core::{backup::BackupManager, config::Config, error::Error};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    name: String,
    version: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Stage {
    pub name: String,
    pub packages: Option<Vec<Package>>,
    pub continue_if_err: Option<bool>,
    pub triggers: Option<Vec<String>>,
    pub backups: Option<Vec<String>>,
    pub restores: Option<Vec<String>>,
}

impl Stage {
    pub fn has_trigger(&self, event_name: &str) -> bool {
        self.triggers
            .as_ref()
            .map(|vec| {
                vec.iter()
                    .any(|keyword| event_name.contains(keyword.as_str()))
            })
            .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub name: String,
    pub id: String,
    pub stages: Vec<Stage>,
}

pub struct StageResult {
    pub name: String,
    pub time: u128,
    pub is_success: bool,
    pub err: Option<Error>,
}

fn exec_command(cmd: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(cmd)
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| format!("Error: '{}': {}", cmd, e))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "Command '{}'failed with code: {:?}",
            cmd,
            status.code()
        ))
    }
}

impl Preset {
    pub fn install_pkg(&self, packages: &[Package]) -> Result<(), Error> {
        let mut args = vec!["-Sy".to_string(), "--noconfirm".to_string()];
        let mut pkg_names: Vec<String> = packages.iter().map(|pkg| pkg.name.to_string()).collect();
        args.append(&mut pkg_names);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        exec_command("pacman", &args_ref).map_err(|e| Error::PackageError(e))?;

        Ok(())
    }

    pub fn backup(&self, backups: &[String], conf: &Config) -> Result<(), Error> {
        let backup_dir_path = conf
            .get_backup_path_by_id(&self.id)
            .unwrap_or("".to_owned());
        let mut backup_info = BackupManager::get_backup_info(&backup_dir_path)
            .map_err(|e| Error::BackupError(e.to_string()))?;
        for file in backups {
            println!(
                "[\x1b[32mINFO\x1b[0m] Backing up file {} -> {}",
                file, backup_dir_path
            );
            backup_info.backup_file(file)?;
        }

        backup_info.update_indexer()?;

        Ok(())
    }

    pub fn restore(&self, paths: &[String], conf: &Config) -> Result<(), Error> {
        let backup_dir_path = conf
            .get_backup_path_by_id(&self.id)
            .unwrap_or("".to_owned());
        let backup_info = BackupManager::get_backup_info(&backup_dir_path)
            .map_err(|e| Error::BackupError(e.to_string()))?;
        for file in paths {
            println!(
                "[\x1b[32mINFO\x1b[0m] Restoring file {} -> {}",
                backup_dir_path, file
            );
            backup_info.restore_file(file)?;
        }

        backup_info.update_indexer()?;

        Ok(())
    }

    pub fn apply_stage(&self, stage: &Stage, conf: &Config) -> Result<(), Error> {
        let pkgs = stage.packages.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if pkgs.len() > 0 {
            self.install_pkg(pkgs)?;
        }

        let backups = stage.backups.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if backups.len() > 0 {
            self.backup(backups, conf)?;
        }

        let restores = stage.restores.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if restores.len() > 0 {
            self.restore(restores, conf)?;
        }
        Ok(())
    }

    pub fn run(&self, conf: &Config, with_trigger: Vec<&str>) -> Result<Vec<StageResult>, Error> {
        let mut stage_results: Vec<StageResult> = vec![];

        for stage in &self.stages {
            let allow_trigger = with_trigger.iter().any(|f| stage.has_trigger(f));
            if !allow_trigger {
                continue;
            }
            println!("[\x1b[32mINFO\x1b[0m] Applying stage #{}", stage.name);
            let start = Instant::now();
            let result = self.apply_stage(stage, conf);
            let duration = start.elapsed();
            let continue_if_err = stage.continue_if_err.unwrap_or(false);

            let is_success = result.is_ok();
            let err = result.err();

            stage_results.push(StageResult {
                name: stage.name.to_owned(),
                time: duration.as_millis(),
                is_success,
                err,
            });

            if !is_success && !continue_if_err {
                break;
            }
        }

        Ok(stage_results)
    }

    pub fn apply(&self, conf: &Config) -> Result<Vec<StageResult>, Error> {
        self.run(conf, vec!["apply"])
    }

    pub fn unapply(&self, conf: &Config) -> Result<Vec<StageResult>, Error> {
        self.run(conf, vec!["unapply"])
    }
}
