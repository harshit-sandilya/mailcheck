use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext) -> Result<()> {
    let email = ctx.store.get_email()?;
    let delay = ctx.store.get_delay()?;
    let patterns = ctx.store.get_patterns()?;

    println!("{}", "mailcheck info".bold());
    println!("{}", "-".repeat(30));
    println!("{:<12} {}", "From-email:".dimmed(), email);
    println!("{:<12} {}ms", "Delay:".dimmed(), delay);
    println!("{:<12} {}", "Patterns:".dimmed(), patterns.len());
    Ok(())
}
