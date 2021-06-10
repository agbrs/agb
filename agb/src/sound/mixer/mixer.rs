use super::hw;
use super::SoundChannel;

pub struct Mixer {
    buffer: MixerBuffer,
    channels: [Option<SoundChannel>; 16],
}

impl Mixer {
    pub(super) fn new() -> Self {
        Mixer {
            buffer: MixerBuffer::new(),
            channels: Default::default(),
        }
    }

    pub fn enable(&self) {
        hw::set_timer_counter_for_frequency_and_enable(SOUND_FREQUENCY);
        hw::set_sound_control_register_for_mixer();
    }

    pub fn vblank(&mut self) {
        self.buffer.swap();
        self.buffer.clear();

        for channel in self.channels.iter_mut() {
            let mut has_finished = false;

            if let Some(some_channel) = channel {
                self.buffer.write_channel(some_channel);
                some_channel.pos += SOUND_BUFFER_SIZE;

                if some_channel.pos >= some_channel.data.len() {
                    if some_channel.should_loop {
                        some_channel.pos = 0;
                    } else {
                        has_finished = true;
                    }
                }
            }

            if has_finished {
                channel.take();
            }
        }
    }

    pub fn play_sound(&mut self, new_channel: SoundChannel) {
        for channel in self.channels.iter_mut() {
            if channel.is_some() {
                continue;
            }

            channel.replace(new_channel);
            return;
        }

        panic!("Cannot play more than 16 sounds at once");
    }
}

// I've picked one frequency that works nicely. But there are others that work nicely
// which we may want to consider in the future: https://web.archive.org/web/20070608011909/http://deku.gbadev.org/program/sound1.html
const SOUND_FREQUENCY: i32 = 10512;
const SOUND_BUFFER_SIZE: usize = 176;

struct MixerBuffer {
    buffer1: [i8; SOUND_BUFFER_SIZE],
    buffer2: [i8; SOUND_BUFFER_SIZE],

    buffer_1_active: bool,
}

impl MixerBuffer {
    fn new() -> Self {
        MixerBuffer {
            buffer1: [0; SOUND_BUFFER_SIZE],
            buffer2: [0; SOUND_BUFFER_SIZE],

            buffer_1_active: true,
        }
    }

    fn swap(&mut self) {
        self.buffer_1_active = !self.buffer_1_active;

        if self.buffer_1_active {
            hw::enable_dma1_for_sound(&self.buffer1);
        } else {
            hw::enable_dma1_for_sound(&self.buffer2);
        }
    }

    fn clear(&mut self) {
        self.get_write_buffer().fill(0);
    }

    fn write_channel(&mut self, channel: &SoundChannel) {
        let data_to_copy = &channel.data[channel.pos..];
        let place_to_write_to = self.get_write_buffer();

        for (i, v) in data_to_copy.iter().take(SOUND_BUFFER_SIZE).enumerate() {
            let v = *v as i8;
            place_to_write_to[i] = place_to_write_to[i].saturating_add(v);
        }
    }

    fn get_write_buffer(&mut self) -> &mut [i8; SOUND_BUFFER_SIZE] {
        if self.buffer_1_active {
            &mut self.buffer2
        } else {
            &mut self.buffer1
        }
    }
}
