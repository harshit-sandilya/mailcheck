use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::company::normalize_domain;
use crate::services::company;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext, domain: String) -> Result<()> {
    let domain = normalize_domain(&domain);
    let local = ctx.store.load_local_companies()?;
    let patterns = company::ranked_patterns(&domain, &local)?;

    if patterns.is_empty() {
        println!(
            "{}",
            format!("No company patterns found for {domain}.").yellow()
        );
        return Ok(());
    }

    println!("{}", format!("Company patterns for {domain}").bold());
    println!("{}", "-".repeat(60));
    for candidate in patterns {
        println!(
            "{:<24} {:>3}%  samples: {:<4} {}",
            candidate.pattern,
            candidate.confidence,
            candidate.samples,
            candidate.source.dimmed()
        );
    }
    Ok(())
}
