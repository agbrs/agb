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
//! const DB_TOFFE: Track = include_xm!("examples/db_toffe.xm");
//!
//! #[agb::entry]
//! fn main(mut gba: Gba) -> ! {
//!     let vblank_provider = agb::interrupt::VBlank::get();
//!
//!     let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
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
//! Note that currently you have to select 18157Hz as the frequency for the mixer.
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

use agb_tracker_interop::{PatternEffect, Sample};
use alloc::vec::Vec;

use agb::{
    fixnum::Num,
    sound::mixer::{ChannelId, Mixer, SoundChannel},
};

/// Import an XM file. Only available if you have the `xm` feature enabled (enabled by default).
#[cfg(feature = "xm")]
pub use agb_xm::include_xm;

#[doc(hidden)]
pub mod __private {
    pub use agb::fixnum::Num;
    pub use agb_tracker_interop;
}

/// A reference to a track. You should create this using one of the include macros.
pub use agb_tracker_interop::Track;

/// Stores the required state in order to play tracker music.
pub struct Tracker {
    track: &'static Track<'static>,
    channels: Vec<TrackerChannel>,

    frame: Num<u32, 8>,
    tick: u32,
    first: bool,

    current_row: usize,
    current_pattern: usize,
}

struct TrackerChannel {
    channel_id: Option<ChannelId>,
    base_speed: Num<u32, 8>,
    volume: Num<i16, 4>,
}

impl Tracker {
    /// Create a new tracker playing a specified track. See the [example](crate#example) for how to use the tracker.
    pub fn new(track: &'static Track<'static>) -> Self {
        let mut channels = Vec::new();
        channels.resize_with(track.num_channels, || TrackerChannel {
            channel_id: None,
            base_speed: 0.into(),
            volume: 0.into(),
        });

        Self {
            track,
            channels,

            frame: 0.into(),
            first: true,
            tick: 0,

            current_row: 0,
            current_pattern: 2,
        }
    }

    /// Call this once per frame before calling [`mixer.frame`](agb::sound::mixer::Mixer::frame()).
    /// See the [example](crate#example) for how to use the tracker.
    pub fn step(&mut self, mixer: &mut Mixer) {
        if !self.increment_frame() {
            return;
        }

        let pattern_to_play = self.track.patterns_to_play[self.current_pattern];
        let current_pattern = &self.track.patterns[pattern_to_play];

        let pattern_data_pos =
            current_pattern.start_position + self.current_row * self.track.num_channels;
        let pattern_slots =
            &self.track.pattern_data[pattern_data_pos..pattern_data_pos + self.track.num_channels];

        for (channel, pattern_slot) in self.channels.iter_mut().zip(pattern_slots) {
            if pattern_slot.sample != 0 && self.tick == 0 {
                let sample = &self.track.samples[pattern_slot.sample as usize - 1];
                channel.play_sound(mixer, sample);
            }

            if self.tick == 0 {
                channel.set_speed(mixer, pattern_slot.speed.change_base());
            }

            channel.apply_effect(mixer, &pattern_slot.effect1, self.tick);
            channel.apply_effect(mixer, &pattern_slot.effect2, self.tick);
        }

        self.increment_step();
    }

    fn increment_frame(&mut self) -> bool {
        if self.first {
            self.first = false;
            return true;
        }

        self.frame += 1;

        if self.frame >= self.track.frames_per_tick {
            self.tick += 1;
            self.frame -= self.track.frames_per_tick;

            if self.tick == self.track.ticks_per_step {
                self.current_row += 1;

                if self.current_row
                    >= self.track.patterns[self.track.patterns_to_play[self.current_pattern]].length
                {
                    self.current_pattern += 1;
                    self.current_row = 0;

                    if self.current_pattern >= self.track.patterns_to_play.len() {
                        self.current_pattern = 0;
                    }
                }

                self.tick = 0;
            }

            true
        } else {
            false
        }
    }

    fn increment_step(&mut self) {}
}

impl TrackerChannel {
    fn play_sound(&mut self, mixer: &mut Mixer<'_>, sample: &Sample<'static>) {
        if let Some(channel) = self
            .channel_id
            .take()
            .and_then(|channel_id| mixer.channel(&channel_id))
        {
            channel.stop();
        }

        let mut new_channel = SoundChannel::new(sample.data);

        new_channel.volume(sample.volume);

        if sample.should_loop {
            new_channel
                .should_loop()
                .restart_point(sample.restart_point);
        }

        self.channel_id = mixer.play_sound(new_channel);
        self.volume = 1.into();
    }

    fn set_speed(&mut self, mixer: &mut Mixer<'_>, speed: Num<u32, 8>) {
        if let Some(channel) = self
            .channel_id
            .as_ref()
            .and_then(|channel_id| mixer.channel(channel_id))
        {
            if speed != 0.into() {
                self.base_speed = speed;
            }

            channel.playback(self.base_speed);
        }
    }

    fn apply_effect(&mut self, mixer: &mut Mixer<'_>, effect: &PatternEffect, tick: u32) {
        if let Some(channel) = self
            .channel_id
            .as_ref()
            .and_then(|channel_id| mixer.channel(channel_id))
        {
            match effect {
                PatternEffect::None => {}
                PatternEffect::Stop => {
                    channel.stop();
                }
                PatternEffect::Arpeggio(first, second) => {
                    match tick % 3 {
                        0 => channel.playback(self.base_speed),
                        1 => channel.playback(first.change_base()),
                        2 => channel.playback(second.change_base()),
                        _ => unreachable!(),
                    };
                }
                PatternEffect::Panning(panning) => {
                    channel.panning(*panning);
                }
                PatternEffect::Volume(volume) => {
                    channel.volume(*volume);
                    self.volume = *volume;
                }
                PatternEffect::VolumeSlide(amount) => {
                    if tick != 0 {
                        self.volume += *amount;
                        if self.volume < 0.into() {
                            self.volume = 0.into();
                        }
                        channel.volume(self.volume);
                    }
                }
                PatternEffect::NoteCut(wait) => {
                    if tick == *wait {
                        channel.volume(0);
                        self.volume = 0.into();
                    }
                }
                PatternEffect::Portamento(amount) => {
                    let mut new_speed = self.base_speed;

                    for _ in 0..tick {
                        new_speed *= amount.change_base();
                    }

                    channel.playback(new_speed);
                }
            }
        }
    }
}

#[cfg(test)]
#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    loop {}
}
