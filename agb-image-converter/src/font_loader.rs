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
