use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

pub struct SeenEmails {
    ids: HashSet<String>, 
}

impl SeenEmails {
    pub fn load(path: &Path) -> Result<Self> { 
        let ids = if path.exists() {
            
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read state file: {}", path.display()))?;
            
            content
                .lines()
                .filter(|line| !line.trim().is_empty()) 
                .map(|line| line.to_string()) 
                .collect()
        } else {
            HashSet::new()
        };

        Ok(SeenEmails { ids })
    }
    
    pub fn save(&self, path: &Path) -> Result<()> { 
        let content = self.ids
            .iter() 
            .map(|id| id.as_str()) 
            .collect::<Vec<_>>()
            .join("\n");
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write state file: {}", path.display()))?;

        Ok(())
    }
    
    pub fn contains(&self, id: &str) -> bool { 
        self.ids.contains(id)
    }
    
    pub fn add(&mut self, id: String) { 
        self.ids.insert(id);
    }
    
    pub fn len(&self) -> usize { 
        self.ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn seen_emails_starts_empty_when_file_missing() {
        let temp_file = NamedTempFile::new().unwrap();
        let non_existent_path = temp_file.path().with_extension("missing");
        
        let seen = SeenEmails::load(&non_existent_path).unwrap();
        assert_eq!(seen.len(), 0);
    }

    #[test]
    fn seen_emails_loads_from_existing_file() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "email1\nemail2\nemail3").unwrap();
        
        let seen = SeenEmails::load(temp_file.path()).unwrap();
        assert_eq!(seen.len(), 3);
        assert!(seen.contains("email1"));
        assert!(seen.contains("email2"));
        assert!(seen.contains("email3"));
    }

    #[test]
    fn seen_emails_can_add_and_save() {
        let temp_file = NamedTempFile::new().unwrap();
        
        let mut seen = SeenEmails::load(temp_file.path()).unwrap();
        seen.add("new_email".to_string());
        seen.save(temp_file.path()).unwrap();
        
        
        let seen2 = SeenEmails::load(temp_file.path()).unwrap();
        assert!(seen2.contains("new_email"));
    }

    #[test]
    fn seen_emails_ignores_empty_lines() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "email1\n\nemail2\n\n").unwrap();
        
        let seen = SeenEmails::load(temp_file.path()).unwrap();
        assert_eq!(seen.len(), 2);
    }
}

