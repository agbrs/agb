use std::{collections::HashMap, error::Error, fs, path::Path};

use proc_macro2::TokenStream;
use proc_macro_error::abort;

use quote::quote;
use syn::LitStr;

use agb_fixnum::Num;

use xmrs::{prelude::*, xm::xmmodule::XmModule};

pub fn agb_xm_core(args: TokenStream) -> TokenStream {
    let input = match syn::parse::<LitStr>(args.into()) {
        Ok(input) => input,
        Err(err) => return proc_macro2::TokenStream::from(err.to_compile_error()),
    };

    let filename = input.value();

    let root = std::env::var("CARGO_MANIFEST_DIR").expect("Failed to get cargo manifest dir");
    let path = Path::new(&root).join(&*filename);

    let include_path = path.to_string_lossy();

    let module = match load_module_from_file(&path) {
        Ok(track) => track,
        Err(e) => abort!(input, e),
    };

    let parsed = parse_module(&module);

    quote! {
        {
            const _: &[u8] = include_bytes!(#include_path);

            #parsed
        }
    }
}

pub fn load_module_from_file(xm_path: &Path) -> Result<Module, Box<dyn Error>> {
    let file_content = fs::read(xm_path)?;
    Ok(XmModule::load(&file_content)?.to_module())
}

pub fn parse_module(module: &Module) -> TokenStream {
    let instruments = &module.instrument;
    let mut instruments_map = HashMap::new();

    let mut samples = vec![];

    for (instrument_index, instrument) in instruments.iter().enumerate() {
        let InstrumentType::Default(ref instrument) = instrument.instr_type else { continue; };

        for (sample_index, sample) in instrument.sample.iter().enumerate() {
            let sample = match &sample.data {
                SampleDataType::Depth8(depth8) => depth8
                    .iter()
                    .map(|value| *value as u8)
                    .collect::<Vec<_>>()
                    .clone(),
                SampleDataType::Depth16(depth16) => depth16
                    .iter()
                    .map(|sample| (sample >> 8) as i8 as u8)
                    .collect::<Vec<_>>(),
            };

            instruments_map.insert((instrument_index, sample_index), samples.len());
            samples.push(sample);
        }
    }

    let mut patterns = vec![];
    let mut pattern_data = vec![];

    for pattern in &module.pattern {
        let mut num_channels = 0;

        for row in pattern.iter() {
            for slot in row {
                let sample = if slot.instrument == 0 {
                    0
                } else {
                    let instrument_index = (slot.instrument - 1) as usize;

                    if let InstrumentType::Default(ref instrument) =
                        module.instrument[instrument_index].instr_type
                    {
                        let sample_slot = instrument.sample_for_note[slot.note as usize] as usize;
                        instruments_map
                            .get(&(instrument_index, sample_slot))
                            .cloned()
                            .unwrap_or(0)
                    } else {
                        0
                    }
                };

                let volume = Num::new(
                    if slot.volume == 0 {
                        64
                    } else {
                        slot.volume as i16
                    } / 64,
                );
                let speed = Num::new(1); // TODO: Calculate speed for the correct note here
                let panning = Num::new(0);

                pattern_data.push(agb_tracker_interop::PatternSlot {
                    volume,
                    speed,
                    panning,
                    sample,
                });
            }

            num_channels = row.len();
        }

        patterns.push(agb_tracker_interop::Pattern { num_channels });
    }

    let samples: Vec<_> = samples
        .iter()
        .map(|sample| agb_tracker_interop::Sample { data: &sample })
        .collect();

    let interop = agb_tracker_interop::Track {
        samples: &samples,
        pattern_data: &pattern_data,
        patterns: &patterns,
    };

    quote!(#interop)
}
