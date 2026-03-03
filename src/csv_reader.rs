use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Replacement {
    pub before: String,
    pub after: String,
}

pub fn read_replacements(path: &Path) -> Result<Vec<Replacement>> {
    let raw_bytes = std::fs::read(path)
        .with_context(|| format!("Failed to read CSV file: {}", path.display()))?;

    let text = decode_text(&raw_bytes)?;

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(text.as_bytes());

    let mut replacements = Vec::new();
    for result in reader.deserialize() {
        let record: Replacement =
            result.with_context(|| "Failed to parse CSV row (expected 'before,after' columns)")?;
        if record.before.is_empty() {
            continue;
        }
        replacements.push(record);
    }

    if replacements.is_empty() {
        anyhow::bail!("No replacement rules found in CSV file: {}", path.display());
    }

    Ok(replacements)
}

fn decode_text(raw: &[u8]) -> Result<String> {
    // Strip UTF-8 BOM if present
    let raw = if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &raw[3..]
    } else {
        raw
    };

    // Try UTF-8 first
    if let Ok(text) = std::str::from_utf8(raw) {
        return Ok(text.to_string());
    }

    // Fallback to Shift-JIS (CP932)
    let (decoded, _, had_errors) = encoding_rs::SHIFT_JIS.decode(raw);
    if had_errors {
        anyhow::bail!("Failed to decode CSV file as UTF-8 or Shift-JIS");
    }

    Ok(decoded.into_owned())
}
