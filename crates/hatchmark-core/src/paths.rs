use crate::Result;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub db_path: PathBuf,
    pub log_path: PathBuf,
    pub state_path: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Result<Self> {
        let dirs = ProjectDirs::from("", "", "Hatchmark").ok_or_else(|| {
            crate::CoreError::Invalid("could not resolve %APPDATA% directory".into())
        })?;
        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        Ok(Self {
            db_path: data_dir.join("hatchmark.db"),
            log_path: data_dir.join("log.txt"),
            state_path: data_dir.join("daemon-state.json"),
            data_dir,
        })
    }
}
