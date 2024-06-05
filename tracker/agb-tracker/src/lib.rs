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
//! use agb::{Gba, sound::mixer::Frequency};
//! use agb_tracker::{include_xm, Track, Tracker};
//!
//! static DB_TOFFE: Track = include_xm!("examples/db_toffe.xm");
//!
//! #[agb::entry]
//! fn main(mut gba: Gba) -> ! {
//!     let vblank_provider = agb::interrupt::VBlank::get();
//!
//!     let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
//!     mixer.enable();
//!
//!     let mut tracker = Tracker::new(&DB_TOFFE);
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

mod mixer;

use agb_tracker_interop::{PatternEffect, Sample};
use alloc::vec::Vec;

pub use mixer::{Mixer, SoundChannel};

use agb_fixnum::Num;

/// Import an XM file. Only available if you have the `xm` feature enabled (enabled by default).
#[cfg(feature = "xm")]
pub use agb_xm::include_xm;

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
pub struct Tracker<M: Mixer> {
    track: &'static Track,
    channels: Vec<TrackerChannel<M>>,
    envelopes: Vec<Option<EnvelopeState>>,

    frame: Num<u32, 8>,
    tick: u32,
    first: bool,

    global_settings: GlobalSettings,

    current_row: usize,
    current_pattern: usize,
}

struct TrackerChannel<M: Mixer> {
    channel_id: Option<M::ChannelId>,
    original_speed: Num<u32, 16>,
    base_speed: Num<u32, 16>,
    volume: Num<i32, 8>,
}

struct EnvelopeState {
    frame: usize,
    envelope_id: usize,
    finished: bool,
    fadeout: Num<i32, 8>,
}

#[derive(Clone)]
struct GlobalSettings {
    ticks_per_step: u32,

    frames_per_tick: Num<u32, 8>,
    volume: Num<i32, 8>,
}

impl<M: Mixer> Tracker<M> {
    /// Create a new tracker playing a specified track. See the [example](crate#example) for how to use the tracker.
    pub fn new(track: &'static Track) -> Self {
        let mut channels = Vec::new();
        channels.resize_with(track.num_channels, Default::default);

        let mut envelopes = Vec::new();
        envelopes.resize_with(track.num_channels, || None);

        let global_settings = GlobalSettings {
            ticks_per_step: track.ticks_per_step,
            frames_per_tick: track.frames_per_tick,
            volume: 1.into(),
        };

        Self {
            track,
            channels,
            envelopes,

            frame: 0.into(),
            first: true,
            tick: 0,

            global_settings,

            current_pattern: 0,
            current_row: 0,
        }
    }

    /// Call this once per frame before calling [`mixer.frame`](agb::sound::mixer::Mixer::frame()).
    /// See the [example](crate#example) for how to use the tracker.
    pub fn step(&mut self, mixer: &mut M) {
        if !self.increment_frame() {
            self.update_envelopes(mixer);
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
                channel.play_sound(mixer, sample, &self.global_settings);
                self.envelopes[i] = sample.volume_envelope.map(|envelope_id| EnvelopeState {
                    frame: 0,
                    envelope_id,
                    finished: false,
                    fadeout: sample.fadeout,
                });
            }

            if self.tick == 0 {
                channel.set_speed(mixer, pattern_slot.speed.change_base());
            }

            channel.apply_effect(
                mixer,
                &pattern_slot.effect1,
                self.tick,
                &mut self.global_settings,
                &mut self.envelopes[i],
            );
            channel.apply_effect(
                mixer,
                &pattern_slot.effect2,
                self.tick,
                &mut self.global_settings,
                &mut self.envelopes[i],
            );
        }

