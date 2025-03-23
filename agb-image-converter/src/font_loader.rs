use crate::ByteString;
use quote::quote;

use proc_macro2::TokenStream;

struct KerningData {
    previous_character: char,
    amount: f32,
}

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
            let width = metrics.width;
            let height = metrics.height;

            let rendered = bitmap
                .chunks(8)
                .map(|chunk| {
                    let mut output = 0u8;
                    for (i, &value) in chunk.iter().enumerate() {
                        if value > 100 {
                            output |= 1 << i;
                        }
                    }

                    output
                })
                .collect();

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
        .map(|x| (x.height as i32 + x.ymin))
        .max()
        .unwrap();

    if (ascent - maximum_above_line) < 0 {
        ascent = maximum_above_line;
    }

    let font = letters.iter().map(|letter_data| {
        let character = letter_data.character;
        let data_raw = ByteString(&letter_data.rendered);
        let height = letter_data.height as u8;
        let width = letter_data.width as u8;
        let xmin = letter_data.xmin as i8;
        let ymin = letter_data.ymin as i8;
        let advance_width = letter_data.advance_width.ceil() as u8;
        let kerning_amounts = letter_data.kerning_data.iter().map(|kerning_data| {
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
    });

    quote![
        Font::new(&[#(#font),*], #line_height, #ascent)
    ]
}
