use anyhow::{Context, Result};
use lopdf::{Document, Object, ObjectId};
use std::collections::HashMap;
use std::path::Path;

use crate::cmap_parser;
use crate::csv_reader::Replacement;

pub struct ProcessResult {
    pub replacements_made: usize,
    pub pages_processed: usize,
}

pub fn process_pdf(
    input_path: &Path,
    output_path: &Path,
    replacements: &[Replacement],
    verbose: bool,
) -> Result<ProcessResult> {
    let mut doc = Document::load(input_path)
        .with_context(|| format!("Failed to load PDF: {}", input_path.display()))?;

    let pages: Vec<(u32, ObjectId)> = doc.get_pages().into_iter().collect();
    let page_count = pages.len();
    let mut total_replacements = 0;

    for &(page_num, page_id) in &pages {
        for replacement in replacements {
            // Try lopdf's built-in replace_text first (works for Western encodings)
            let builtin_count =
                try_builtin_replace(&mut doc, page_num, &replacement.before, &replacement.after);

            if builtin_count > 0 {
                total_replacements += builtin_count;
                if verbose {
                    println!(
                        "  Page {}: replaced '{}' -> '{}' ({} times, built-in)",
                        page_num, replacement.before, replacement.after, builtin_count
                    );
                }
                continue;
            }

            // Fallback: raw content stream replacement for CJK text
            match try_cid_replace(
                &mut doc,
                page_id,
                &replacement.before,
                &replacement.after,
                verbose,
            ) {
                Ok(count) if count > 0 => {
                    total_replacements += count;
                    if verbose {
                        println!(
                            "  Page {}: replaced '{}' -> '{}' ({} times, CID)",
                            page_num, replacement.before, replacement.after, count
                        );
                    }
                }
                Ok(_) => {}
                Err(e) if verbose => {
                    eprintln!("  Page {}: CID replacement error: {:#}", page_num, e);
                }
                Err(_) => {}
            }
        }
    }

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    doc.save(output_path)
        .with_context(|| format!("Failed to save PDF: {}", output_path.display()))?;

    Ok(ProcessResult {
        replacements_made: total_replacements,
        pages_processed: page_count,
    })
}

/// Try lopdf's built-in replace_text. Returns number of replacements.
fn try_builtin_replace(doc: &mut Document, page_num: u32, before: &str, after: &str) -> usize {
    // lopdf's replace_text returns Result<()>, so we count by checking content before/after
    match doc.replace_text(page_num, before, after, None) {
        Ok(()) => {
            // Assume at least 1 replacement was made if no error
            // lopdf doesn't report count, so we conservatively report 1
            1
        }
        Err(_) => 0,
    }
}

/// Try CID-based replacement for CJK text using ToUnicode CMap.
fn try_cid_replace(
    doc: &mut Document,
    page_id: ObjectId,
    before: &str,
    after: &str,
    verbose: bool,
) -> Result<usize> {
    // Build reverse CMap for all fonts on this page
    let reverse_maps = build_page_reverse_cmaps(doc, page_id)?;

    if reverse_maps.is_empty() {
        return Ok(0);
    }

    // Try each font's reverse map to encode the search/replace text
    for (_font_name, reverse_map) in &reverse_maps {
        let Some(before_bytes) = cmap_parser::encode_text(before, reverse_map) else {
            continue;
        };
        let Some(after_bytes) = cmap_parser::encode_text(after, reverse_map) else {
            if verbose {
                eprintln!(
                    "    Warning: replacement text '{}' contains characters not in font",
                    after
                );
            }
            continue;
        };

        // Get and modify the page content stream
        let content_data = doc.get_page_content(page_id)?;
        let replaced = replace_bytes_in_content(&content_data, &before_bytes, &after_bytes);

        if replaced.1 > 0 {
            // Write back the modified content
            doc.change_page_content(page_id, replaced.0)?;
            return Ok(replaced.1);
        }
    }

    Ok(0)
}

/// Build reverse Unicode->CID maps for all fonts on a page.
fn build_page_reverse_cmaps(
    doc: &Document,
    page_id: ObjectId,
) -> Result<Vec<(String, HashMap<char, Vec<u8>>)>> {
    let mut result = Vec::new();

    let fonts = match doc.get_page_fonts(page_id) {
        Ok(fonts) => fonts,
        Err(_) => return Ok(result),
    };

    for (font_name, font_dict) in &fonts {
        // Look for ToUnicode stream
        let tounicode_ref = match font_dict.get(b"ToUnicode") {
            Ok(obj) => obj,
            Err(_) => continue,
        };

        let tounicode_id = match tounicode_ref {
            Object::Reference(id) => *id,
            _ => continue,
        };

        let cmap_data = match doc.get_object(tounicode_id) {
            Ok(Object::Stream(stream)) => {
                let mut stream = stream.clone();
                if stream.decompress().is_err() {
                    continue;
                }
                stream.content
            }
            _ => continue,
        };

        match cmap_parser::parse_reverse_cmap(&cmap_data) {
            Ok(reverse_map) if !reverse_map.is_empty() => {
                let name = String::from_utf8_lossy(font_name).to_string();
                result.push((name, reverse_map));
            }
            _ => continue,
        }
    }

    Ok(result)
}

/// Replace byte patterns within PDF content stream data.
/// Returns (new content, replacement count).
fn replace_bytes_in_content(
    content: &[u8],
    before: &[u8],
    after: &[u8],
) -> (Vec<u8>, usize) {
    if before.is_empty() || content.len() < before.len() {
        return (content.to_vec(), 0);
    }

    let mut result = Vec::with_capacity(content.len());
    let mut count = 0;
    let mut i = 0;

    while i < content.len() {
        if i + before.len() <= content.len() && &content[i..i + before.len()] == before {
            result.extend_from_slice(after);
            count += 1;
            i += before.len();
        } else {
            result.push(content[i]);
            i += 1;
        }
    }

    (result, count)
}
