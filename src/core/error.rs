use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum Error {
    #[error("Failed to open file: {0}")]
    OpenFile(String),

    #[error("Invalid config: {0}")]
    InvalidConfiguration(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("Hook error: {0}")]
    HookError(String),

    #[error("Preset error: {0}")]
    PresetError(String),

    #[error("Package error: {0}")]
    PackageError(String),

    #[error("Backup error: {0}")]
    BackupError(String),

    #[error("An unknown error occurred!")]
    UnknownError,
}
