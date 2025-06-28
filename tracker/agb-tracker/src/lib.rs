#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![deny(missing_docs)]

//! # agb_tracker
//! `agb_tracker` is a library for playing tracker music on the Game Boy Advance (GBA)
//! using the [`agb`](https://github.com/agbrs/agb) library.
//!
//! The default mechanism for playing background music using `agb` is to include a
//! the entire music as a raw sound file. However, this can get very large (>8MB) for
//! only a few minutes of music, taking up most of your limited ROM space.
//!
//! Using a tracker, you can store many minutes of music in only a few kB of ROM which makes
//! the format much more space efficient at the cost of some CPU.
//!
//! This library uses about 20-30% of the GBA's CPU time per frame, for 4 channels but most of that is
//! `agb`'s mixing. The main [`step`](Tracker::step()) function uses around 2000 cycles (<1%).
//!
//! # Example
//!
//! ```rust,no_run
//! #![no_std]
//! #![no_main]
//!
//! extern crate alloc;
//!
//! use agb::{Gba, sound::mixer::Frequency};
//! use agb_tracker::{include_xm, Track, Tracker};
//!
//! static BACKGROUND_MUSIC: Track = include_xm!("examples/tracks/peak_and_drozerix_-_spectrum.xm");
//!
//! #[agb::entry]
//! fn main(mut gba: Gba) -> ! {
//!     let vblank_provider = agb::interrupt::VBlank::get();
//!
//!     let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
//!     let mut tracker = Tracker::new(&BACKGROUND_MUSIC);
//!
//!     loop {
//!         tracker.step(&mut mixer);
//!         mixer.frame();
//!
//!         vblank_provider.wait_for_vblank();
//!     }
//! }
//! ```
//!
//! Note that currently you have to select 32768Hz as the frequency for the mixer.
//! This restriction will be lifted in a future version.
//!
//! # Concepts
//!
//! The main concept of the `agb_tracker` crate is to move as much of the work to build
//! time as possible to make the actual playing as fast as we can. The passed tracker file
//! gets parsed and converted into a simplified format which is then played while the game
//! is running.
//!
//! In theory, the format the tracker file gets converted into is agnostic to the base format.
//! Currently, only XM is implemented, however, more formats could be added in future depending
//! on demand.

extern crate alloc;

mod lookups;
mod mixer;

use agb_tracker_interop::{Jump, PatternEffect, Sample, Waveform};
use alloc::vec::Vec;

pub use mixer::{Mixer, SoundChannel};

use agb_fixnum::Num;

/// Import an XM file. Only available if you have the `xm` feature enabled (enabled by default).
#[cfg(feature = "xm")]
pub use agb_xm::include_xm;

/// Import an S3M file. Only available if you have the `xm` feature enabled (enabled by default).
#[cfg(feature = "xm")]
pub use agb_xm::include_s3m;

/// Import a MOD file. Only available if you have the `xm` feature enabled (enabled by default).
#[cfg(feature = "xm")]
pub use agb_xm::include_mod;

/// Import a midi file. Only available if you have the `midi` feature enabled (enabled by default).
/// This is currently experimental, and many types of MIDI file or MIDI features are not supported.
///
/// Takes 2 arguments, an SF2 file and a midi file.
#[cfg(feature = "midi")]
pub use agb_midi::include_midi;

#[doc(hidden)]
pub mod __private {
    pub use agb_fixnum::Num;
    pub use agb_tracker_interop;
}

/// A reference to a track. You should create this using one of the include macros.
pub use agb_tracker_interop::Track;

/// Stores the required state in order to play tracker music.
pub struct TrackerInner<'track, TChannelId> {
    track: &'track Track,
    channels: Vec<TrackerChannel>,
    envelopes: Vec<Option<EnvelopeState>>,

    mixer_channels: Vec<Option<TChannelId>>,

    frame: Num<u32, 8>,
    tick: u32,
    first: bool,

    global_settings: GlobalSettings,

    current_row: usize,
    current_pattern: usize,
    current_jump: Option<Jump>,
}

#[derive(Default)]
struct TrackerChannel {
    original_speed: Num<u32, 16>,
    base_speed: Num<u32, 16>,
    volume: Num<i32, 8>,

    vibrato: Waves,

    current_volume: Num<i32, 8>,
    current_speed: Num<u32, 16>,
    current_panning: Num<i32, 8>,
    is_playing: bool,

    // if some, should set the current position to this
    current_pos: Option<u16>,
}

