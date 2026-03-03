# pdf_replacer_cli

A command-line tool for batch text replacement in PDF files. Reads replacement rules from a CSV file and applies them to one or more PDFs.

## Features

- Single file and batch folder processing
- CSV-based replacement rules (supports UTF-8 and Shift-JIS)
- Japanese (CJK) text replacement via CID font reverse mapping
- Non-destructive output by default (original files preserved)

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/pdf_replacer_cli.exe`.

## Usage

```
pdf_replacer_cli [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to a PDF file or a folder containing PDFs

Options:
  -c, --csv <CSV>        Path to the CSV file [default: replacements.csv]
  -o, --output <OUTPUT>  Output file or directory path
      --in-place         Overwrite original files
  -v, --verbose          Verbose output
  -h, --help             Print help
```

### Examples

```bash
# Single file (outputs to input_replaced.pdf)
pdf_replacer_cli input.pdf

# Folder batch processing (outputs to ./pdfs_replaced/)
pdf_replacer_cli ./pdfs/

# Custom CSV and output path
pdf_replacer_cli input.pdf -c rules.csv -o output.pdf

# Verbose mode
pdf_replacer_cli input.pdf -v

# Overwrite originals
pdf_replacer_cli input.pdf --in-place
```

## CSV Format

Place a file named `replacements.csv` in the current directory. The file must have a header row with two columns:

```csv
before,after
Old Company,New Company
foo,bar
```

Both UTF-8 (with or without BOM) and Shift-JIS encoded CSV files are supported.

## Limitations

- Only works with text-based PDFs (not scanned/image PDFs)
- Replacement text must use characters present in the original font
- Encrypted PDFs may not be fully supported
- Text split across multiple PDF operators may not be matched
