#![deny(clippy::all)]

use proc_macro::TokenStream;
use quote::quote;
use std::{
    collections::hash_map::DefaultHasher,
    fs,
    fs::File,
    hash::{Hash, Hasher},
    io::Write,
    path::Path,
};
use syn::parse_macro_input;

#[cfg(all(not(feature = "freq18157"), not(feature = "freq32768")))]
const FREQUENCY: u32 = 10512;
#[cfg(feature = "freq18157")]
const FREQUENCY: u32 = 18157;
#[cfg(feature = "freq32768")]
const FREQUENCY: u32 = 32768;
#[cfg(all(feature = "freq18157", feature = "freq32768"))]
compile_error!("Must have at most one of freq18157 or freq32768 features enabled");

#[proc_macro]
pub fn include_wav(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::LitStr);

    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let include_path = path.to_string_lossy();

    let out_file_path_include = {
        let out_dir = std::env::var("OUT_DIR").expect("Expected OUT_DIR");
        let out_filename = get_out_filename(&path);

        let out_file_path = Path::new(&out_dir).with_file_name(&out_filename);

        let out_file_mtime = fs::metadata(&out_file_path).and_then(|metadata| metadata.modified());
        let in_file_mtime = fs::metadata(&path).and_then(|metadata| metadata.modified());

        let should_write = match (out_file_mtime, in_file_mtime) {
            (Ok(out_file_mtime), Ok(in_file_mtime)) => out_file_mtime <= in_file_mtime,
            _ => true,
        };

        if should_write {
            let wav_reader = hound::WavReader::open(&path)
                .unwrap_or_else(|_| panic!("Failed to load file {}", include_path));

            assert_eq!(
                wav_reader.spec().sample_rate,
                FREQUENCY,
                "agb currently only supports sample rate of {}Hz",
                FREQUENCY
            );

            let samples = samples_from_reader(wav_reader);

            let mut out_file =
                File::create(&out_file_path).expect("Failed to open file for writing");

            out_file
                .write_all(&samples.collect::<Vec<_>>())
                .expect("Failed to write to temporary file");
        }

        out_file_path
    }
    .canonicalize()
    .expect("Failed to canonicalize");

    let out_file_path_include = out_file_path_include.to_string_lossy();

    let result = quote! {
        {
            #[repr(align(4))]
            struct AlignmentWrapper<const N: usize>([u8; N]);

            const _: &[u8] = include_bytes!(#include_path);

            &AlignmentWrapper(*include_bytes!(#out_file_path_include)).0
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

fn get_out_filename(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);

    format!("{}.raw", hasher.finish())
}