#[derive(Default)]
struct Waves {
    waveform: Waveform,
    frame: usize,
    speed: usize,
    amount: Num<i32, 12>,

    enable: bool,
}

impl Waves {
    fn value(&self) -> Num<u32, 8> {
        assert!(self.amount.abs() <= 1.into());

        calculate_wave(self.waveform, self.amount, self.frame)
    }
}

fn calculate_wave(waveform: Waveform, amount: Num<i32, 12>, frame: usize) -> Num<u32, 8> {
    let lookup = match waveform {
        Waveform::Sine => lookups::SINE_LOOKUP,
        Waveform::Saw => lookups::SAW_LOOKUP,
        Waveform::Square => lookups::SQUARE_LOOKUP,
    };

    (amount * lookup[frame] + 1).try_change_base().unwrap()
}

struct EnvelopeState {
    frame: usize,
    envelope_id: usize,
    finished: bool,
    fadeout: Num<i32, 8>,

    vibrato_pos: usize,
}

#[derive(Clone)]
struct GlobalSettings {
    ticks_per_step: u32,

    frames_per_tick: Num<u32, 8>,
    volume: Num<i32, 8>,
}

impl<'track, TChannelId> TrackerInner<'track, TChannelId> {
    /// Create a new tracker playing a specified track. See the [example](crate#example) for how to use the tracker.
    pub fn new(track: &'track Track) -> Self {
        let mut channels = Vec::new();
        channels.resize_with(track.num_channels, Default::default);

        let mut envelopes = Vec::new();
        envelopes.resize_with(track.num_channels, || None);

        let mut mixer_channels = Vec::new();
        mixer_channels.resize_with(track.num_channels, || None);

        let global_settings = GlobalSettings {
            ticks_per_step: track.ticks_per_step,
            frames_per_tick: track.frames_per_tick,
            volume: 1.into(),
        };

        Self {
            track,
            mixer_channels,
            channels,
            envelopes,

            frame: 0.into(),
            first: true,
            tick: 0,

            global_settings,

            current_pattern: 0,
            current_row: 0,
            current_jump: None,
        }
    }

    /// Call this once per frame before calling [`mixer.frame`](agb::sound::mixer::Mixer::frame()).
    /// See the [example](crate#example) for how to use the tracker.
    pub fn step<M: Mixer<ChannelId = TChannelId>>(&mut self, mixer: &mut M) {
        if !self.increment_frame() {
            self.update_envelopes();

            self.realise(mixer);
            return;
        }

        let pattern_to_play = self.track.patterns_to_play[self.current_pattern];
        let current_pattern = &self.track.patterns[pattern_to_play];

        let pattern_data_pos =
            current_pattern.start_position + self.current_row * self.track.num_channels;
        let pattern_slots =
            &self.track.pattern_data[pattern_data_pos..pattern_data_pos + self.track.num_channels];

        for (i, (channel, pattern_slot)) in self.channels.iter_mut().zip(pattern_slots).enumerate()
        {
            if pattern_slot.sample != 0 && self.tick == 0 {
                let sample = &self.track.samples[pattern_slot.sample as usize - 1];

                if let Some(channel) = self.mixer_channels[i]
                    .take()
                    .and_then(|channel_id| mixer.channel(&channel_id))
                {
                    channel.stop();
                }

                let mut new_channel = M::SoundChannel::new(&sample.data);
                if sample.should_loop {
                    new_channel
                        .should_loop()
                        .restart_point(sample.restart_point);
                }

                self.mixer_channels[i] = mixer.play_sound(new_channel);

                channel.reset(sample);

                self.envelopes[i] = sample.volume_envelope.map(|envelope_id| EnvelopeState {
                    frame: 0,
                    envelope_id,
                    finished: false,
                    fadeout: sample.fadeout,

                    vibrato_pos: 0,
                });
            }

            if self.tick == 0 {
                channel.set_speed(pattern_slot.speed.change_base());
            }

            channel.vibrato.enable = false;

            channel.apply_effect(
                &pattern_slot.effect1,
                self.tick,
                &mut self.global_settings,
                &mut self.envelopes[i],
                &mut self.current_jump,
            );
            channel.apply_effect(
                &pattern_slot.effect2,
                self.tick,
                &mut self.global_settings,
                &mut self.envelopes[i],
                &mut self.current_jump,
            );
        }

        self.update_envelopes();
        self.realise(mixer);
    }

