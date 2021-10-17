use hound;
use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use syn::parse_macro_input;

#[proc_macro]
pub fn include_wav(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);

    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let include_path = path.to_string_lossy();

    let mut wav_reader = hound::WavReader::open(&path)
        .unwrap_or_else(|_| panic!("Failed to load file {}", include_path));

    assert_eq!(
        wav_reader.spec().sample_rate,
        10512,
        "agb currently only supports sample rate of 10512Hz"
    );

    let samples = wav_reader
        .samples::<i8>()
        .map(|sample| sample.unwrap() as u8);

    let result = quote! {
        {
            const _: &[u8] = include_bytes!(#include_path);

            &[
                #(#samples),*
            ]
        }
    };

    TokenStream::from(result)
}
