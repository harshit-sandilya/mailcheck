use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::record::OutputRecord;
use crate::services::{csv, smtp::SmtpService};
use crate::storage::store::Store;

pub async fn run(ctx: &AppContext, csv_path: String, out: Option<String>) -> Result<()> {
    let email = ctx.store.get_email()?;
    let delay = ctx.store.get_delay()?;
    let patterns = ctx.store.get_patterns()?;

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
        eprintln!(
            "{}",
            format!(
                "[{}/{}] checking {} {} @ {}...",
                i + 1,
                total,
                row.first,
                row.last,
                row.domain
            )
            .dimmed()
        );
        let results = smtp
            .check_all(&row.domain, &row.first, &row.last, &patterns)
            .await?;
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
        "{:<10} {:<12} {:<12} {:<32} {:<6} {}",
        "DOMAIN".bold(),
        "FIRST".bold(),
        "LAST".bold(),
        "EMAIL".bold(),
        "PASS".bold(),
        "REASON".bold()
    );
    println!("{}", "-".repeat(100));

    for r in results {
        let (mark, label) = if r.passed {
            ("✔".green().to_string(), "pass".green().to_string())
        } else {
            ("✘".red().to_string(), "fail".red().to_string())
        };
        println!(
            "{:<10} {:<12} {:<12} {:<32} {mark} {:<4} {}",
            r.domain,
            r.first_name,
            r.last_name,
            r.email,
            label,
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
