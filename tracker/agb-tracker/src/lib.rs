#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

use alloc::vec::Vec;

use agb::sound::mixer::{ChannelId, Mixer, SoundChannel};

#[cfg(feature = "xm")]
pub use agb_xm::import_xm;

pub use agb_tracker_interop as __private;

pub use __private::Track;

pub struct Tracker {
    track: &'static Track<'static>,
    channels: Vec<Option<ChannelId>>,

    step: u16,
    current_row: usize,
    current_pattern: usize,
}

impl Tracker {
    pub fn new(track: &'static Track<'static>) -> Self {
        let mut channels = Vec::new();
        channels.resize_with(track.num_channels, || None);

        Self {
            track,
            channels,

            step: 0,
            current_row: 0,
            current_pattern: 0,
        }
    }

    pub fn step(&mut self, mixer: &mut Mixer) {
        if self.step != 0 {
            self.increment_step();
            return; // TODO: volume / pitch slides
        }

        let current_pattern = &self.track.patterns[self.current_pattern];

        let pattern_data_pos =
            current_pattern.start_position + self.current_row * self.track.num_channels;
        let pattern_slots =
            &self.track.pattern_data[pattern_data_pos..pattern_data_pos + self.track.num_channels];

        for (channel_id, pattern_slot) in self.channels.iter_mut().zip(pattern_slots) {
            if pattern_slot.sample == 0 {
                if pattern_slot.speed == 0.into() {
                    if let Some(channel) = channel_id
                        .take()
                        .and_then(|channel_id| mixer.channel(&channel_id))
                    {
                        channel.stop();
                    }
                }
            } else {
                if let Some(channel) = channel_id
                    .take()
                    .and_then(|channel_id| mixer.channel(&channel_id))
                {
                    channel.stop();
                }

                let sample = &self.track.samples[pattern_slot.sample - 1];
                let mut new_channel = SoundChannel::new(sample.data);
                new_channel
                    .panning(pattern_slot.panning)
                    .volume(pattern_slot.volume)
                    .playback(pattern_slot.speed);

                if sample.should_loop {
                    new_channel.should_loop();
                }

                *channel_id = mixer.play_sound(new_channel);
            }
        }

        self.increment_step();
    }

    fn increment_step(&mut self) {
        self.step += 1;

        if self.step == self.track.frames_per_step * 2 {
            self.current_row += 1;

            if self.current_row > self.track.patterns[self.current_pattern].length {
                self.current_pattern += 1;
                self.current_row = 0;
            }

            self.step = 0;
        }
    }
}

#[cfg(test)]
#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    loop {}
}
