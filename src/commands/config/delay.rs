use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext, ms: u64) -> Result<()> {
    ctx.store.set_delay(ms as i64)?;
    println!("{} {}ms", "Delay set to:".green(), ms);
    if ms == 0 {
        println!(
            "{}",
            "Warning: a delay of 0 risks getting your IP greylisted on batch runs.".yellow()
        );
    }
    Ok(())
}
