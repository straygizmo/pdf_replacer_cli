mod batch;
mod cli;
mod cmap_parser;
mod csv_reader;
mod pdf_processor;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    // Read CSV replacements
    let replacements = csv_reader::read_replacements(&cli.csv)?;
    println!("Loaded {} replacement rules from {}", replacements.len(), cli.csv.display());

    if cli.verbose {
        for r in &replacements {
            println!("  '{}' -> '{}'", r.before, r.after);
        }
    }

    // Determine mode: single file or batch
    if cli.input.is_file() {
        let output_path = determine_output_file(&cli.input, &cli.output, cli.in_place);
        let result =
            pdf_processor::process_pdf(&cli.input, &output_path, &replacements, cli.verbose)?;
        println!(
            "Done: {} pages processed, {} replacements made",
            result.pages_processed, result.replacements_made
        );
        println!("Output: {}", output_path.display());
    } else if cli.input.is_dir() {
        let output_dir = determine_output_dir(&cli.input, &cli.output, cli.in_place);
        let report =
            batch::process_folder(&cli.input, &output_dir, &replacements, cli.verbose)?;
        println!();
        println!("Batch complete:");
        println!("  Total:   {}", report.total);
        println!("  Success: {}", report.success);
        println!("  Failed:  {}", report.failed);
        if !report.errors.is_empty() {
            println!("  Errors:");
            for (file, err) in &report.errors {
                println!("    {}: {}", file, err);
            }
        }
        println!("Output directory: {}", output_dir.display());
    } else {
        anyhow::bail!("Input path does not exist: {}", cli.input.display());
    }

    Ok(())
}

fn determine_output_file(input: &PathBuf, output: &Option<PathBuf>, in_place: bool) -> PathBuf {
    if in_place {
        return input.clone();
    }
    if let Some(out) = output {
        return out.clone();
    }
    // Default: input_replaced.pdf
    let stem = input
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");
    let ext = input
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("pdf");
    input.with_file_name(format!("{}_replaced.{}", stem, ext))
}

fn determine_output_dir(input: &PathBuf, output: &Option<PathBuf>, in_place: bool) -> PathBuf {
    if in_place {
        return input.clone();
    }
    if let Some(out) = output {
        return out.clone();
    }
    // Default: input_dir_replaced/
    input.with_file_name(format!(
        "{}_replaced",
        input
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
    ))
}
