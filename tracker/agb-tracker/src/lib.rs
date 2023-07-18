#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use agb_tracker_interop::{PatternEffect, Sample};
use alloc::vec::Vec;

use agb::{
    fixnum::Num,
    sound::mixer::{ChannelId, Mixer, SoundChannel},
};

#[cfg(feature = "xm")]
pub use agb_xm::import_xm;

pub mod __private {
    pub use agb::fixnum::Num;
    pub use agb_tracker_interop;
}

pub use agb_tracker_interop::Track;

pub struct Tracker {
    track: &'static Track<'static>,
    channels: Vec<TrackerChannel>,

    frame: Num<u16, 8>,
    tick: u16,
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
            current_pattern: 0,
        }
    }

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

            channel.apply_effect(
                mixer,
                &pattern_slot.effect1,
                self.tick,
                pattern_slot.speed.change_base(),
            );
            channel.apply_effect(
                mixer,
                &pattern_slot.effect2,
                self.tick,
                pattern_slot.speed.change_base(),
            );
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
        self.channel_id
            .take()
            .and_then(|channel_id| mixer.channel(&channel_id))
            .map(|channel| channel.stop());

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

    fn apply_effect(
        &mut self,
        mixer: &mut Mixer<'_>,
        effect: &PatternEffect,
        tick: u16,
        speed: Num<u32, 8>,
    ) {
        if let Some(channel) = self
            .channel_id
            .as_ref()
            .and_then(|channel_id| mixer.channel(&channel_id))
        {
            if speed != 0.into() {
                channel.playback(speed);
                self.base_speed = speed;
            }

            channel.playback(self.base_speed);

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
            }
        }
    }
}

#[cfg(test)]
#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    loop {}
}
