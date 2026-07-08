use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext, pattern: String) -> Result<()> {
    let before = ctx.store.get_patterns()?;
    if !before.contains(&pattern) {
        eprintln!("{}", format!("Pattern not found: {pattern}").yellow());
        return Ok(());
    }

    ctx.store.remove_pattern(pattern.clone())?;
    println!("{} {}", "Removed pattern:".green(), pattern);
    Ok(())
}
