use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::services::company;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext) -> Result<()> {
    let local = ctx.store.load_local_companies()?;
    let domains = company::all_domains(&local)?;

    if domains.is_empty() {
        println!("{}", "No company-specific patterns configured.".yellow());
        return Ok(());
    }

    println!("{}", "Known companies".bold());
    println!("{}", "-".repeat(50));
    for (domain, source) in domains {
        let count = company::ranked_patterns(&domain, &local)?.len();
        println!("{domain:<32} {count:<3} {}", source.dimmed());
    }
    Ok(())
}
