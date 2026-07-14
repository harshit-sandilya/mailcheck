mod app;
mod cli;
mod commands;
mod models;
mod services;
mod storage;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;

use app::context::AppContext;
use cli::{Cli, Commands, CompanyAction, ConfigAction, PatternAction};
use storage::file_store::FileStore;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let store = Arc::new(FileStore::default());
    let ctx = AppContext { store };

    match cli.command {
        Commands::Find {
            domain,
            first,
            last,
        } => commands::find::run(&ctx, domain, first, last).await,
        Commands::FindAll { csv, out } => commands::findall::run(&ctx, csv, out).await,
        Commands::Config { action } => match action {
            ConfigAction::SetEmail { email } => commands::config::email::run(&ctx, email),
            ConfigAction::SetDelay { ms } => commands::config::delay::run(&ctx, ms),
        },
        Commands::Patterns { action } => match action {
            PatternAction::List => commands::patterns::list::run(&ctx),
            PatternAction::Add { pattern } => commands::patterns::add::run(&ctx, pattern),
            PatternAction::Remove { pattern } => commands::patterns::remove::run(&ctx, pattern),
            PatternAction::Reset => commands::patterns::reset::run(&ctx),
        },
        Commands::Companies { action } => match action {
            CompanyAction::List => commands::companies::list::run(&ctx),
            CompanyAction::Show { domain } => commands::companies::show::run(&ctx, domain),
            CompanyAction::Add {
                domain,
                pattern,
                confidence,
                samples,
            } => commands::companies::add::run(&ctx, domain, pattern, confidence, samples),
            CompanyAction::Reset { domain } => commands::companies::reset::run(&ctx, domain),
        },
        Commands::Info => commands::info::run(&ctx),
        Commands::Update => commands::update::run().await,
    }?;

    Ok(())
}
