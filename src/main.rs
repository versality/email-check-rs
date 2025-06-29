use anyhow::Result;
use clap::Parser;
use log::{info, debug};

mod config;
mod mail;
mod state;
mod sync;
mod notify;

use config::Config;
use state::SeenEmails;

#[derive(Parser)]
#[command(name = "email-check")]
#[command(about = "A simple email notification checker")]
struct Cli {
    #[arg(long)]
    no_sync: bool,
    
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    if cli.verbose {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::init();
    }
    
    let config = Config::load()?;
    debug!("Loaded config: {:?}", config);
    
    let mut seen_emails = SeenEmails::load(&config.state_file)?;
    debug!("Loaded {} seen email IDs", seen_emails.len());
    
    if !cli.no_sync {
        info!("Syncing mail...");
        sync::sync_mail()?;
    }
    
    let emails = mail::scan_maildir(&config.maildir)?;
    info!("Found {} total emails", emails.len());
    
    let new_emails: Vec<_> = emails
        .into_iter()
        .filter(|email| !seen_emails.contains(&email.id)) 
        .collect();

    info!("Found {} new emails", new_emails.len());
    
    if !new_emails.is_empty() {
        for email in &new_emails { 
            notify::send_notification(&email.subject, &email.from)?;
        }
        info!("Sent {} notifications", new_emails.len());
    }
    
    for email in new_emails { 
        seen_emails.add(email.id);
    }
    
    seen_emails.save(&config.state_file)?;
    info!("Mail check completed");

    Ok(())
}
