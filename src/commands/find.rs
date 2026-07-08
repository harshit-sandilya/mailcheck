use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::models::record::OutputRecord;
use crate::services::smtp::SmtpService;
use crate::storage::store::Store;

pub async fn run(ctx: &AppContext, domain: String, first: String, last: String) -> Result<()> {
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

    let smtp = SmtpService::new(email, delay.max(0) as u64)?;
    let results = smtp.check_all(&domain, &first, &last, &patterns).await?;

    print_table(&results);
    Ok(())
}

fn print_table(results: &[OutputRecord]) {
    println!(
        "{:<32} {:<8} {}",
        "EMAIL".bold(),
        "RESULT".bold(),
        "REASON".bold()
    );
    println!("{}", "-".repeat(80));

    for r in results {
        let (mark, label) = if r.passed {
            ("✔".green().to_string(), "pass".green().to_string())
        } else {
            ("✘".red().to_string(), "fail".red().to_string())
        };
        println!("{:<32} {mark} {:<6} {}", r.email, label, r.reason.dimmed());
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
