use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext, email: String) -> Result<()> {
    if !email.contains('@') || email.starts_with('@') || email.ends_with('@') {
        eprintln!(
            "{}",
            format!("'{email}' doesn't look like a valid email address.").red()
        );
        return Ok(());
    }

    ctx.store.set_email(email.clone())?;
    println!("{} {}", "From-email set to:".green(), email);
    Ok(())
}
