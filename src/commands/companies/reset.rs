use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::company::normalize_domain;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext, domain: String) -> Result<()> {
    let domain = normalize_domain(&domain);
    if ctx.store.reset_company(domain.clone())? {
        println!("{} {}", "Removed local company data for:".green(), domain);
    } else {
        println!(
            "{}",
            format!("No local company data found for {domain}.").yellow()
        );
    }
    Ok(())
}
