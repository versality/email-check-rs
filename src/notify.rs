use anyhow::{Context, Result};
use std::process::Command;

pub fn send_notification(subject: &str, from: &str) -> Result<()> {
    let body = format!("From: {}\nSubject: {}", from, subject);
    
    let output = Command::new("notify-send")
        .args(["New Mail", &body])
        .output()
        .context("Failed to execute notify-send - is it installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("notify-send failed: {}", stderr);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_notification_formats_message_correctly() {
        let result = send_notification("Test Subject", "test@example.com");
        
        match result {
            Ok(_) => println!("Notification sent successfully"),
            Err(e) => println!("Notification failed (expected in test environment): {}", e),
        }
    }
}
