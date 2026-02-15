use std::collections::HashMap;
use std::path::Path;

use proc_macro2::TokenStream;
use serde::Deserialize;

use crate::colour::Colour;
use crate::font_loader::{KerningData, LetterData, generate_font_tokens};
use crate::image_loader::Image;

#[derive(Deserialize)]
struct FontJson {
    #[serde(rename = "in-glyphs")]
    in_glyphs: Vec<String>,
    #[serde(rename = "glyph-width")]
    glyph_width: usize,
    #[serde(rename = "glyph-height")]
    glyph_height: usize,
    #[serde(rename = "glyph-ofs-x")]
    glyph_ofs_x: usize,
    #[serde(rename = "glyph-ofs-y")]
    glyph_ofs_y: usize,
    #[serde(rename = "glyph-sep-x")]
    glyph_sep_x: usize,
    #[serde(rename = "glyph-sep-y")]
    glyph_sep_y: usize,
    #[serde(rename = "glyph-baseline")]
    glyph_baseline: i32,
    #[serde(rename = "glyph-spacing")]
    glyph_spacing: i32,
    #[serde(rename = "font-is-mono")]
    font_is_mono: bool,
    #[serde(rename = "font-ascend")]
    font_ascend: i64,
    #[serde(rename = "font-descend")]
    font_descend: i64,
    #[serde(rename = "font-line-gap")]
    font_line_gap: i64,
    #[serde(rename = "font-px-size")]
    font_px_size: i64,
    #[serde(default)]
    overrides: Vec<String>,
}

struct GlyphMetrics {
    width: usize,
    height: usize,
    baseline: i32,
    spacing: i32,
    is_mono: bool,
}

pub fn load_font_from_json(json_data: &str, image_path: &Path) -> TokenStream {
    let (letters, line_height, ascent) = load_font_from_json_letters(json_data, image_path);
    generate_font_tokens(letters, line_height, ascent)
}

pub(crate) fn load_font_from_json_letters(
    json_data: &str,
    image_path: &Path,
) -> (Vec<LetterData>, i32, i32) {
    let font_json: FontJson = serde_json::from_str(json_data).expect("Failed to parse font JSON");

    let image = Image::load_from_file(image_path);

    let metrics = GlyphMetrics {
        width: font_json.glyph_width,
        height: font_json.glyph_height,
        baseline: font_json.glyph_baseline,
        spacing: font_json.glyph_spacing,
        is_mono: font_json.font_is_mono,
    };

    let mut letters: Vec<LetterData> = Vec::new();

    for (row_idx, row_str) in font_json.in_glyphs.iter().enumerate() {
        for (col_idx, c) in row_str.chars().enumerate() {
            let cell_x = font_json.glyph_ofs_x + col_idx * (metrics.width + font_json.glyph_sep_x);
            let cell_y = font_json.glyph_ofs_y + row_idx * (metrics.height + font_json.glyph_sep_y);

            let letter = extract_glyph(&image, c, cell_x, cell_y, &metrics);
            letters.push(letter);
        }
    }

    // Synthesize space character if not already extracted from the image
    let has_space = letters.iter().any(|l| l.character == ' ');
    if !has_space {
        letters.push(LetterData {
            character: ' ',
            width: 0,
            height: 0,
            xmin: 0,
            ymin: 0,
            advance_width: 3.0,
            rendered: vec![],
            kerning_data: Vec::new(),
        });
    }

    let char_to_idx: HashMap<char, usize> = letters
        .iter()
        .enumerate()
        .map(|(i, l)| (l.character, i))
        .collect();

    for line in &font_json.overrides {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("kern ") {
            parse_kern_override(rest, &char_to_idx, &mut letters);
        } else if let Some(rest) = line.strip_prefix("hide ") {
            let chars_to_hide = parse_glyphs(rest);
            for c in chars_to_hide {
                if let Some(&idx) = char_to_idx.get(&c) {
                    letters[idx].rendered.clear();
                    letters[idx].width = 0;
                    letters[idx].height = 0;
                }
            }
        }
    }

    // Sort kerning data for each letter
    for letter in &mut letters {
        letter
            .kerning_data
            .sort_unstable_by_key(|kd| kd.previous_character);
    }

    letters.sort_unstable_by_key(|l| l.character);

    let ascent = (font_json.font_ascend / font_json.font_px_size) as i32;
    let line_height = ((font_json.font_ascend - font_json.font_descend + font_json.font_line_gap)
        / font_json.font_px_size) as i32;

    (letters, line_height, ascent)
}

fn is_background(colour: Colour) -> bool {
    colour.is_transparent() || (colour.r == 255 && colour.g == 255 && colour.b == 255)
}

