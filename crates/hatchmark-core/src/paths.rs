use crate::{CoreError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_PROFILE: &str = "default";

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub profiles_dir: PathBuf,
    pub active_profile_path: PathBuf,
    pub log_path: PathBuf,
    pub state_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct ActiveProfile {
    name: String,
}

impl AppPaths {
    pub fn resolve() -> Result<Self> {
        let dirs = ProjectDirs::from("", "", "Hatchmark").ok_or_else(|| {
            CoreError::Invalid("could not resolve %APPDATA% directory".into())
        })?;
        let data_dir = dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;
        let profiles_dir = data_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir)?;

        let paths = Self {
            log_path: data_dir.join("log.txt"),
            state_path: data_dir.join("daemon-state.json"),
            active_profile_path: data_dir.join("active-profile.json"),
            profiles_dir,
            data_dir,
        };
        paths.migrate_legacy_db()?;
        Ok(paths)
    }

    /// One-time migration: if the pre-profiles hatchmark.db exists and
    /// profiles/default.db does not, move the legacy DB into the profiles
    /// directory so existing users keep their data.
    fn migrate_legacy_db(&self) -> Result<()> {
        let legacy = self.data_dir.join("hatchmark.db");
        let default_db = self.profile_db(DEFAULT_PROFILE);
        if legacy.exists() && !default_db.exists() {
            std::fs::rename(&legacy, &default_db)?;
            for (src_name, dst_name) in [
                ("hatchmark.db-wal", format!("{DEFAULT_PROFILE}.db-wal")),
                ("hatchmark.db-shm", format!("{DEFAULT_PROFILE}.db-shm")),
            ] {
                let src = self.data_dir.join(src_name);
                if src.exists() {
                    let _ = std::fs::rename(src, self.profiles_dir.join(dst_name));
                }
            }
        }
        Ok(())
    }

    pub fn profile_db(&self, name: &str) -> PathBuf {
        self.profiles_dir.join(format!("{name}.db"))
    }

    /// Read the active profile name from disk. Defaults to "default" if
    /// no config file exists yet.
    pub fn active_profile(&self) -> Result<String> {
        if !self.active_profile_path.exists() {
            return Ok(DEFAULT_PROFILE.to_string());
        }
        let raw = std::fs::read(&self.active_profile_path)?;
        let cfg: ActiveProfile = serde_json::from_slice(&raw)
            .map_err(|e| CoreError::Invalid(format!("active-profile.json: {e}")))?;
        Ok(cfg.name)
    }

    pub fn set_active_profile(&self, name: &str) -> Result<()> {
        validate_profile_name(name)?;
        let cfg = ActiveProfile {
            name: name.to_string(),
        };
        std::fs::write(
            &self.active_profile_path,
            serde_json::to_vec_pretty(&cfg)?,
        )?;
        Ok(())
    }

    /// Convenience: path of the DB for the currently active profile.
    pub fn active_profile_db(&self) -> Result<PathBuf> {
        Ok(self.profile_db(&self.active_profile()?))
    }

    pub fn list_profiles(&self) -> Result<Vec<String>> {
        let mut names: Vec<String> = std::fs::read_dir(&self.profiles_dir)?
            .filter_map(|e| e.ok())
            .filter_map(|entry| {
                let p = entry.path();
                if p.extension().and_then(|e| e.to_str()) == Some("db") {
                    p.file_stem()
                        .and_then(|s| s.to_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();
        if !names.iter().any(|n| n == DEFAULT_PROFILE) {
            names.push(DEFAULT_PROFILE.to_string());
        }
        names.sort();
        Ok(names)
    }

    pub fn create_profile(&self, name: &str) -> Result<PathBuf> {
        validate_profile_name(name)?;
        let path = self.profile_db(name);
        if path.exists() {
            return Err(CoreError::Invalid(format!(
                "profile '{name}' already exists"
            )));
        }
        // Touch the file so list_profiles sees it. The Db layer runs
        // migrations on first open.
        std::fs::write(&path, [])?;
        Ok(path)
    }

    pub fn delete_profile(&self, name: &str) -> Result<()> {
        validate_profile_name(name)?;
        if name == DEFAULT_PROFILE {
            return Err(CoreError::Invalid(
                "cannot delete the 'default' profile".into(),
            ));
        }
        if self.active_profile()? == name {
            return Err(CoreError::Invalid(
                "cannot delete the active profile; switch first".into(),
            ));
        }
        let path = self.profile_db(name);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        for suffix in ["-wal", "-shm"] {
            let sidecar = self.profiles_dir.join(format!("{name}.db{suffix}"));
            let _ = std::fs::remove_file(sidecar);
        }
        Ok(())
    }
}

fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 64 {
        return Err(CoreError::Invalid(
            "profile name must be 1-64 chars".into(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CoreError::Invalid(
            "profile name: letters, digits, '-', '_' only".into(),
        ));
    }
    Ok(())
}
