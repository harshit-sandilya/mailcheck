use anyhow::Result;
use colored::*;

use crate::app::context::AppContext;
use crate::storage::store::Store;

pub fn run(ctx: &AppContext) -> Result<()> {
    ctx.store.reset_patterns()?;
    println!("{}", "Patterns reset to built-in defaults.".green());
    Ok(())
}
