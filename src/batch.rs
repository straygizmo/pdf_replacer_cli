use anyhow::{Context, Result};
use std::path::Path;
use walkdir::WalkDir;

use crate::csv_reader::Replacement;
use crate::pdf_processor;

pub struct BatchReport {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub errors: Vec<(String, String)>,
}

pub fn process_folder(
    input_dir: &Path,
    output_dir: &Path,
    replacements: &[Replacement],
    verbose: bool,
) -> Result<BatchReport> {
    let pdf_files: Vec<_> = WalkDir::new(input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("pdf"))
                    .unwrap_or(false)
        })
        .collect();

    if pdf_files.is_empty() {
        anyhow::bail!(
            "No PDF files found in directory: {}",
            input_dir.display()
        );
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;

    let mut report = BatchReport {
        total: pdf_files.len(),
        success: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for entry in &pdf_files {
        let input_path = entry.path();
        let relative = input_path
            .strip_prefix(input_dir)
            .unwrap_or(input_path.file_name().map(Path::new).unwrap_or(input_path));
        let output_path = output_dir.join(relative);

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let display_name = input_path.display().to_string();
        if verbose {
            println!("Processing: {}", display_name);
        }

        match pdf_processor::process_pdf(input_path, &output_path, replacements, verbose) {
            Ok(result) => {
                report.success += 1;
                if verbose {
                    println!(
                        "  -> {} replacements made, saved to {}",
                        result.replacements_made,
                        output_path.display()
                    );
                }
            }
            Err(e) => {
                report.failed += 1;
                let err_msg = format!("{:#}", e);
                eprintln!("  Error processing {}: {}", display_name, err_msg);
                report.errors.push((display_name, err_msg));
            }
        }
    }

    Ok(report)
}
