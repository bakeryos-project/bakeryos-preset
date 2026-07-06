use serde::{Deserialize, Serialize};
use std::{
    process::{Command, Stdio},
    time::Instant,
};

use crate::core::{backup::BackupManager, config::Config, error::Error};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Package {
    name: String,
    version: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GSettingsConfigItem {
    pub id: String,
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct GSettingsConfig {
    pub gsettings: Vec<GSettingsConfigItem>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DConfConfigItem {
    pub path: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct DConfConfig {
    pub dconf: Vec<DConfConfigItem>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EnvConfigItem {
    pub name: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct EnvConfig {
    pub env: Vec<EnvConfigItem>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum SetConfig {
    GSettingsConfig(GSettingsConfig),
    DConfConfig(DConfConfig),
    EnvConfig(EnvConfig),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Hooks {
    pub pre_install_pkg: Option<Vec<String>>,
    pub after_install_pkg: Option<Vec<String>>,
    pub pre_uninstall_pkg: Option<Vec<String>>,
    pub after_uninstall_pkg: Option<Vec<String>>,
    pub pre_backup: Option<Vec<String>>,
    pub after_backup: Option<Vec<String>>,
}

impl Hooks {
    pub fn run_hook(&self, hook_name: &str) -> Result<(), Error> {
        let scripts = match hook_name {
            "pre_install_pkg" => self
                .pre_install_pkg
                .as_ref()
                .map(|v| v.as_slice())
                .unwrap_or(&[]),

            _ => &[],
        };

        println!("[\x1b[32mINFO\x1b[0m] Runnninng hook: {}", hook_name);

        for s in scripts {
            exec_command(s, &[]).map_err(|e| Error::HookError(e))?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Stage {
    pub name: String,
    pub install_packages: Option<Vec<Package>>,
    pub uninstall_packages: Option<Vec<Package>>,
    pub continue_if_err: Option<bool>,
    pub triggers: Option<Vec<String>>,
    pub backups: Option<Vec<String>>,
    pub restores: Option<Vec<String>>,
    pub configs: Option<Vec<SetConfig>>,
    pub hooks: Option<Hooks>,
}

impl Stage {
    pub fn has_trigger(&self, event_name: &str) -> bool {
        self.triggers
            .as_ref()
            .map(|vec| vec.iter().any(|keyword| keyword.as_str() == event_name))
            .unwrap_or(false)
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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
    pub fn apply_gsettings(&self, cfg: &GSettingsConfigItem) -> Result<(), Error> {
        println!(
            "[\x1b[32mINFO\x1b[0m] Setting GSettings: [{}] {} -> {}",
            cfg.id, cfg.key, cfg.value
        );
        exec_command("gsettings", &["set", &cfg.id, &cfg.key, &cfg.value])
            .map_err(|e| Error::ConfigError(e))?;
        Ok(())
    }

    pub fn apply_dconf(&self, cfg: &DConfConfigItem) -> Result<(), Error> {
        println!(
            "[\x1b[32mINFO\x1b[0m] Setting DConf: {} -> {}",
            cfg.path, cfg.value
        );
        exec_command("dconf", &["write", &cfg.path, &cfg.value])
            .map_err(|e| Error::ConfigError(e))?;
        Ok(())
    }

    pub fn apply_env(&self, cfg: &EnvConfigItem) -> Result<(), Error> {
        println!(
            "[\x1b[32mINFO\x1b[0m] Permanently writing Environment Variable: {}={}",
            cfg.name, cfg.value
        );

        let home = std::env::var("HOME").map_err(|e| Error::ConfigError(e.to_string()))?;
        let bashrc_path = format!("{}/.bashrc", home);

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(bashrc_path)
            .map_err(|e| Error::ConfigError(e.to_string()))?;

        writeln!(file, "export {}={}", cfg.name, cfg.value)
            .map_err(|e| Error::ConfigError(e.to_string()))?;

        Ok(())
    }

    pub fn install_pkg(&self, packages: &[Package]) -> Result<(), Error> {
        let mut args = vec!["-S".to_string(), "--noconfirm".to_string()];
        let mut pkg_names: Vec<String> = packages.iter().map(|pkg| pkg.name.to_string()).collect();
        args.append(&mut pkg_names);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        exec_command("pacman", &args_ref).map_err(|e| Error::PackageError(e))?;

        Ok(())
    }

    pub fn uninstall_pkg(&self, packages: &[Package]) -> Result<(), Error> {
        // -R: Remove packages
        // -n: Remove backup configuration files (nosave)
        // -s: Remove unneeded dependencies (recursive)
        let mut args = vec!["-Rns".to_string(), "--noconfirm".to_string()];
        let mut pkg_names: Vec<String> = packages.iter().map(|pkg| pkg.name.to_string()).collect();
        args.append(&mut pkg_names);
        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        println!(
            "[\x1b[32mINFO\x1b[0m] Uninstalling packages via pacman: {:?}",
            pkg_names
        );

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
        let binding = Hooks {
            pre_install_pkg: vec![].into(),
            after_install_pkg: vec![].into(),
            pre_uninstall_pkg: vec![].into(),
            after_uninstall_pkg: vec![].into(),
            pre_backup: vec![].into(),
            after_backup: vec![].into(),
        };
        let hooks = stage.hooks.as_ref().unwrap_or(&binding);

        let install_packages = stage
            .install_packages
            .as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        if install_packages.len() > 0 {
            hooks.run_hook("pre_install_pkg")?;
            self.install_pkg(install_packages)?;
            hooks.run_hook("after_install_pkg")?;
        }

        let backups = stage.backups.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if backups.len() > 0 {
            hooks.run_hook("pre_backup")?;
            self.backup(backups, conf)?;
            hooks.run_hook("after_backup")?;
        }

        let configs = stage.configs.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        for config_enum in configs {
            match config_enum {
                SetConfig::GSettingsConfig(cfg) => {
                    let gsettings = &cfg.gsettings;
                    for f in gsettings {
                        self.apply_gsettings(f)?;
                    }
                }
                SetConfig::DConfConfig(cfg) => {
                    let dconf = &cfg.dconf;
                    for f in dconf {
                        self.apply_dconf(f)?;
                    }
                }
                SetConfig::EnvConfig(cfg) => {
                    let env = &cfg.env;
                    for f in env {
                        self.apply_env(f)?;
                    }
                }
            }
        }

        let restores = stage.restores.as_ref().map(|v| v.as_slice()).unwrap_or(&[]);
        if restores.len() > 0 {
            self.restore(restores, conf)?;
        }

        let uninstall_packages = stage
            .uninstall_packages
            .as_ref()
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        if uninstall_packages.len() > 0 {
            hooks.run_hook("pre_uninstall_pkg")?;
            self.uninstall_pkg(uninstall_packages)?;
            hooks.run_hook("after_uninstall_pkg")?;
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
