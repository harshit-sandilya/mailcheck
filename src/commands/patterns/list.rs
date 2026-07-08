use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::patterns::BUILTIN_PATTERNS;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext) -> Result<()> {
    let patterns = ctx.store.get_patterns()?;

    if patterns.is_empty() {
        println!("{}", "No patterns configured.".yellow());
        return Ok(());
    }

    println!("{}", "Active patterns".bold());
    println!("{}", "-".repeat(30));
    for p in &patterns {
        let tag = if BUILTIN_PATTERNS.contains(&p.as_str()) {
            "".to_string()
        } else {
            " (custom)".cyan().to_string()
        };
        println!("{p}{tag}");
    }
    Ok(())
}
