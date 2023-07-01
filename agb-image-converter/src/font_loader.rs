use crate::ByteString;
use quote::quote;

use proc_macro2::TokenStream;

struct LetterData {
    width: usize,
    height: usize,
    xmin: i32,
    ymin: i32,
    advance_width: f32,
    rendered: Vec<u8>,
}

pub fn load_font(font_data: &[u8], pixels_per_em: f32) -> TokenStream {
    let font = fontdue::Font::from_bytes(
        font_data,
        fontdue::FontSettings {
            collection_index: 0,
            scale: pixels_per_em,
        },
    )
    .expect("Invalid font data");

    let line_metrics = font.horizontal_line_metrics(pixels_per_em).unwrap();

    let line_height = line_metrics.new_line_size as i32;
    let mut ascent = line_metrics.ascent as i32;

    let letters: Vec<_> = (0..128)
        .map(|i| font.rasterize(char::from_u32(i).unwrap(), pixels_per_em))
        .map(|(metrics, bitmap)| {
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

            LetterData {
                width,
                height,
                rendered,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                advance_width: metrics.advance_width,
            }
        })
        .collect();

    let maximum_above_line = letters
        .iter()
        .map(|x| (x.height as i32 + x.ymin))
        .max()
        .unwrap();

    if (ascent - maximum_above_line) < 0 {
        ascent = maximum_above_line;
    }

    let font = letters.iter().map(|letter_data| {
        let data_raw = ByteString(&letter_data.rendered);
        let height = letter_data.height as u8;
        let width = letter_data.width as u8;
        let xmin = letter_data.xmin as i8;
        let ymin = letter_data.ymin as i8;
        let advance_width = letter_data.advance_width.ceil() as u8;

        quote!(
            display::FontLetter::new(
                #width,
                #height,
                #data_raw,
                #xmin,
                #ymin,
                #advance_width,
            )
        )
    });

    quote![
        display::Font::new(&[#(#font),*], #line_height, #ascent)
    ]
}