    /// Stops all channels.
    ///
    /// It is expected that you don't call step after this. But doing so will continue from
    /// where you left off. However, notes which were playing won't resume.
    pub fn stop<M: Mixer<ChannelId = TChannelId>>(&mut self, mixer: &mut M) {
        for channel_id in &mut self.mixer_channels {
            if let Some(channel) = channel_id
                .take()
                .and_then(|channel_id| mixer.channel(&channel_id))
            {
                channel.stop();
            }
        }
    }

    fn realise<M: Mixer<ChannelId = TChannelId>>(&mut self, mixer: &mut M) {
        for (i, (mixer_channel, tracker_channel)) in self
            .mixer_channels
            .iter()
            .zip(&mut self.channels)
            .enumerate()
        {
            tracker_channel.tick();

            if let Some(channel) = mixer_channel
                .as_ref()
                .and_then(|channel_id| mixer.channel(channel_id))
            {
                let mut current_speed = tracker_channel.current_speed;

                if tracker_channel.vibrato.speed != 0 && tracker_channel.vibrato.enable {
                    current_speed *= tracker_channel.vibrato.value().change_base();
                } else if let Some(envelope) = &mut self.envelopes[i] {
                    let track_envelope = &self.track.envelopes[envelope.envelope_id];

                    if track_envelope.vib_speed != 0 {
                        current_speed *= calculate_wave(
                            track_envelope.vib_waveform,
                            track_envelope.vib_amount.change_base(),
                            envelope.vibrato_pos,
                        )
                        .change_base();
                        envelope.vibrato_pos =
                            (envelope.vibrato_pos + track_envelope.vib_speed as usize) % 64;
                    }
                }

                channel.playback(current_speed.change_base());
                channel.volume(tracker_channel.current_volume.try_change_base().unwrap());
                channel.panning(tracker_channel.current_panning.try_change_base().unwrap());

                if let Some(offset) = tracker_channel.current_pos.take() {
                    channel.set_pos(offset as u32);
                }

                if tracker_channel.is_playing {
                    channel.resume();
                } else {
                    channel.pause();
                }
            }
        }
    }

    fn update_envelopes(&mut self) {
        for (channel, envelope_state_option) in self.channels.iter_mut().zip(&mut self.envelopes) {
            if let Some(envelope_state) = envelope_state_option {
                let envelope = &self.track.envelopes[envelope_state.envelope_id];

                if !channel.update_volume_envelope(envelope_state, envelope, &self.global_settings)
                {
                    envelope_state_option.take();
                } else {
                    envelope_state.frame += 1;

                    if !envelope_state.finished
                        && let Some(sustain) = envelope.sustain
                        && envelope_state.frame >= sustain
                    {
                        envelope_state.frame = sustain;
                    }

                    if let Some(loop_end) = envelope.loop_end
                        && envelope_state.frame >= loop_end
                    {
                        envelope_state.frame = envelope.loop_start.unwrap_or(0);
                    }

                    if envelope_state.frame >= envelope.amount.len() {
                        envelope_state.frame = envelope.amount.len() - 1;
                    }
                }
            }
        }
    }

    fn increment_frame(&mut self) -> bool {
        if self.first {
            self.first = false;
            return true;
        }

        self.frame += 1;

        if self.frame >= self.global_settings.frames_per_tick {
            self.tick += 1;
            self.frame -= self.global_settings.frames_per_tick;

            if self.tick >= self.global_settings.ticks_per_step {
                if let Some(jump) = self.current_jump.take() {
                    self.handle_jump(jump);
                } else {
                    self.current_row += 1;

                    if self.current_row
                        >= self.track.patterns[self.track.patterns_to_play[self.current_pattern]]
                            .length
                    {
                        self.current_pattern += 1;
                        self.current_row = 0;

                        if self.current_pattern >= self.track.patterns_to_play.len() {
                            self.current_pattern = self.track.repeat;
                        }
                    }
                }

                self.tick = 0;
            }

            true
        } else {
            false
        }
    }

    fn handle_jump(&mut self, jump: Jump) {
        match jump {
            Jump::Position { pattern } => {
                self.current_pattern = pattern as usize;
                self.current_row = 0;
            }
            Jump::PatternBreak { row } => {
                self.current_pattern += 1;
                self.current_row = row as usize;
            }
            Jump::Combined { pattern, row } => {
                self.current_pattern = pattern as usize;
                self.current_row = row as usize;
            }
        };
        if self.current_pattern >= self.track.patterns_to_play.len() {
            self.current_pattern = self.track.repeat;
        }
        if self.current_row
            >= self.track.patterns[self.track.patterns_to_play[self.current_pattern]].length
        {
            // TODO: reconsider this default
            self.current_row = 0;
        }
    }
}

