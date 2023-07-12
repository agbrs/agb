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

    struct SampleData {
        data: Vec<u8>,
        should_loop: bool,
        fine_tune: f64,
        relative_note: i8,
    }

    let mut samples = vec![];

    for (instrument_index, instrument) in instruments.iter().enumerate() {
        let InstrumentType::Default(ref instrument) = instrument.instr_type else { continue; };

        for (sample_index, sample) in instrument.sample.iter().enumerate() {
            let should_loop = !matches!(sample.flags, LoopType::No);
            let fine_tune = sample.finetune as f64;
            let relative_note = sample.relative_note;

            let mut sample = match &sample.data {
                SampleDataType::Depth8(depth8) => {
                    depth8.iter().map(|value| *value as u8).collect::<Vec<_>>()
                }
                SampleDataType::Depth16(depth16) => depth16
                    .iter()
                    .map(|sample| (sample >> 8) as i8 as u8)
                    .collect::<Vec<_>>(),
            };

            if should_loop {
                sample.append(&mut sample.clone());
                sample.append(&mut sample.clone());
                sample.append(&mut sample.clone());
                sample.append(&mut sample.clone());
                sample.append(&mut sample.clone());
                sample.append(&mut sample.clone());
                sample.append(&mut sample.clone());
            }

            instruments_map.insert((instrument_index, sample_index), samples.len());
            samples.push(SampleData {
                data: sample,
                should_loop,
                fine_tune,
                relative_note,
            });
        }
    }

    let mut patterns = vec![];
    let mut pattern_data = vec![];

    for pattern in &module.pattern {
        let mut num_channels = 0;
        let start_pos = pattern_data.len();

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
                            .map(|sample_idx| sample_idx + 1)
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

                if sample == 0 {
                    // TODO should take into account previous sample played on this channel
                    pattern_data.push(agb_tracker_interop::PatternSlot {
                        volume: Num::new(0),
                        speed: Num::new(0),
                        panning: Num::new(0),
                        sample: 0,
                    })
                } else {
                    let sample_played = &samples[sample - 1];

                    let speed = note_to_speed(
                        slot.note,
                        sample_played.fine_tune,
                        sample_played.relative_note,
                    );
                    let panning = Num::new(0);

                    pattern_data.push(agb_tracker_interop::PatternSlot {
                        volume,
                        speed,
                        panning,
                        sample,
                    });
                }
            }

            num_channels = row.len();
        }

        patterns.push(agb_tracker_interop::Pattern {
            num_channels,
            length: pattern.len(),
            start_position: start_pos,
        });
    }

    let samples: Vec<_> = samples
        .iter()
        .map(|sample| agb_tracker_interop::Sample {
            data: &sample.data,
            should_loop: sample.should_loop,
        })
        .collect();

    let frames_per_step =
        ((60.0 * 60.0) / module.default_bpm as f64 / module.default_tempo as f64) as u16;

    let interop = agb_tracker_interop::Track {
        samples: &samples,
        pattern_data: &pattern_data,
        patterns: &patterns,

        frames_per_step,
    };

    quote!(#interop)
}

fn note_to_frequency(note: Note, fine_tune: f64, relative_note: i8) -> f64 {
    let real_note = (note as usize as f64) + (relative_note as f64);
    let period = 10.0 * 12.0 * 16.0 * 4.0 - (real_note as f64) * 16.0 * 4.0 - fine_tune / 2.0;
    8363.0 * 2.0f64.powf((6.0 * 12.0 * 16.0 * 4.0 - period) / (12.0 * 16.0 * 4.0))
}

fn note_to_speed(note: Note, fine_tune: f64, relative_note: i8) -> Num<u32, 8> {
    let frequency = note_to_frequency(note, fine_tune, relative_note);

    let gba_audio_frequency = 18157f64;

    let speed: f64 = frequency / gba_audio_frequency;
    Num::from_raw((speed * (1 << 8) as f64) as u32)
}
