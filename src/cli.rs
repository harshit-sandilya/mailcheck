use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mailcheck", version, about = "SMTP mailchecking utility")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Find likely addresses and check them against the domain's mail server.
    Find {
        domain: String,
        first: String,
        last: String,
    },

    /// Check people from a CSV file sequentially.
    FindAll {
        csv: String,
        #[arg(short, long)]
        out: Option<String>,
    },

    /// Change the SMTP sender and delay settings.
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Manage global fallback address patterns.
    Patterns {
        #[command(subcommand)]
        action: PatternAction,
    },

    /// Manage community and local company-specific patterns.
    Companies {
        #[command(subcommand)]
        action: CompanyAction,
    },

    /// Show the active configuration.
    Info,
    /// Replace this binary with the latest GitHub release.
    Update,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    SetEmail { email: String },
    SetDelay { ms: u64 },
}

#[derive(Subcommand)]
pub enum PatternAction {
    List,
    Add { pattern: String },
    Remove { pattern: String },
    Reset,
}

#[derive(Subcommand)]
pub enum CompanyAction {
    /// List domains with company-specific pattern data.
    List,
    /// Show ranked patterns for one domain.
    Show { domain: String },
    /// Add or override a pattern in the local registry.
    Add {
        domain: String,
        pattern: String,
        #[arg(short, long)]
        confidence: u8,
        #[arg(short, long, default_value_t = 1)]
        samples: u32,
    },
    /// Remove all local overrides for a domain.
    Reset { domain: String },
}