impl TrackerChannel {
    fn reset(&mut self, sample: &Sample) {
        self.volume = sample.volume.change_base();
        self.current_volume = self.volume;
        self.current_panning = 0.into();
        self.is_playing = true;
    }

    fn set_speed(&mut self, speed: Num<u32, 8>) {
        if speed != 0.into() {
            self.base_speed = speed.change_base();
            self.original_speed = self.base_speed;
        }

        self.current_speed = self.base_speed;
    }

    fn apply_effect(
        &mut self,
        effect: &PatternEffect,
        tick: u32,
        global_settings: &mut GlobalSettings,
        envelope_state: &mut Option<EnvelopeState>,
        current_jump: &mut Option<Jump>,
    ) {
        match effect {
            PatternEffect::None => {}
            PatternEffect::Stop => {
                self.current_volume = 0.into();

                if let Some(envelope_state) = envelope_state {
                    envelope_state.finished = true;
                }
            }
            PatternEffect::Arpeggio(first, second) => {
                match tick % 3 {
                    0 => self.current_speed = self.base_speed.change_base(),
                    1 => self.current_speed = first.change_base(),
                    2 => self.current_speed = second.change_base(),
                    _ => unreachable!(),
                };
            }
            PatternEffect::Panning(panning) => {
                self.current_panning = panning.change_base();
            }
            PatternEffect::Volume(volume) => {
                self.current_volume = (volume.change_base() * global_settings.volume)
                    .try_change_base()
                    .unwrap();

                self.volume = volume.change_base();
            }
            PatternEffect::VolumeSlide(amount, keep_vibrato) => {
                if tick != 0 {
                    self.volume = (self.volume + amount.change_base()).max(0.into());
                    self.current_volume = (self.volume * global_settings.volume)
                        .try_change_base()
                        .unwrap();
                }

                self.vibrato.enable = *keep_vibrato;
            }
            PatternEffect::FineVolumeSlide(amount) => {
                if tick == 0 {
                    self.volume = (self.volume + amount.change_base()).max(0.into());
                    self.current_volume = (self.volume * global_settings.volume)
                        .try_change_base()
                        .unwrap();
                }
            }
            PatternEffect::NoteCut(wait) => {
                if tick == *wait {
                    self.current_volume = 0.into();

                    if let Some(envelope_state) = envelope_state {
                        envelope_state.finished = true;
                    }
                }
            }
            PatternEffect::NoteDelay(wait) => {
                if tick < *wait {
                    self.is_playing = false;
                }

                if tick == *wait {
                    self.is_playing = true;
                    self.current_volume = (self.volume * global_settings.volume)
                        .try_change_base()
                        .unwrap();
                }
            }
            PatternEffect::Portamento(amount) => {
                if tick != 0 {
                    self.base_speed *= amount.change_base();
                    self.current_speed = self.base_speed.change_base();
                }
            }
            PatternEffect::FinePortamento(amount) => {
                if tick == 1 {
                    self.base_speed *= amount.change_base();
                    self.current_speed = self.base_speed.change_base();
                }
            }
            PatternEffect::TonePortamento(amount, target) => {
                self.current_volume = (self.volume * global_settings.volume)
                    .try_change_base()
                    .unwrap();

                if tick != 0 {
                    if *amount < 1.into() {
                        self.base_speed =
                            (self.base_speed * amount.change_base()).max(target.change_base());
                    } else {
                        self.base_speed =
                            (self.base_speed * amount.change_base()).min(target.change_base());
                    }
                }

                self.current_speed = self.base_speed.change_base();
            }
            PatternEffect::PitchBend(amount) => {
                if tick == 0 {
                    self.base_speed = self.original_speed * amount.change_base();
                    self.current_speed = self.base_speed.change_base();
                }
            }

            PatternEffect::SetTicksPerStep(amount) => {
                global_settings.ticks_per_step = *amount;
            }
            PatternEffect::SetFramesPerTick(new_frames_per_tick) => {
                global_settings.frames_per_tick = *new_frames_per_tick;
            }
            PatternEffect::SetGlobalVolume(volume) => {
                global_settings.volume = *volume;
            }
            PatternEffect::GlobalVolumeSlide(volume_delta) => {
                global_settings.volume =
                    (global_settings.volume + *volume_delta).clamp(0.into(), 1.into());
            }
            PatternEffect::Vibrato(waveform, amount, speed) => {
                if *amount != 0.into() {
                    self.vibrato.amount = amount.change_base();
                }

                if *speed != 0 {
                    self.vibrato.speed = *speed as usize;
                }

                self.vibrato.waveform = *waveform;
                self.vibrato.enable = true;
            }
            PatternEffect::Jump(jump) => {
                *current_jump = Some(jump.clone());
            }
            PatternEffect::SampleOffset(offset) => {
                if tick == 0 {
                    self.current_pos = Some(*offset);
                }
            }
            PatternEffect::Retrigger(volume_change, ticks) => {
                if tick.is_multiple_of(*ticks as u32) {
                    match volume_change {
                        agb_tracker_interop::RetriggerVolumeChange::DecreaseByOne => {
                            self.volume = (self.volume - Num::new(1) / 64).max(0.into());
                            self.current_volume = (self.volume * global_settings.volume)
                                .try_change_base()
                                .unwrap();
                        }
                        agb_tracker_interop::RetriggerVolumeChange::NoChange => {}
                    }

                    self.current_pos = Some(0);
                }
            }
        }
    }