fn extract_glyph(
    image: &Image,
    character: char,
    cell_x: usize,
    cell_y: usize,
    metrics: &GlyphMetrics,
) -> LetterData {
    // Scan cell for bounding box of foreground pixels
    let mut top_row: Option<usize> = None;
    let mut bottom_row: usize = 0;
    let mut left_col: Option<usize> = None;
    let mut right_col: usize = 0;
    let mut has_pixels = false;

    for py in 0..metrics.height {
        for px in 0..metrics.width {
            let img_x = cell_x + px;
            let img_y = cell_y + py;

            if img_x < image.width
                && img_y < image.height
                && !is_background(image.colour(img_x, img_y))
            {
                if top_row.is_none() {
                    top_row = Some(py);
                }
                bottom_row = py;
                if left_col.is_none() || px < left_col.unwrap() {
                    left_col = Some(px);
                }
                if px > right_col {
                    right_col = px;
                }
                has_pixels = true;
            }
        }
    }

    if !has_pixels {
        let advance_width = if metrics.is_mono {
            (metrics.width as i32 + metrics.spacing) as f32
        } else {
            metrics.spacing as f32
        };
        return LetterData {
            character,
            width: 0,
            height: 0,
            xmin: 0,
            ymin: 0,
            advance_width,
            rendered: vec![],
            kerning_data: Vec::new(),
        };
    }

    let top = top_row.unwrap();
    let left = left_col.unwrap();
    let content_width = right_col - left + 1;
    let width = content_width.div_ceil(8) * 8;
    let height = bottom_row - top + 1;

    // Pack bitmap starting from left_col
    let mut rendered = Vec::with_capacity(height * (width / 8));
    for py in top..=bottom_row {
        for chunk_start in (0..width).step_by(8) {
            let mut byte = 0u8;
            for bit in 0..8 {
                let px = left + chunk_start + bit;
                if px <= right_col {
                    let img_x = cell_x + px;
                    let img_y = cell_y + py;
                    if img_x < image.width
                        && img_y < image.height
                        && !is_background(image.colour(img_x, img_y))
                    {
                        byte |= 1 << bit;
                    }
                }
            }
            rendered.push(byte);
        }
    }

    let xmin = left as i32;
    let ymin = metrics.baseline - height as i32 - top as i32;
    let advance_width = if metrics.is_mono {
        (metrics.width as i32 + metrics.spacing) as f32
    } else {
        (right_col as i32 + 1 + metrics.spacing) as f32
    };

    LetterData {
        character,
        width,
        height,
        xmin,
        ymin,
        advance_width,
        rendered,
        kerning_data: Vec::new(),
    }
}

fn parse_kern_override(rest: &str, char_to_idx: &HashMap<char, usize>, letters: &mut [LetterData]) {
    // Format: <left-glyphs> <right-glyphs> <x> [y]
    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 || parts.len() > 4 {
        return;
    }

    let left_glyphs = parse_glyphs(parts[0]);
    let right_glyphs = parse_glyphs(parts[1]);
    let amount: f32 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => return,
    };
    // parts[3] (y) is ignored — only horizontal kerning matters for GBA

    for right_c in right_glyphs {
        if let Some(&right_idx) = char_to_idx.get(&right_c) {
            for &left_c in &left_glyphs {
                letters[right_idx].kerning_data.push(KerningData {
                    previous_character: left_c,
                    amount,
                });
            }
        }
    }
}

/// Parse a glyph specification string, handling escape sequences per yal.cc docs:
/// - `\\` → backslash
/// - `\n`, `\r`, `\t` → newline, carriage return, tab
/// - `\s` → common Unicode space characters
/// - `\x12` → 2-digit hex code point
/// - `\u1234` → 4-digit hex code point
/// - `\u{123...}` → variable-length hex code point
/// - `\[a-z]` → character range (inclusive)
/// - Any other character → literal
fn parse_glyphs(spec: &str) -> Vec<char> {
    let mut result = Vec::new();
    let mut chars = spec.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }

        match chars.next() {
            Some('\\') => result.push('\\'),
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some('s') => {
                result.extend([
                    '\u{0020}', '\u{00A0}', '\u{2000}', '\u{2001}', '\u{2002}', '\u{2003}',
                    '\u{2004}', '\u{2005}', '\u{2006}', '\u{2007}', '\u{2008}', '\u{2009}',
                    '\u{200A}', '\u{202F}', '\u{205F}', '\u{3000}',
                ]);
            }
            Some('x') => {
                let hex: String = chars.by_ref().take(2).collect();
                if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                    result.push(ch);
                }
            }
            Some('u') => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    let hex: String = chars.by_ref().take_while(|&c| c != '}').collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        result.push(ch);
                    }
                } else {
                    let hex: String = chars.by_ref().take(4).collect();
                    if let Some(ch) = u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        result.push(ch);
                    }
                }
            }
            Some('[') => {
                let start = chars.next();
                let dash = chars.next();
                let end = chars.next();
                let close = chars.next();
                if let (Some(start), Some('-'), Some(end), Some(']')) = (start, dash, end, close) {
                    for code in (start as u32)..=(end as u32) {
                        if let Some(ch) = char::from_u32(code) {
                            result.push(ch);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_loader;
    use std::path::PathBuf;

    fn test_data_dir() -> PathBuf {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        PathBuf::from(manifest_dir)
            .parent()
            .unwrap()
            .join("agb/examples/font")
    }

    #[test]
    fn json_and_ttf_produce_same_output() {
        let dir = test_data_dir();

        // Load via TTF path
        let ttf_data =
            std::fs::read(dir.join("Dungeon Puzzler Font.ttf")).expect("Failed to read TTF file");
        let (ttf_letters, ttf_line_height, ttf_ascent) =
            font_loader::load_font_letters(&ttf_data, 8.0);

        // Load via JSON path
        let json_data = std::fs::read_to_string(dir.join("Dungeon Puzzler Font.json"))
            .expect("Failed to read JSON file");
        let aseprite_path = dir.join("font.aseprite");
        let (json_letters, json_line_height, json_ascent) =
            load_font_from_json_letters(&json_data, &aseprite_path);

        assert_eq!(ttf_line_height, json_line_height, "line_height mismatch");
        assert_eq!(ttf_ascent, json_ascent, "ascent mismatch");

        // Compare each character defined in the JSON output
        for json_letter in &json_letters {
            let ttf_letter = ttf_letters
                .iter()
                .find(|l| l.character == json_letter.character);

            let ttf_letter = match ttf_letter {
                Some(l) => l,
                None => panic!(
                    "character {:?} found in JSON but not in TTF",
                    json_letter.character
                ),
            };

            assert_eq!(
                json_letter, ttf_letter,
                "mismatch for character {:?}",
                json_letter.character
            );
        }
    }
}
