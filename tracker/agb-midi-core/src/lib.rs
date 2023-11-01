use std::{
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use agb_fixnum::Num;
use agb_tracker_interop::{Pattern, PatternEffect, PatternSlot, Sample, Track};
use midly::{Format, MetaMessage, Smf, Timing, TrackEventKind};
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
    midi: Smf<'static>,
}

impl MidiInfo {
    pub fn load_from_file(sf2_file: &Path, midi_file: &Path) -> Result<Self, Box<dyn Error>> {
        let mut sound_font_file = BufReader::new(File::open(sf2_file)?);
        let sound_font = SoundFont::new(&mut sound_font_file)?;

        let midi_data = fs::read(midi_file)?;
        let smf = Smf::parse(&midi_data)?;

        Ok(Self {
            sound_font,
            midi: smf.make_static(),
        })
    }
}

pub fn parse_midi(midi_info: &MidiInfo) -> TokenStream {
    let midi = &midi_info.midi;

    assert_eq!(
        midi.header.format,
        Format::SingleTrack,
        "Only single track is currently supported"
    );
    let Timing::Metrical(timing) = midi.header.timing else { panic!("Only metrical timing is currently supported") };
    let ticks_per_beat = timing.as_int();

    let mut channel_data = vec![];
    let mut current_ticks = 0;
    let mut pattern = vec![];

    let mut initial_microseconds_per_beat = None;

    for event in &midi.tracks[0] {
        current_ticks += event.delta.as_int();

        match event.kind {
            TrackEventKind::Midi { channel, message } => {
                let channel_id = channel.as_int() as usize;
                channel_data.resize(
                    channel_data.len().max(channel_id + 1),
                    ChannelData::default(),
                );
                let channel_data = &mut channel_data[channel_id];

                match message {
                    midly::MidiMessage::NoteOff { .. } => pattern.push(PatternSlot {
                        speed: 0.into(),
                        sample: 0,
                        effect1: PatternEffect::Stop,
                        effect2: PatternEffect::None,
                    }),
                    midly::MidiMessage::NoteOn { key, vel } => pattern.push(PatternSlot {
                        speed: midi_key_to_speed(key.as_int() as i8),
                        sample: channel_data.current_sample,
                        effect1: PatternEffect::Volume(Num::from_f32(vel.as_int() as f32 / 128.0)),
                        effect2: PatternEffect::None,
                    }),
                    midly::MidiMessage::Aftertouch { .. } => {}
                    midly::MidiMessage::PitchBend { .. } => {}
                    midly::MidiMessage::ProgramChange { program } => {
                        channel_data.current_sample = program.as_int().into();
                    }
                    midly::MidiMessage::Controller { .. } => {}
                    midly::MidiMessage::ChannelAftertouch { .. } => {}
                }
            }
            TrackEventKind::Meta(MetaMessage::Tempo(tempo)) => {
                initial_microseconds_per_beat = Some(tempo.as_int());
            }
            _ => {}
        }
    }

    let mut samples = vec![];
    let sf2 = &midi_info.sound_font;
    let sf2_data = sf2.get_wave_data();

    struct SampleData {
        data: Vec<u8>,
        restart_point: Option<u32>,
    }

    for sample in sf2.get_sample_headers() {
        let sample_start = sample.get_start() as usize;
        let sample_end = sample.get_end() as usize;

        let sample_data = &sf2_data[sample_start..sample_end];

        let loop_start = sample.get_start_loop() as usize;
        let restart_point = if loop_start < sample_start {
            None
        } else {
            Some((loop_start - sample_start) as u32)
        };

        let data = sample_data
            .iter()
            .map(|data| (data >> 8) as i8 as u8)
            .collect::<Vec<_>>();

        let sample = SampleData {
            data,
            restart_point,
        };

        samples.push(sample);
    }

    let samples: Vec<_> = samples
        .iter()
        .map(|sample| Sample {
            data: &sample.data,
            should_loop: sample.restart_point.is_some(),
            restart_point: sample.restart_point.unwrap_or(0),
            volume: 256.into(),
            volume_envelope: None,
            fadeout: 0.into(),
        })
        .collect();

    let pattern = pattern.into_iter().collect::<Vec<_>>();

    let track = Track {
        samples: &samples,
        envelopes: &[],
        pattern_data: &pattern,
        patterns: &[Pattern {
            length: pattern.len() / channel_data.len(),
            start_position: 0,
        }],
        patterns_to_play: &[0],
        num_channels: channel_data.len(),
        frames_per_tick: Num::from_f64(
            initial_microseconds_per_beat.expect("No tempo was ever sent") as f64 / 16742.706298828, // microseconds per frame
        ),
        ticks_per_step: ticks_per_beat.into(),
        repeat: 0,
    };

    quote!(#track)
}

#[derive(Default, Clone)]
struct ChannelData {
    current_sample: u16,
}

fn midi_key_to_speed(key: i8) -> Num<u16, 8> {
    Num::from_f64(440.0 * 2f64.powf((key - 69) as f64 / 12.0))
}
