use anyhow::{Context, Result};
use std::path::Path;

use crate::models::record::{InputRecord, OutputRecord};

/// Reads domain,first,last rows from a CSV file. Header row required
/// (columns: domain,first,last, any order).
pub fn read_input<P: AsRef<Path>>(path: P) -> Result<Vec<InputRecord>> {
    let mut reader = csv::Reader::from_path(&path)
        .with_context(|| format!("failed to open input CSV: {}", path.as_ref().display()))?;

    let mut records = Vec::new();
    for (i, row) in reader.deserialize::<InputRecord>().enumerate() {
        match row {
            Ok(r) => records.push(r),
            // one bad row shouldn't kill the whole batch
            Err(e) => eprintln!("skipping row {} (malformed): {e}", i + 2),
        }
    }
    Ok(records)
}

/// Writes results to CSV, including SMTP status and company-pattern evidence.
pub fn write_output<P: AsRef<Path>>(path: P, records: &[OutputRecord]) -> Result<()> {
    let mut writer = csv::Writer::from_path(&path)
        .with_context(|| format!("failed to create output CSV: {}", path.as_ref().display()))?;
    for record in records {
        writer.serialize(record)?;
    }
    writer.flush()?;
    Ok(())
}
