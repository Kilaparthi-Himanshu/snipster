use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
};

use crate::shortcut::Shortcut;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub shortcut: Shortcut,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: Shortcut::default(),
        }
    }
}

impl Settings {
    pub fn load() -> Result<(Self, PathBuf)> {
        let path = settings_path()?;

        if !path.exists() {
            return Ok((Self::default(), path));
        }

        let contents = fs::read_to_string(&path)
            .context("Could not read Snipster settings")?;

        let settings: Self = toml::from_str(&contents)
            .context("Could not parse Snipster settings.toml")?;

        Ok((settings, path))
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let parent = path
            .parent()
            .context("Settings directory is unavailable")?;

        fs::create_dir_all(parent)
            .context("Could not create settings directory")?;

        let contents = toml::to_string_pretty(self)
            .context("Could not serialize Snipster settings")?;

        fs::write(path, contents)
            .context("Could not save Snipster settings")?;

        Ok(())
    }
}

fn settings_path() -> Result<PathBuf> {
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .context("Windows LOCALAPPDATA directory was not found")?;

    Ok(PathBuf::from(local_app_data)
        .join("Snipster")
        .join("settings.toml"))
}
