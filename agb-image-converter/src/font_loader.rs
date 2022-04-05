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

    let font = (0..128)
        .map(|i| font.rasterize(char::from_u32(i).unwrap(), pixels_per_em))
        .map(|(metrics, bitmap)| {
            let width = metrics.width;
            let height = metrics.height;
            LetterData {
                width,
                height,
                rendered: bitmap,
                xmin: metrics.xmin,
                ymin: metrics.ymin,
                advance_width: metrics.advance_width,
            }
        })
        .map(|letter_data| {
            let data_raw = ByteString(&letter_data.rendered);
            let height = letter_data.height as u8;
            let width = letter_data.width as u8;
            let xmin = letter_data.xmin as i8;
            let ymin = letter_data.ymin as i8;
            let advance_width = letter_data.advance_width as u8;

            quote!(
                agb::display::FontLetter::new(
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
        agb::display::Font::new(&[#(#font),*])
    ]
}
