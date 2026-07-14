use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::patterns::validate_pattern;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext, pattern: String) -> Result<()> {
    if let Err(e) = validate_pattern(&pattern) {
        eprintln!("{}", e.red());
        return Ok(());
    }

    ctx.store.add_pattern(pattern.clone())?;
    println!("{} {}", "Added pattern:".green(), pattern);
    Ok(())
}
