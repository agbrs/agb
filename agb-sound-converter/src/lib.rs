#![deny(clippy::all)]

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{ToTokens, quote};
use std::path::Path;
use syn::parse_macro_input;

use quote::TokenStreamExt;
struct ByteString<'a>(&'a [u8]);
impl ToTokens for ByteString<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(Literal::byte_string(self.0));
    }
}

#[proc_macro]
pub fn include_wav(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);

    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let include_path = path.to_string_lossy();

    let wav_reader = hound::WavReader::open(&path)
        .unwrap_or_else(|_| panic!("Failed to load file {include_path}"));

    let samples: Vec<u8> = samples_from_reader(wav_reader).collect();
    let samples = ByteString(&samples);

    let result = quote! {
        {
            #[repr(align(4))]
            struct AlignmentWrapper<const N: usize>([u8; N]);

            const _: &[u8] = include_bytes!(#include_path);

            &AlignmentWrapper(*#samples).0
        }
    };

    TokenStream::from(result)
}

fn samples_from_reader<'a, R>(reader: hound::WavReader<R>) -> Box<dyn Iterator<Item = u8> + 'a>
where
    R: std::io::Read + 'a,
{
    let bitrate = reader.spec().bits_per_sample;
    let reduction = bitrate - 8;

    match reader.spec().sample_format {
        hound::SampleFormat::Float => Box::new(
            reader
                .into_samples::<f32>()
                .map(|sample| (sample.unwrap() * (i8::MAX as f32)) as u8),
        ),
        hound::SampleFormat::Int => Box::new(
            reader
                .into_samples::<i32>()
                .map(move |sample| (sample.unwrap() >> reduction) as u8),
        ),
    }
}
