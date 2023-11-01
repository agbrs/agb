use std::{error::Error, fs::File, io::BufReader, path::Path};

use agb_tracker_interop::Track;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::quote;
use rustysynth::SoundFont;
use syn::{
    parse::{Parse, ParseStream},
    LitStr, Token,
};

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

pub fn agb_midi_core(args: TokenStream) -> TokenStream {
    let input: MidiCoreInput = match syn::parse2(args.clone()) {
        Ok(input) => input,
        Err(e) => abort!(args, e),
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
        Err(e) => abort!(args, e),
    };

    let parsed = parse_midi(&midi_info);

    quote! {
        {
            const _: &[u8] = include_bytes!(#sf2_include_path);
            const _: &[u8] = include_bytes!(#midi_file_include_path);

            #parsed
        }
    }
}

pub struct MidiInfo {
    sound_font: SoundFont,
}

impl MidiInfo {
    pub fn load_from_file(sf2_file: &Path, midi_file: &Path) -> Result<Self, Box<dyn Error>> {
        let mut sound_font_file = BufReader::new(File::open(sf2_file)?);
        let sound_font = SoundFont::new(&mut sound_font_file)?;

        Ok(Self { sound_font })
    }
}

pub fn parse_midi(midi_info: &MidiInfo) -> TokenStream {
    let track = Track {
        samples: &[],
        envelopes: &[],
        pattern_data: &[],
        patterns: &[],
        patterns_to_play: &[],
        num_channels: 0,
        frames_per_tick: 2.into(),
        ticks_per_step: 2,
        repeat: 0,
    };

    quote!(#track)
}
