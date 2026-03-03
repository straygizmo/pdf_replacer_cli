use anyhow::Result;
use std::collections::HashMap;

/// Parse a ToUnicode CMap stream and build a reverse map: Unicode char -> CID bytes.
///
/// The CMap maps CID byte sequences to Unicode code points. We invert this
/// so we can encode Unicode text back into CID byte sequences for replacement.
pub fn parse_reverse_cmap(cmap_data: &[u8]) -> Result<HashMap<char, Vec<u8>>> {
    let text = String::from_utf8_lossy(cmap_data);
    let mut reverse_map = HashMap::new();

    parse_bfchar_sections(&text, &mut reverse_map);
    parse_bfrange_sections(&text, &mut reverse_map);

    Ok(reverse_map)
}

/// Parse `beginbfchar` / `endbfchar` sections.
///
/// Format:
/// ```text
/// N beginbfchar
/// <CID_hex> <Unicode_hex>
/// endbfchar
/// ```
fn parse_bfchar_sections(text: &str, map: &mut HashMap<char, Vec<u8>>) {
    let mut rest = text;
    while let Some(start) = rest.find("beginbfchar") {
        let section_start = start + "beginbfchar".len();
        let Some(end) = rest[section_start..].find("endbfchar") else {
            break;
        };
        let section = &rest[section_start..section_start + end];

        for line in section.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let tokens: Vec<&str> = extract_hex_tokens(line);
            if tokens.len() >= 2 {
                let cid_bytes = hex_to_bytes(tokens[0]);
                let unicode_chars = hex_to_unicode(tokens[1]);
                for ch in unicode_chars {
                    map.insert(ch, cid_bytes.clone());
                }
            }
        }

        rest = &rest[section_start + end + "endbfchar".len()..];
    }
}

/// Parse `beginbfrange` / `endbfrange` sections.
///
/// Format:
/// ```text
/// N beginbfrange
/// <CID_start> <CID_end> <Unicode_start>
/// endbfrange
/// ```
fn parse_bfrange_sections(text: &str, map: &mut HashMap<char, Vec<u8>>) {
    let mut rest = text;
    while let Some(start) = rest.find("beginbfrange") {
        let section_start = start + "beginbfrange".len();
        let Some(end) = rest[section_start..].find("endbfrange") else {
            break;
        };
        let section = &rest[section_start..section_start + end];

        for line in section.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let tokens: Vec<&str> = extract_hex_tokens(line);
            if tokens.len() >= 3 {
                let cid_start = hex_to_u32(tokens[0]);
                let cid_end = hex_to_u32(tokens[1]);
                let byte_len = tokens[0].len() / 2; // number of bytes in CID

                // Check if third token is an array [...] or a single hex value
                if tokens[2].starts_with('[') {
                    // Array form: <start> <end> [<u1> <u2> ...]
                    // Not common, skip for now
                    continue;
                }

                let unicode_start = hex_to_u32(tokens[2]);

                for offset in 0..=(cid_end.saturating_sub(cid_start)) {
                    let cid_val = cid_start + offset;
                    let unicode_val = unicode_start + offset;

                    if let Some(ch) = char::from_u32(unicode_val) {
                        let cid_bytes = u32_to_bytes(cid_val, byte_len);
                        map.insert(ch, cid_bytes);
                    }
                }
            }
        }

        rest = &rest[section_start + end + "endbfrange".len()..];
    }
}

/// Extract hex tokens from angle-bracket delimited strings.
/// E.g., "<0041> <0061>" -> ["0041", "0061"]
fn extract_hex_tokens(line: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('<') {
        if let Some(close) = rest[open + 1..].find('>') {
            tokens.push(&rest[open + 1..open + 1 + close]);
            rest = &rest[open + 1 + close + 1..];
        } else {
            break;
        }
    }
    tokens
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    let hex = hex.trim();
    (0..hex.len())
        .step_by(2)
        .filter_map(|i| {
            if i + 2 <= hex.len() {
                u8::from_str_radix(&hex[i..i + 2], 16).ok()
            } else {
                None
            }
        })
        .collect()
}

fn hex_to_u32(hex: &str) -> u32 {
    u32::from_str_radix(hex.trim(), 16).unwrap_or(0)
}

fn hex_to_unicode(hex: &str) -> Vec<char> {
    let bytes = hex_to_bytes(hex);
    // Interpret as big-endian UTF-16
    let mut chars = Vec::new();
    let mut i = 0;
    while i + 1 < bytes.len() {
        let code = u16::from_be_bytes([bytes[i], bytes[i + 1]]);
        if let Some(ch) = char::from_u32(code as u32) {
            chars.push(ch);
        }
        i += 2;
    }
    chars
}

fn u32_to_bytes(val: u32, byte_len: usize) -> Vec<u8> {
    let all_bytes = val.to_be_bytes();
    let start = 4 - byte_len.min(4);
    all_bytes[start..].to_vec()
}

/// Encode a Unicode string into CID bytes using the reverse CMap.
/// Returns None if any character is not found in the map.
pub fn encode_text(text: &str, reverse_map: &HashMap<char, Vec<u8>>) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    for ch in text.chars() {
        let cid = reverse_map.get(&ch)?;
        result.extend_from_slice(cid);
    }
    Some(result)
}
