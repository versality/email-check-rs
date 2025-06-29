use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub maildir: PathBuf,
    pub state_file: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Could not determine user data directory")?
            .join("email-check");

        std::fs::create_dir_all(&data_dir)
            .context("Failed to create data directory")?;

        let config = Config {
            maildir: dirs::home_dir()
                .context("Could not determine home directory")?
                .join("Maildir"),
            state_file: data_dir.join("seen_ids"),
        };

        Ok(config)
    }
}