    #[must_use]
    fn update_volume_envelope(
        &mut self,
        envelope_state: &EnvelopeState,
        envelope: &agb_tracker_interop::Envelope,
        global_settings: &GlobalSettings,
    ) -> bool {
        let amount = envelope.amount[envelope_state.frame];

        if envelope_state.finished {
            self.volume = (self.volume - envelope_state.fadeout).max(0.into());
        }

        self.current_volume = (self.volume * amount.change_base() * global_settings.volume)
            .try_change_base()
            .unwrap();

        self.volume != 0.into()
    }

    fn tick(&mut self) {
        self.vibrato.frame = (self.vibrato.frame + self.vibrato.speed) % 64;
    }
}

#[cfg(all(test, feature = "agb"))]
#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    loop {
        agb::halt();
    }
}

#[cfg(feature = "agb")]
impl SoundChannel for agb::sound::mixer::SoundChannel {
    fn new(data: &alloc::borrow::Cow<'static, [u8]>) -> Self {
        Self::new(match data {
            alloc::borrow::Cow::Borrowed(data) =>
            // Safety: should be good by construction, but it'll blow up if you try and play it and it isn't aligned
            unsafe { agb::sound::mixer::SoundData::new(data) },
            alloc::borrow::Cow::Owned(_) => {
                unimplemented!("Must use borrowed COW data for tracker")
            }
        })
    }

    fn stop(&mut self) {
        self.stop();
    }

    fn pause(&mut self) -> &mut Self {
        self.pause()
    }

    fn resume(&mut self) -> &mut Self {
        self.resume()
    }

    fn should_loop(&mut self) -> &mut Self {
        self.should_loop()
    }

    fn volume(&mut self, value: impl Into<Num<i16, 8>>) -> &mut Self {
        self.volume(value)
    }

    fn restart_point(&mut self, value: impl Into<Num<u32, 8>>) -> &mut Self {
        self.restart_point(value)
    }

    fn playback(&mut self, playback_speed: impl Into<Num<u32, 8>>) -> &mut Self {
        self.playback(playback_speed)
    }

    fn panning(&mut self, panning: impl Into<Num<i16, 8>>) -> &mut Self {
        self.panning(panning)
    }

    fn set_pos(&mut self, pos: impl Into<Num<u32, 8>>) -> &mut Self {
        self.set_pos(pos)
    }
}

#[cfg(feature = "agb")]
impl Mixer for agb::sound::mixer::Mixer<'_> {
    type ChannelId = agb::sound::mixer::ChannelId;
    type SoundChannel = agb::sound::mixer::SoundChannel;

    fn channel(&mut self, channel_id: &Self::ChannelId) -> Option<&mut Self::SoundChannel> {
        self.channel(channel_id)
    }

    fn play_sound(&mut self, channel: Self::SoundChannel) -> Option<Self::ChannelId> {
        self.play_sound(channel)
    }
}

#[cfg(feature = "agb")]
/// The type to use if you're using agb-tracker with agb
pub type Tracker = TrackerInner<'static, agb::sound::mixer::ChannelId>;
