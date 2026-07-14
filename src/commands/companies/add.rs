use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::company::{normalize_domain, validate_domain};
use crate::models::patterns::validate_pattern;
use crate::storage::store::Store;

pub fn run(
    ctx: &AppContext,
    domain: String,
    pattern: String,
    confidence: u8,
    samples: u32,
) -> Result<()> {
    if let Err(error) = validate_domain(&domain) {
        eprintln!("{}", error.red());
        return Ok(());
    }
    if let Err(error) = validate_pattern(&pattern) {
        eprintln!("{}", error.red());
        return Ok(());
    }
    if confidence > 100 {
        eprintln!("{}", "Confidence must be between 0 and 100.".red());
        return Ok(());
    }
    if samples == 0 {
        eprintln!("{}", "Samples must be at least 1.".red());
        return Ok(());
    }

    let domain = normalize_domain(&domain);
    ctx.store
        .upsert_company_pattern(domain.clone(), pattern.clone(), confidence, samples)?;
    println!(
        "{} {} → {} (confidence: {}%, samples: {})",
        "Saved local company pattern:".green(),
        domain,
        pattern,
        confidence,
        samples
    );
    Ok(())
}
