use std::{
    borrow::Cow,
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::BufReader,
    path::Path,
};

use agb_fixnum::Num;
use agb_tracker_interop::{Envelope, Pattern, PatternEffect, PatternSlot, Sample, Track};
use midly::{Format, MetaMessage, Smf, Timing, TrackEventKind};
use rustysynth::SoundFont;

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

pub fn parse_midi(midi_info: &MidiInfo) -> Track {
    let mut samples = vec![];
    let sf2 = &midi_info.sound_font;
    let sf2_data = sf2.get_wave_data();

    let mut preset_lookup = HashMap::new();

    for (i, preset) in sf2.get_presets().iter().enumerate() {
        preset_lookup.insert(
            preset.get_bank_number() << 16 | preset.get_patch_number(),
            i,
        );
    }

    let mut envelopes = vec![];

    for sample in sf2.get_sample_headers() {
        let sample_start = sample.get_start() as usize;
        let mut sample_end = sample.get_end() as usize;
        let sample_loop_end = sample.get_end_loop() as usize;

        if sample_loop_end > sample_start && sample_loop_end < sample_end {
            sample_end = sample_loop_end;
        }

        let sample_data = &sf2_data[sample_start..sample_end];

        let loop_start = sample.get_start_loop() as usize;
        let restart_point = if loop_start < sample_start {
            None
        } else {
            Some((loop_start - sample_start) as u32)
        };

        let note_offset = sample.get_original_pitch();

        let data = sample_data
            .iter()
            .map(|data| (data >> 8) as i8 as u8)
            .collect::<Vec<_>>();

        let instrument_region = sf2
            .get_instruments()
            .iter()
            .flat_map(|i| i.get_regions().iter())
            .find(|region| region.get_sample_id() == samples.len());

        let envelope = instrument_region.map(|region| {
            let delay = region.get_delay_volume_envelope();
            let attack = region.get_attack_volume_envelope();
            let hold = region.get_hold_volume_envelope();
            let decay = region.get_decay_volume_envelope();
            let sustain = region.get_sustain_volume_envelope() / 100.0;
            let release = region.get_release_volume_envelope();

            let envelope_data = EnvelopeData {
                delay,
                attack,
                hold,
                decay,
                sustain,
                release,
            };

            if let Some(index) = envelopes
                .iter()
                .position(|envelope| envelope == &envelope_data)
            {
                index
            } else {
                envelopes.push(envelope_data);
                envelopes.len() - 1
            }
        });

        let sample = SampleData {
            data,
            restart_point,
            note_offset,

            sample_rate: sample.get_sample_rate() as u32,
            envelope,
        };

        samples.push(sample);
    }

    let midi = &midi_info.midi;

    assert_eq!(
        midi.header.format,
        Format::SingleTrack,
        "Only single track is currently supported"
    );
    let Timing::Metrical(timing) = midi.header.timing else {
        panic!("Only metrical timing is currently supported")
    };
    let ticks_per_beat = timing.as_int();

    let mut channel_data = vec![];
    let mut current_ticks = 0;

    let mut initial_microseconds_per_beat = None;

    let mut patterns = vec![];

    for event in &midi.tracks[0] {
        current_ticks += event.delta.as_int();

        match event.kind {
            TrackEventKind::Midi { channel, message } => {
                let channel_id = channel.as_int() as usize;

                channel_data.resize(
                    channel_data.len().max(channel_id + 1),
                    ChannelData::default(),
                );
                patterns.resize_with(patterns.len().max(channel_id + 1), Vec::new);

                let channel_data = &mut channel_data[channel_id];
                let pattern = &mut patterns[channel_id];

                pattern.resize_with((current_ticks as usize).saturating_sub(1), Default::default);

                match message {
                    midly::MidiMessage::NoteOff { .. } => pattern.push(PatternSlot {
                        speed: 0.into(),
                        sample: 0,
                        effect1: PatternEffect::Stop,
                        effect2: PatternEffect::None,
                    }),
                    midly::MidiMessage::NoteOn { key, vel } => {
                        if vel == 0 {
                            pattern.push(PatternSlot {
                                speed: 0.into(),
                                sample: 0,
                                effect1: PatternEffect::Stop,
                                effect2: PatternEffect::None,
                            });
                            continue;
                        }

                        let Some(current_sample) = channel_data.current_sample else {
                            continue;
                        };

                        let preset = &sf2.get_presets()[current_sample];
                        let region = preset
                            .get_regions()
                            .iter()
                            .find(|region| {
                                region.contains(key.as_int() as i32, vel.as_int() as i32)
                            })
                            .expect("cannot find preset with correct region");
                        let instrument = &sf2.get_instruments()[region.get_instrument_id()];
                        let instrument_region = instrument
                            .get_regions()
                            .iter()
                            .find(|region| {
                                region.contains(key.as_int() as i32, vel.as_int() as i32)
                            })
                            .expect("cannot find instrument with correct region");
                        let sample_id = instrument_region.get_sample_id();

                        let coarse_tune = instrument_region.get_coarse_tune();
                        let fine_tune = instrument_region.get_fine_tune();

                        let sample = &samples[sample_id];

                        pattern.push(PatternSlot {
                            speed: midi_key_to_speed(
                                key.as_int() as i16,
                                sample,
                                channel_data.get_tune()
                                    + coarse_tune as f64
                                    + fine_tune as f64 / 8192.0,
                            ),
                            sample: sample_id as u16 + 1,
                            effect1: PatternEffect::Volume(Num::from_f32(
                                vel.as_int() as f32 / 128.0 * channel_data.volume,
                            )),
                            effect2: PatternEffect::Panning(Num::from_f32(channel_data.panning)),
                        });
                    }
                    midly::MidiMessage::Aftertouch { .. } => {}
                    midly::MidiMessage::PitchBend { bend } => {
                        // bend is between 0 and 16383 where 0 = -2 semitones and 16384 is +2 semitones (I think)
                        let amount = (bend.0.as_int() as f64 - (16384.0 / 2.0)) / (16384.0 / 2.0);

                        let amount = 2.0f64.powf((amount * 2.0) / 12.0);

                        pattern.push(PatternSlot {
                            speed: 0.into(),
                            sample: 0,
                            effect1: PatternEffect::PitchBend(Num::from_f64(amount)),
                            effect2: PatternEffect::None,
                        });
                    }
                    midly::MidiMessage::ProgramChange { program } => {
                        let mut lookup_id = program.as_int().into();
                        if channel_id == 9 {
                            lookup_id += 128 << 16;
                        }

                        channel_data.current_sample = preset_lookup.get(&lookup_id).copied();
                    }
                    midly::MidiMessage::Controller { controller, value } => {
                        match controller.as_int() {
                            0 => assert_eq!(value.as_int(), 0, "no support for changing bank yet"),
                            6 => channel_data.data_entry_coarse(value.as_int() as i32),
                            7 => channel_data.volume = value.as_int() as f32 / 128.0,
                            10 => channel_data.panning = value.as_int() as f32 / 64.0 - 1.0,
                            26 => channel_data.data_entry_fine(value.as_int() as i32),
                            100 => channel_data.set_rpn(value.as_int() as i32),
                            _ => {}
                        }
                    }
                    midly::MidiMessage::ChannelAftertouch { .. } => {}
                }
            }
            TrackEventKind::Meta(MetaMessage::Tempo(tempo)) => {
                initial_microseconds_per_beat = Some(tempo.as_int());
            }
            _ => {}
        }
    }

    patterns.retain(|pattern| {
        !pattern.iter().all(|pattern_slot| {
            matches!(pattern_slot.effect1, PatternEffect::None)
                && matches!(pattern_slot.effect2, PatternEffect::None)
        })
    });

    for pattern in &mut patterns {
        pattern.resize_with(current_ticks as usize, Default::default);
    }

    let frames_per_tick = initial_microseconds_per_beat.expect("No tempo was ever sent") as f64
        / 16742.706298828 // microseconds per frame
        / ticks_per_beat as f64;

    struct ParsedEnvelopeData {
        amounts: Vec<Num<i16, 8>>,
        decay: f32,
    }

    let envelopes: Vec<_> = envelopes
        .iter()
        .map(|envelope| {
            let mut amounts = vec![];

            let ticks_per_second = (60.0 / frames_per_tick) as f32;

            let delay_ticks = (envelope.delay * ticks_per_second) as usize;
            let attack_ticks = (envelope.attack * ticks_per_second) as usize;
            let hold_ticks = (envelope.hold * ticks_per_second) as usize;
            let decay_ticks = (envelope.decay * ticks_per_second) as usize;
            let release_ticks = envelope.release * ticks_per_second;

            // volume envelope looks like the following:
            //          /--------\
            //         /          \______
            //        /                  \
            //       /                    \
            // _____/                      \
            // delay      hold    sustain*
            //      attack      decay    release**
            //
            // *  The sustain is actually a single point with the sustain set in the envelope data
            // ** release is stored separately alongside the sample's 'fadeout'

            amounts.resize(delay_ticks, Num::<i16, 8>::default());
            for i in 0..attack_ticks {
                amounts.push(Num::from_f32(i as f32 / attack_ticks as f32));
            }

            amounts.resize(amounts.len() + hold_ticks, 1.into());
            for i in 0..decay_ticks {
                amounts.push(Num::from_f32(
                    (decay_ticks - i) as f32 / decay_ticks as f32 * (1.0 - envelope.sustain)
                        + envelope.sustain,
                ));
            }

            if amounts.is_empty() {
                amounts.push(1.into());
            }

            ParsedEnvelopeData {
                amounts,
                decay: (1.0 / release_ticks).min(0.5),
            }
        })
        .collect();

    let samples: Vec<_> = samples
        .iter()
        .map(|sample| Sample {
            data: sample.data.clone().into(),
            should_loop: sample.restart_point.is_some(),
            restart_point: sample.restart_point.unwrap_or(0),
            volume: 256.into(),
            volume_envelope: sample.envelope,
            fadeout: sample
                .envelope
                .map(|e| Num::from_f32(envelopes[e].decay))
                .unwrap_or(0.into()),
        })
        .collect();

    let resulting_num_channels = patterns.len();
    let mut pattern = Vec::with_capacity(current_ticks as usize * resulting_num_channels);
    for i in 0..current_ticks {
        for pattern_slots in &patterns {
            pattern.push(pattern_slots[i as usize].clone());
        }
    }

    let envelopes: Vec<_> = envelopes
        .iter()
        .map(|envelope| Envelope {
            amount: envelope.amounts.clone().into(),
            sustain: Some(envelope.amounts.len() - 1),
            loop_start: None,
            loop_end: None,

            vib_waveform: Default::default(),
            vib_amount: Default::default(),
            vib_speed: Default::default(),
        })
        .collect();

    Track {
        samples: samples.into(),
        envelopes: envelopes.into(),
        patterns: Cow::from(vec![Pattern {
            length: pattern.len() / resulting_num_channels,
            start_position: 0,
        }]),
        pattern_data: pattern.into(),
        patterns_to_play: Cow::from(vec![0]),
        num_channels: resulting_num_channels,
        frames_per_tick: Num::from_f64(frames_per_tick),
        ticks_per_step: 1,
        repeat: 0,
    }
}

