use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::company::{normalize_domain, validate_domain};
use crate::models::record::OutputRecord;
use crate::services::{company, csv, smtp::SmtpService};
use crate::storage::store::Store;

pub async fn run(ctx: &AppContext, csv_path: String, out: Option<String>) -> Result<()> {
    let email = ctx.store.get_email()?;
    let delay = ctx.store.get_delay()?;
    let patterns = ctx.store.get_patterns()?;
    let local_companies = ctx.store.load_local_companies()?;

    if patterns.is_empty() {
        eprintln!(
            "{}",
            "No patterns configured — try `mailcheck patterns reset`.".red()
        );
        return Ok(());
    }

    let input = csv::read_input(&csv_path)?;
    if input.is_empty() {
        eprintln!("{}", "No valid rows found in input CSV.".yellow());
        return Ok(());
    }

    let smtp = SmtpService::new(email, delay.max(0) as u64)?;

    let mut all_results: Vec<OutputRecord> = Vec::new();
    let total = input.len();
    for (i, row) in input.iter().enumerate() {
        if let Err(error) = validate_domain(&row.domain) {
            eprintln!(
                "{}",
                format!("[{}/{}] skipping {}: {error}", i + 1, total, row.domain).yellow()
            );
            continue;
        }
        let domain = normalize_domain(&row.domain);
        eprintln!(
            "{}",
            format!(
                "[{}/{}] checking {} {} @ {}...",
                i + 1,
                total,
                row.first,
                row.last,
                domain
            )
            .dimmed()
        );
        let ranked = company::ranked_patterns(&domain, &local_companies)?;
        let effective_patterns = company::merge_with_fallbacks(&ranked, &patterns);
        let mut results = smtp
            .check_all(&domain, &row.first, &row.last, &effective_patterns)
            .await?;
        company::annotate_results(&mut results, &ranked, &row.first, &row.last);
        all_results.extend(results);
    }

    match out {
        Some(path) => {
            csv::write_output(&path, &all_results)?;
            println!(
                "{} {} ({} rows)",
                "Wrote results to:".green(),
                path,
                all_results.len()
            );
        }
        None => print_table(&all_results),
    }

    Ok(())
}

fn print_table(results: &[OutputRecord]) {
    println!(
        "{:<10} {:<12} {:<12} {:<32} {:<14} {:<7} {}",
        "DOMAIN".bold(),
        "FIRST".bold(),
        "LAST".bold(),
        "EMAIL".bold(),
        "STATUS".bold(),
        "CONF".bold(),
        "REASON".bold()
    );
    println!("{}", "-".repeat(112));

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
            "{:<10} {:<12} {:<12} {:<32} {mark} {:<12} {:<7} {}",
            r.domain,
            r.first_name,
            r.last_name,
            r.email,
            label,
            confidence,
            r.reason.dimmed()
        );
    }

    let hits = results.iter().filter(|r| r.passed).count();
    println!();
    println!(
        "{} {}/{} candidates confirmed",
        "Summary:".bold(),
        hits,
        results.len()
    );
}
