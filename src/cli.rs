use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "pdf_replacer")]
#[command(about = "Replace text in PDF files using a CSV mapping")]
pub struct Cli {
    /// Path to a single PDF file or a folder containing PDFs
    pub input: PathBuf,

    /// Path to the CSV file with before/after columns
    #[arg(short, long, default_value = "replacements.csv")]
    pub csv: PathBuf,

    /// Output directory or file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Overwrite original files in place
    #[arg(long, default_value_t = false)]
    pub in_place: bool,

    /// Verbose output
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}
