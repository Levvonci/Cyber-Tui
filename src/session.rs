use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Session {
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub username: Option<String>,
}

fn config_dir() -> Result<PathBuf> {
    let base = dirs::config_dir().context("Impossibile determinare la config dir")?;
    let dir = base.join("cyber-tui");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn session_path() -> Result<PathBuf> {
    Ok(config_dir()?.join("session.json"))
}

impl Session {
    pub fn load() -> Result<Self> {
        let path = session_path()?;
        if !path.exists() {
            return Ok(Session::default());
        }
        let data = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data).unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = session_path()?;
        fs::write(&path, serde_json::to_string_pretty(self)?)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut p = fs::metadata(&path)?.permissions();
            p.set_mode(0o600);
            fs::set_permissions(&path, p)?;
        }
        Ok(())
    }

    pub fn clear() -> Result<()> {
        let path = session_path()?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
    pub fn is_authenticated(&self) -> bool {
        self.id_token.is_some()
    }
}
