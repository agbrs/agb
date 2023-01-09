#![deny(clippy::all)]

mod mmutil;
mod mmutil_sys;

use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::{format_ident, quote, ToTokens};
use std::path::{Path, PathBuf};
use syn::{parse::Parser, parse_macro_input, punctuated::Punctuated, LitStr};

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
        .unwrap_or_else(|_| panic!("Failed to load file {}", include_path));

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

#[proc_macro]
pub fn include_sounds(input: TokenStream) -> TokenStream {
    let parser = Punctuated::<LitStr, syn::Token![,]>::parse_separated_nonempty;
    let parsed = match parser.parse(input) {
        Ok(e) => e,
        Err(e) => return e.to_compile_error().into(),
    };

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");

    let filenames: Vec<PathBuf> = parsed
        .iter()
        .map(|s| s.value())
        .map(|s| Path::new(&root).join(&*s))
        .collect();

    let mm_converted = mmutil::mm_convert(&filenames);

    let mod_files = mm_converted.constants.iter().filter_map(|(name, value)| {
        let name_ident = format_ident!("{}", name);
        let value = *value as isize;

        if name.starts_with("MOD_") {
            Some(quote! {
                #name_ident = #value,
            })
        } else {
            None
        }
    });

    let sfx_files = mm_converted.constants.iter().filter_map(|(name, value)| {
        let name_ident = format_ident!("{}", name);
        let value = *value as isize;

        if name.starts_with("SFX_") {
            Some(quote! {
                #name_ident = #value,
            })
        } else {
            None
        }
    });

    let include_files = filenames.iter().map(|filename| {
        let filename = filename.to_string_lossy();
        quote! { const _: &[u8] = include_bytes!(#filename); }
    });

    let soundbank_data = ByteString(&mm_converted.soundbank_data);
    let soundbank_data = quote! {
        {
            #[repr(align(4))]
            struct AlignmentWrapper<const N: usize>([u8; N]);

            &AlignmentWrapper(*#soundbank_data).0
        }
    };

    let result = quote! {
        mod music {
            use ::agb::sound::tracker::*;

            pub struct Music;

            #[derive(Debug, Clone, Copy)]
            pub enum ModFiles {
                #(#mod_files)*
            }

            #[derive(Debug, Clone, Copy)]
            pub enum SfxFiles {
                #(#sfx_files)*
            }

            unsafe impl TrackerId for ModFiles {
                fn id(self) -> i32 { self as i32 }
            }

            unsafe impl TrackerId for SfxFiles {
                fn id(self) -> i32 { self as i32 }
            }

            const SOUND_BANK_DATA: &[u8] = #soundbank_data;

            unsafe impl TrackerOutput for Music {
                type ModId = ModFiles;
                type SfxId = SfxFiles;

                fn sound_bank() -> &'static [u8] {
                    SOUND_BANK_DATA
                }
            }

            #(#include_files)*
       }
    };

    TokenStream::from(result)
}
