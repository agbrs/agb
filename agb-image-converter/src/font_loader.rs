use crate::ByteString;
use quote::{ToTokens, quote};

use proc_macro2::TokenStream;

pub(crate) mod from_json;
pub(crate) mod from_ttf;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct KerningData {
    pub previous_character: char,
    pub amount: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LetterData {
    pub character: char,
    pub width: usize,
    pub height: usize,
    pub xmin: i32,
    pub ymin: i32,
    pub advance_width: f32,
    pub rendered: Vec<u8>,
    pub kerning_data: Vec<KerningData>,
}

impl ToTokens for LetterData {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let character = self.character;
        let data_raw = ByteString(&self.rendered);
        let height = self.height as u8;
        let width = self.width as u8;
        let xmin = self.xmin as i8;
        let ymin = self.ymin as i8;
        let advance_width = self.advance_width.ceil() as u8;
        let kerning_amounts = self.kerning_data.iter().map(|kerning_data| {
            let amount = kerning_data.amount as i8;
            let c = kerning_data.previous_character;
            quote! {
                (#c, #amount)
            }
        });

        quote!(
            FontLetter::new(
                #character,
                #width,
                #height,
                #data_raw,
                #xmin,
                #ymin,
                #advance_width,
                &[
                    #(#kerning_amounts),*
                ]
            )
        )
        .to_tokens(tokens);
    }
}

/// Pack a 2D grid of pixels into a 1-bit-per-pixel bitmap with rows padded to 8-pixel alignment.
/// `is_pixel_set(x, y)` determines whether each pixel is foreground.
pub(crate) fn pack_1bpp(
    content_width: usize,
    height: usize,
    is_pixel_set: impl Fn(usize, usize) -> bool,
) -> Vec<u8> {
    let width = content_width.div_ceil(8) * 8;
    let mut rendered = Vec::with_capacity(height * (width / 8));
    for y in 0..height {
        for chunk_start in (0..width).step_by(8) {
            let mut byte = 0u8;
            for bit in 0..8 {
                let px = chunk_start + bit;
                if px < content_width && is_pixel_set(px, y) {
                    byte |= 1 << bit;
                }
            }
            rendered.push(byte);
        }
    }
    rendered
}

pub(crate) fn generate_font_tokens(
    letters: Vec<LetterData>,
    line_height: i32,
    mut ascent: i32,
) -> TokenStream {
    let maximum_above_line = letters
        .iter()
        .map(|x| x.height as i32 + x.ymin)
        .max()
        .unwrap();

    if (ascent - maximum_above_line) < 0 {
        ascent = maximum_above_line;
    }

    let ascii_letters = (0x21..0x7F).map(|idx| {
        let c = char::from_u32(idx).expect("ascii character should be valid");
        let letter_idx = letters
            .binary_search_by_key(&c, |x| x.character)
            .unwrap_or(0);

        letters[letter_idx].clone()
    });

    let non_ascii_letters = letters
        .iter()
        .filter(|&x| !(0x21..0x7F).contains(&(x.character as u32)))
        .cloned();

    quote![
        Font::new(&[#(#ascii_letters),*], &[#(#non_ascii_letters),*], #line_height, #ascent)
    ]
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    fn test_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
    }

    #[test]
    fn json_and_ttf_produce_same_output() {
        let dir = test_data_dir();

        // Load via TTF path
        let ttf_data =
            std::fs::read(dir.join("Dungeon Puzzler Font.ttf")).expect("Failed to read TTF file");
        let (ttf_letters, ttf_line_height, ttf_ascent) =
            super::from_ttf::load_font_letters(&ttf_data, 8.0);

        // Load via JSON path
        let json_data = std::fs::read_to_string(dir.join("Dungeon Puzzler Font.json"))
            .expect("Failed to read JSON file");
        let aseprite_path = dir.join("font.aseprite");
        let (json_letters, json_line_height, json_ascent) =
            super::from_json::load_font_from_json_letters(&json_data, &aseprite_path);

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