#[derive(Clone, Default)]
struct ChannelData {
    current_sample: Option<usize>,
    volume: f32,
    panning: f32,
    rpn: i32,
    fine_tune: i16,
    course_tune: i16,
}

impl ChannelData {
    fn set_rpn(&mut self, value: i32) {
        self.rpn = value;
    }

    fn data_entry_fine(&mut self, value: i32) {
        if self.rpn == 1 {
            self.fine_tune = (((self.fine_tune as i32) & 0xFF80) | value) as i16;
        }
    }

    fn data_entry_coarse(&mut self, value: i32) {
        if self.rpn == 1 {
            self.fine_tune = (self.fine_tune & 0x7F) | (value << 7) as i16;
        } else if self.rpn == 2 {
            self.course_tune = (value - 64) as i16;
        }
    }

    fn get_tune(&self) -> f64 {
        self.course_tune as f64 + (1.0 / 8192f64) * (self.fine_tune - 8192) as f64
    }
}

#[derive(Debug)]
struct SampleData {
    data: Vec<u8>,
    restart_point: Option<u32>,
    sample_rate: u32,
    note_offset: i32,
    envelope: Option<usize>,
}

fn midi_key_to_speed(key: i16, sample: &SampleData, tune: f64) -> Num<u16, 8> {
    let sample_rate = sample.sample_rate as f64;
    let relative_note = sample.note_offset as f64;

    Num::from_f64(
        2f64.powf((key as f64 - relative_note + tune + 1.0) / 12.0) * sample_rate / 32768.0,
    )
}

#[derive(Clone, PartialEq)]
struct EnvelopeData {
    delay: f32,
    attack: f32,
    hold: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}
