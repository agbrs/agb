use crate::ByteString;
use quote::quote;

use proc_macro2::TokenStream;

struct LetterData {
    width: usize,
    rendered: Vec<u8>,
}

pub fn load_font(font_data: &[u8], pixels_per_em: f32) -> TokenStream {
    let font = fontdue::Font::from_bytes(font_data, Default::default()).expect("Invalid font data");

    let font = (0..128)
        .map(|i| font.rasterize(char::from_u32(i).unwrap(), pixels_per_em))
        .map(|(metrics, bitmap)| {
            let width = metrics.width;
            LetterData {
                width,
                rendered: bitmap,
            }
        })
        .map(|letter_data| {
            let data_raw = ByteString(&letter_data.rendered);
            let width = letter_data.width as u8;

            quote!(agb::display::FontLetter {
                width: #width,
                data: #data_raw,
            })
        });

    quote![
        #(#font),*
    ]
}
