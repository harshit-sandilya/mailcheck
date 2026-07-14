use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::company::{normalize_domain, validate_domain};
use crate::models::record::OutputRecord;
use crate::services::{company, smtp::SmtpService};
use crate::storage::store::Store;

pub async fn run(ctx: &AppContext, domain: String, first: String, last: String) -> Result<()> {
    if let Err(error) = validate_domain(&domain) {
        eprintln!("{}", error.red());
        return Ok(());
    }
    let domain = normalize_domain(&domain);
    let email = ctx.store.get_email()?;
    let delay = ctx.store.get_delay()?;
    let patterns = ctx.store.get_patterns()?;
    let local_companies = ctx.store.load_local_companies()?;
    let ranked = company::ranked_patterns(&domain, &local_companies)?;
    let patterns = company::merge_with_fallbacks(&ranked, &patterns);

    if patterns.is_empty() {
        eprintln!(
            "{}",
            "No patterns configured — try `mailcheck patterns reset`.".red()
        );
        return Ok(());
    }

    let smtp = SmtpService::new(email, delay.max(0) as u64)?;
    let mut results = smtp.check_all(&domain, &first, &last, &patterns).await?;
    company::annotate_results(&mut results, &ranked, &first, &last);

    print_table(&results);
    Ok(())
}

fn print_table(results: &[OutputRecord]) {
    println!(
        "{:<32} {:<14} {:<7} {}",
        "EMAIL".bold(),
        "RESULT".bold(),
        "CONF".bold(),
        "REASON".bold()
    );
    println!("{}", "-".repeat(80));

    for r in results {
        let (mark, label) = match r.status.as_str() {
            "confirmed" => ("✔".green().to_string(), r.status.green().to_string()),
            "rejected" => ("✘".red().to_string(), r.status.red().to_string()),
            _ => ("?".yellow().to_string(), r.status.yellow().to_string()),
        };
        let confidence = r
            .confidence
            .map(|value| format!("{value}%"))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<32} {mark} {:<12} {:<7} {}",
            r.email,
            label,
            confidence,
            r.reason.dimmed()
        );
    }

    let hits: Vec<&OutputRecord> = results.iter().filter(|r| r.passed).collect();
    println!();
    if hits.is_empty() {
        println!("{}", "No confirmed email found.".yellow());
    } else {
        println!(
            "{} {}",
            "Confirmed:".green().bold(),
            hits.iter()
                .map(|r| r.email.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
}
