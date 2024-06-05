use std::path::Path;

use agb_midi_core::{parse_midi, MidiInfo};
use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

#[proc_macro_error]
#[proc_macro]
pub fn include_midi(args: TokenStream) -> TokenStream {
    agb_midi_core(args)
}

struct MidiCoreInput {
    sf2_file: LitStr,
    _comma: Token![,],
    midi_file: LitStr,
}

impl Parse for MidiCoreInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            sf2_file: input.parse()?,
            _comma: input.parse()?,
            midi_file: input.parse()?,
        })
    }
}

fn agb_midi_core(args: TokenStream) -> TokenStream {
    let input: MidiCoreInput = match syn::parse(args.clone()) {
        Ok(input) => input,
        Err(e) => abort!(proc_macro2::TokenStream::from(args), e),
    };

    let sf2_file = input.sf2_file.value();
    let midi_file = input.midi_file.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let sf2_file = Path::new(&root).join(&*sf2_file);
    let midi_file = Path::new(&root).join(&*midi_file);

    let sf2_include_path = sf2_file.to_string_lossy();
    let midi_file_include_path = midi_file.to_string_lossy();

    let midi_info = match MidiInfo::load_from_file(&sf2_file, &midi_file) {
        Ok(track) => track,
        Err(e) => abort!(proc_macro2::TokenStream::from(args), e),
    };

    let parsed = parse_midi(&midi_info);

    quote! {
        {
            const _: &[u8] = include_bytes!(#sf2_include_path);
            const _: &[u8] = include_bytes!(#midi_file_include_path);

            #parsed
        }
    }
    .into()
}
