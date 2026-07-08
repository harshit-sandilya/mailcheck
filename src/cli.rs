use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mailcheck", version, about = "SMTP mailchecking utility")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Find {
        domain: String,
        first: String,
        last: String,
    },

    FindAll {
        csv: String,
        #[arg(short, long)]
        out: Option<String>,
    },

    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    Patterns {
        #[command(subcommand)]
        action: PatternAction,
    },

    Info,
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
