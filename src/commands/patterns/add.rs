use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::storage::store::Store;

const VALID_TOKENS: [&str; 4] = ["{first}", "{last}", "{f}", "{l}"];

/// Rejects patterns with an unknown {token} or an unmatched brace, so a typo
/// doesn't silently produce a literal "{firsst}" candidate later.
fn validate(pattern: &str) -> Result<(), String> {
    let mut chars = pattern.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c == '{' {
            let end = pattern[i..]
                .find('}')
                .ok_or_else(|| format!("unmatched '{{' in pattern: {pattern}"))?;
            let token = &pattern[i..i + end + 1];
            if !VALID_TOKENS.contains(&token) {
                return Err(format!(
                    "unknown token '{token}' — valid tokens are {}",
                    VALID_TOKENS.join(", ")
                ));
            }
        }
    }
    Ok(())
}

pub fn run(ctx: &AppContext, pattern: String) -> Result<()> {
    if let Err(e) = validate(&pattern) {
        eprintln!("{}", e.red());
        return Ok(());
    }

    ctx.store.add_pattern(pattern.clone())?;
    println!("{} {}", "Added pattern:".green(), pattern);
    Ok(())
}
