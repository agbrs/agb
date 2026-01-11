use crate::ByteString;
use quote::{ToTokens, quote};

use proc_macro2::TokenStream;

#[derive(Clone)]
struct KerningData {
    previous_character: char,
    amount: f32,
}

#[derive(Clone)]
struct LetterData {
    character: char,
    width: usize,
    height: usize,
    xmin: i32,
    ymin: i32,
    advance_width: f32,
    rendered: Vec<u8>,
    kerning_data: Vec<KerningData>,
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

pub fn load_font(font_data: &[u8], pixels_per_em: f32) -> TokenStream {
    let font = fontdue::Font::from_bytes(
        font_data,
        fontdue::FontSettings {
            scale: pixels_per_em,
            ..Default::default()
        },
    )
    .expect("Invalid font data");

    let line_metrics = font.horizontal_line_metrics(pixels_per_em).unwrap();

    let line_height = line_metrics.new_line_size as i32;
    let mut ascent = line_metrics.ascent as i32;

    let mut letters: Vec<_> = font
        .chars()
        .iter()
        .map(|(&c, &index)| (c, index, font.rasterize(c, pixels_per_em)))
        .map(|(c, index, (metrics, bitmap))| {
            let width = metrics.width.div_ceil(8) * 8;
            let height = metrics.height;

            let rendered = if bitmap.is_empty() {
                vec![]
            } else {
                bitmap
                    .chunks(metrics.width)
                    .flat_map(|row| {
                        row.chunks(8).map(|chunk| {
                            let mut output = 0u8;
                            for (i, &value) in chunk.iter().enumerate() {
                                if value > 100 {
                                    output |= 1 << i;
                                }
                            }

                            output
                        })
                    })
                    .collect()
            };

            let mut kerning_data: Vec<_> = font
                .chars()
                .iter()
                .filter_map(|(&left_char, &left_index)| {
                    let kerning = font.horizontal_kern_indexed(
                        left_index.into(),
                        index.into(),
                        pixels_per_em,
                    )?;

                    Some(KerningData {
                        previous_character: left_char,
                        amount: kerning,
                    })
                })
                .collect();

            kerning_data.sort_unstable_by_key(|kd| kd.previous_character);

            LetterData {
                character: c,
                width,
                height,
                rendered,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                advance_width: metrics.advance_width,
                kerning_data,
            }
        })
        .collect();

    letters.sort_unstable_by_key(|letter| letter.character);

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
