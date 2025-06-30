use anyhow::{Context, Result};
use std::process::Command;

pub fn sync_mail() -> Result<()> {
    let output = Command::new("mbsync")
        .args(["-Va"])
        .output()
        .context("Failed to execute mbsync - is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("mbsync failed: {}", stderr);
    }

    Ok(())
}