        self.update_envelopes(mixer);
    }

    fn update_envelopes(&mut self, mixer: &mut M) {
        for (channel, envelope_state_option) in self.channels.iter_mut().zip(&mut self.envelopes) {
            if let Some(envelope_state) = envelope_state_option {
                let envelope = &self.track.envelopes[envelope_state.envelope_id];

                if !channel.update_volume_envelope(
                    mixer,
                    envelope_state,
                    envelope,
                    &self.global_settings,
                ) {
                    envelope_state_option.take();
                } else {
                    envelope_state.frame += 1;

                    if !envelope_state.finished {
                        if let Some(sustain) = envelope.sustain {
                            if envelope_state.frame >= sustain {
                                envelope_state.frame = sustain;
                            }
                        }
                    }

                    if let Some(loop_end) = envelope.loop_end {
                        if envelope_state.frame >= loop_end {
                            envelope_state.frame = envelope.loop_start.unwrap_or(0);
                        }
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
                self.current_row += 1;

                if self.current_row
                    >= self.track.patterns[self.track.patterns_to_play[self.current_pattern]].length
                {
                    self.current_pattern += 1;
                    self.current_row = 0;

                    if self.current_pattern >= self.track.patterns_to_play.len() {
                        self.current_pattern = self.track.repeat;
                    }
                }

                self.tick = 0;
            }

            true
        } else {
            false
        }
    }
}

impl<M: Mixer> TrackerChannel<M> {
    fn play_sound(&mut self, mixer: &mut M, sample: &Sample, global_settings: &GlobalSettings) {
        if let Some(channel) = self
            .channel_id
            .take()
            .and_then(|channel_id| mixer.channel(&channel_id))
        {
            channel.stop();
        }

        let mut new_channel = M::SoundChannel::new(match sample.data {
            alloc::borrow::Cow::Borrowed(data) => data,
            alloc::borrow::Cow::Owned(_) => {
                unimplemented!("Must use borrowed COW data for tracker")
            }
        });

        new_channel.volume(
            (sample.volume.change_base() * global_settings.volume)
                .try_change_base()
                .unwrap(),
        );

        if sample.should_loop {
            new_channel
                .should_loop()
                .restart_point(sample.restart_point);
        }

        self.channel_id = mixer.play_sound(new_channel);
        self.volume = sample.volume.change_base();
    }

    fn set_speed(&mut self, mixer: &mut M, speed: Num<u32, 8>) {
        if let Some(channel) = self
            .channel_id
            .as_ref()
            .and_then(|channel_id| mixer.channel(channel_id))
        {
            if speed != 0.into() {
                self.base_speed = speed.change_base();
                self.original_speed = self.base_speed;
            }

            channel.playback(self.base_speed.change_base());
        }
    }

    fn apply_effect(
        &mut self,
        mixer: &mut M,
        effect: &PatternEffect,
        tick: u32,
        global_settings: &mut GlobalSettings,
        envelope_state: &mut Option<EnvelopeState>,
    ) {
        if let Some(channel) = self
            .channel_id
            .as_ref()
            .and_then(|channel_id| mixer.channel(channel_id))
        {
            match effect {
                PatternEffect::None => {}
                PatternEffect::Stop => {
                    channel.volume(0);
                    if let Some(envelope_state) = envelope_state {
                        envelope_state.finished = true;
                    }
                }
                PatternEffect::Arpeggio(first, second) => {
                    match tick % 3 {
                        0 => channel.playback(self.base_speed.change_base()),
                        1 => channel.playback(first.change_base()),
                        2 => channel.playback(second.change_base()),
                        _ => unreachable!(),
                    };
                }
                PatternEffect::Panning(panning) => {
                    channel.panning(panning.change_base());
                }
                PatternEffect::Volume(volume) => {
                    channel.volume(
                        (volume.change_base() * global_settings.volume)
                            .try_change_base()
                            .unwrap(),
                    );
                    self.volume = volume.change_base();
                }
                PatternEffect::VolumeSlide(amount) => {
                    if tick != 0 {
                        self.volume = (self.volume + amount.change_base()).max(0.into());
                        channel.volume(
                            (self.volume * global_settings.volume)
                                .try_change_base()
                                .unwrap(),
                        );
                    }
                }
                PatternEffect::FineVolumeSlide(amount) => {
                    if tick == 0 {
                        self.volume = (self.volume + amount.change_base()).max(0.into());
                        channel.volume(
                            (self.volume * global_settings.volume)
                                .try_change_base()
                                .unwrap(),
                        );
                    }
                }
                PatternEffect::NoteCut(wait) => {
                    if tick == *wait {
                        channel.volume(0);

                        if let Some(envelope_state) = envelope_state {
                            envelope_state.finished = true;
                        }
                    }
                }
                PatternEffect::NoteDelay(wait) => {
                    if tick < *wait {
                        channel.pause();
                    }

                    if tick == *wait {
                        channel.resume();
                        channel.volume(
                            (self.volume * global_settings.volume)
                                .try_change_base()
                                .unwrap(),
                        );
                    }
                }
                PatternEffect::Portamento(amount) => {
                    if tick != 0 {
                        self.base_speed *= amount.change_base();
                        channel.playback(self.base_speed.change_base());
                    }
                }
                PatternEffect::TonePortamento(amount, target) => {
                    channel.volume(
                        (self.volume * global_settings.volume)
                            .try_change_base()
                            .unwrap(),
                    );

                    if tick != 0 {
                        if *amount < 1.into() {
                            self.base_speed =
                                (self.base_speed * amount.change_base()).max(target.change_base());
                        } else {
                            self.base_speed =
                                (self.base_speed * amount.change_base()).min(target.change_base());
                        }
                    }

                    channel.playback(self.base_speed.change_base());
                }
                PatternEffect::PitchBend(amount) => {
                    if tick == 0 {
                        self.base_speed = self.original_speed * amount.change_base();
                        channel.playback(self.base_speed.change_base());
                    }
                }
                // These are global effects handled below
                PatternEffect::SetTicksPerStep(_)
                | PatternEffect::SetFramesPerTick(_)
                | PatternEffect::SetGlobalVolume(_)
                | PatternEffect::GlobalVolumeSlide(_) => {}
            }
        }

        // Some effects have to happen regardless of if we're actually playing anything
        match effect {
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
            _ => {}
        }
    }

    #[must_use]
    fn update_volume_envelope(
        &mut self,
        mixer: &mut M,
        envelope_state: &EnvelopeState,
        envelope: &agb_tracker_interop::Envelope,
        global_settings: &GlobalSettings,
    ) -> bool {
        if let Some(channel) = self
            .channel_id
            .as_ref()
            .and_then(|channel_id| mixer.channel(channel_id))
        {
            let amount = envelope.amount[envelope_state.frame];

            if envelope_state.finished {
                self.volume = (self.volume - envelope_state.fadeout).max(0.into());
            }

            channel.volume(
                (self.volume * amount.change_base() * global_settings.volume)
                    .try_change_base()
                    .unwrap(),
            );

            self.volume != 0.into()
        } else {
            false
        }
    }
}

impl<M: Mixer> Default for TrackerChannel<M> {
    fn default() -> Self {
        Self {
            channel_id: None,
            original_speed: Num::default(),
            base_speed: Num::default(),
            volume: Num::default(),
        }
    }
}

#[cfg(all(test, feature = "agb"))]
#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    loop {}
}

#[cfg(feature = "agb")]
impl SoundChannel for agb::sound::mixer::SoundChannel {
    fn new(data: &'static [u8]) -> Self {
        Self::new(data)
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
}

#[cfg(feature = "agb")]
impl<'gba> Mixer for agb::sound::mixer::Mixer<'gba> {
    type ChannelId = agb::sound::mixer::ChannelId;
    type SoundChannel = agb::sound::mixer::SoundChannel;

    fn channel(&mut self, channel_id: &Self::ChannelId) -> Option<&mut Self::SoundChannel> {
        self.channel(channel_id)
    }

    fn play_sound(&mut self, channel: Self::SoundChannel) -> Option<Self::ChannelId> {
        self.play_sound(channel)
    }
}
