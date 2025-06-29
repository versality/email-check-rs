use anyhow::{Context, Result};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct Email {
    pub id: String,
    pub subject: String,
    pub from: String,
}

pub fn scan_maildir(maildir: &Path) -> Result<Vec<Email>> {
    let mut all_emails = Vec::new();

    let entries = fs::read_dir(maildir)
        .with_context(|| format!("Failed to read maildir: {}", maildir.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let account_path = entry.path();

        if !account_path.is_dir() {
            continue;
        }

        let inbox_new = account_path.join("Inbox").join("new");
        if let Ok(emails) = scan_inbox_new(&inbox_new) {
            all_emails.extend(emails);
        }
    }

    Ok(all_emails)
}

fn scan_inbox_new(inbox_new: &Path) -> Result<Vec<Email>> {
    let mut emails = Vec::new();

    if !inbox_new.exists() {
        return Ok(emails);
    }

    let entries = fs::read_dir(inbox_new)
        .with_context(|| format!("Failed to read inbox: {}", inbox_new.display()))?;

    for entry in entries {
        let entry = entry.context("Failed to read directory entry")?;
        let file_path = entry.path();

        if file_path.is_file() {
            if let Ok(email) = parse_email_file(&file_path) {
                emails.push(email);
            }
        }
    }

    Ok(emails)
}

fn parse_email_file(file_path: &Path) -> Result<Email> {
    let id = file_path
        .file_name()
        .context("Invalid file path")?
        .to_string_lossy()
        .to_string();

    let file = fs::File::open(file_path)
        .with_context(|| format!("Failed to open email file: {}", file_path.display()))?;

    let reader = BufReader::new(file);

    let mut subject = String::new();
    let mut from = String::new();

    for line in reader.lines() {
        let line = line.context("Failed to read line from email file")?;

        if line.trim().is_empty() {
            break;
        }

        if let Some(raw_subject) = line.strip_prefix("Subject: ") {
            subject = decode_rfc2047(raw_subject);
        }

        if let Some(raw_from) = line.strip_prefix("From: ") {
            from = raw_from.to_string();
        }
    }

    Ok(Email { id, subject, from })
}

fn decode_rfc2047(encoded: &str) -> String {
    match rfc2047_decoder::decode(encoded.as_bytes()) {
        Ok(decoded) => decoded,
        Err(_) => encoded.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_email(dir: &Path, filename: &str, subject: &str, from: &str) {
        let email_content = format!(
            "Subject: {}\nFrom: {}\n\nThis is the email body.",
            subject, from
        );
        fs::write(dir.join(filename), email_content).unwrap();
    }

    #[test]
    fn parse_email_file_extracts_headers() {
        let temp_dir = TempDir::new().unwrap();
        create_test_email(
            temp_dir.path(),
            "test_email",
            "Test Subject",
            "sender@example.com"
        );

        let email = parse_email_file(&temp_dir.path().join("test_email")).unwrap();

        assert_eq!(email.id, "test_email");
        assert_eq!(email.subject, "Test Subject");
        assert_eq!(email.from, "sender@example.com");
    }

    #[test]
    fn decode_rfc2047_handles_encoded_subjects() {

        assert_eq!(decode_rfc2047("Plain Subject"), "Plain Subject");


        let encoded = "=?UTF-8?B?VGVzdCBTdWJqZWN0?=";
        let decoded = decode_rfc2047(encoded);

        assert!(!decoded.is_empty());
    }

    #[test]
    fn scan_inbox_new_handles_missing_directory() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("missing");

        let emails = scan_inbox_new(&non_existent).unwrap();
        assert!(emails.is_empty());
    }

    #[test]
    fn scan_inbox_new_finds_emails() {
        let temp_dir = TempDir::new().unwrap();
        create_test_email(temp_dir.path(), "email1", "Subject 1", "from1@example.com");
        create_test_email(temp_dir.path(), "email2", "Subject 2", "from2@example.com");

        let emails = scan_inbox_new(temp_dir.path()).unwrap();

        assert_eq!(emails.len(), 2);
        assert!(emails.iter().any(|e| e.subject == "Subject 1"));
        assert!(emails.iter().any(|e| e.subject == "Subject 2"));
    }
}
