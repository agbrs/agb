use std::{collections::HashMap, error::Error, fs, path::Path};

use agb_tracker_interop::PatternEffect;
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
        restart_point: u32,
    }

    let mut samples = vec![];

    for (instrument_index, instrument) in instruments.iter().enumerate() {
        let InstrumentType::Default(ref instrument) = instrument.instr_type else { continue; };

        for (sample_index, sample) in instrument.sample.iter().enumerate() {
            let should_loop = !matches!(sample.flags, LoopType::No);
            let fine_tune = sample.finetune as f64;
            let relative_note = sample.relative_note;
            let restart_point = sample.loop_start;
            let sample_len = if sample.loop_length > 0 {
                (sample.loop_length + sample.loop_start) as usize
            } else {
                usize::MAX
            };

            let sample = match &sample.data {
                SampleDataType::Depth8(depth8) => depth8
                    .iter()
                    .map(|value| *value as u8)
                    .take(sample_len)
                    .collect::<Vec<_>>(),
                SampleDataType::Depth16(depth16) => depth16
                    .iter()
                    .map(|sample| (sample >> 8) as i8 as u8)
                    .take(sample_len)
                    .collect::<Vec<_>>(),
            };

            instruments_map.insert((instrument_index, sample_index), samples.len());
            samples.push(SampleData {
                data: sample,
                should_loop,
                fine_tune,
                relative_note,
                restart_point,
            });
        }
    }

    let mut patterns = vec![];
    let mut pattern_data = vec![];

    for pattern in &module.pattern {
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

                let mut effect1 = match slot.volume {
                    0x10..=0x50 => {
                        PatternEffect::Volume(Num::new((slot.volume - 0x10) as i16) / 64)
                    }
                    0xC0..=0xCF => PatternEffect::Panning(
                        Num::new(slot.volume as i16 - (0xC0 + (0xCF - 0xC0) / 2)) / 64,
                    ),
                    _ => PatternEffect::None,
                };

                let effect2 = match slot.effect_type {
                    0x8 => {
                        PatternEffect::Panning(Num::new(slot.effect_parameter as i16 - 128) / 128)
                    }
                    0xC => PatternEffect::Volume(Num::new(slot.effect_parameter as i16) / 255),
                    _ => PatternEffect::None,
                };

                if matches!(slot.note, Note::KeyOff) {
                    effect1 = PatternEffect::Stop;
                }

                if sample == 0 {
                    pattern_data.push(agb_tracker_interop::PatternSlot {
                        speed: 0.into(),
                        sample: 0,
                        effect1,
                        effect2,
                    });
                } else {
                    let sample_played = &samples[sample - 1];

                    let speed = note_to_speed(
                        slot.note,
                        sample_played.fine_tune,
                        sample_played.relative_note,
                        module.frequency_type,
                    );

                    pattern_data.push(agb_tracker_interop::PatternSlot {
                        speed,
                        sample,
                        effect1,
                        effect2,
                    });
                }
            }
        }

        patterns.push(agb_tracker_interop::Pattern {
            length: pattern.len(),
            start_position: start_pos,
        });
    }

    let samples: Vec<_> = samples
        .iter()
        .map(|sample| agb_tracker_interop::Sample {
            data: &sample.data,
            should_loop: sample.should_loop,
            restart_point: sample.restart_point,
        })
        .collect();

    let patterns_to_play = module
        .pattern_order
        .iter()
        .map(|order| *order as usize)
        .collect::<Vec<_>>();

    // Number 150 here deduced experimentally
    let frames_per_tick = Num::<u16, 8>::new(150) / module.default_bpm;
    let ticks_per_step = module.default_tempo;

    let interop = agb_tracker_interop::Track {
        samples: &samples,
        pattern_data: &pattern_data,
        patterns: &patterns,
        num_channels: module.get_num_channels(),
        patterns_to_play: &patterns_to_play,

        frames_per_tick,
        ticks_per_step,
    };

    quote!(#interop)
}

fn note_to_speed(
    note: Note,
    fine_tune: f64,
    relative_note: i8,
    frequency_type: FrequencyType,
) -> Num<u32, 8> {
    let frequency = match frequency_type {
        FrequencyType::LinearFrequencies => {
            note_to_frequency_linear(note, fine_tune, relative_note)
        }
        FrequencyType::AmigaFrequencies => note_to_frequency_amega(note, fine_tune, relative_note),
    };

    let gba_audio_frequency = 18157f64;

    let speed: f64 = frequency / gba_audio_frequency;
    Num::from_raw((speed * (1 << 8) as f64) as u32)
}

fn note_to_frequency_linear(note: Note, fine_tune: f64, relative_note: i8) -> f64 {
    let real_note = (note as usize as f64) + (relative_note as f64);
    let period = 10.0 * 12.0 * 16.0 * 4.0 - (real_note as f64) * 16.0 * 4.0 - fine_tune / 2.0;
    8363.0 * 2.0f64.powf((6.0 * 12.0 * 16.0 * 4.0 - period) / (12.0 * 16.0 * 4.0))
}

fn note_to_frequency_amega(note: Note, fine_tune: f64, relative_note: i8) -> f64 {
    let note = (note as usize) + relative_note as usize;
    let pos = (note % 12) * 8 + (fine_tune / 16.0) as usize;
    let frac = (fine_tune / 16.0) - (fine_tune / 16.0).floor();

    let period = ((AMEGA_FREQUENCIES[pos] as f64 * (1.0 - frac))
        + AMEGA_FREQUENCIES[pos + 1] as f64 * frac)
        * 32.0 // docs say 16 here, but for some reason I need 32 :/
        / (1 << ((note as i64) / 12)) as f64;

    8363.0 * 1712.0 / period
}

const AMEGA_FREQUENCIES: &[u32] = &[
    907, 900, 894, 887, 881, 875, 868, 862, 856, 850, 844, 838, 832, 826, 820, 814, 808, 802, 796,
    791, 785, 779, 774, 768, 762, 757, 752, 746, 741, 736, 730, 725, 720, 715, 709, 704, 699, 694,
    689, 684, 678, 675, 670, 665, 660, 655, 651, 646, 640, 636, 632, 628, 623, 619, 614, 610, 604,
    601, 597, 592, 588, 584, 580, 575, 570, 567, 563, 559, 555, 551, 547, 543, 538, 535, 532, 528,
    524, 520, 516, 513, 508, 505, 502, 498, 494, 491, 487, 484, 480, 477, 474, 470, 467, 463, 460,
    457,
];
