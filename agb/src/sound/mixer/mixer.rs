use super::hw;
use super::hw::LeftOrRight;
use super::SoundChannel;

pub struct Mixer {
    buffer_l: MixerBuffer,
    buffer_r: MixerBuffer,
    channels: [Option<SoundChannel>; 16],
}

impl Mixer {
    pub(super) fn new() -> Self {
        Mixer {
            buffer_l: MixerBuffer::new(LeftOrRight::Left),
            buffer_r: MixerBuffer::new(LeftOrRight::Right),
            channels: Default::default(),
        }
    }

    pub fn enable(&self) {
        hw::set_timer_counter_for_frequency_and_enable(SOUND_FREQUENCY);
        hw::set_sound_control_register_for_mixer();
    }

    pub fn vblank(&mut self) {
        self.buffer_l.swap();
        self.buffer_r.swap();
        self.buffer_l.clear();
        self.buffer_r.clear();

        for channel in self.channels.iter_mut() {
            let mut has_finished = false;

            if let Some(some_channel) = channel {
                self.buffer_l.write_channel(some_channel);
                self.buffer_r.write_channel(some_channel);
                some_channel.pos += SOUND_BUFFER_SIZE;

                if some_channel.pos.floor() >= some_channel.data.len() {
                    if some_channel.should_loop {
                        some_channel.pos = 0.into();
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

    lr: LeftOrRight,
}

impl MixerBuffer {
    fn new(lr: LeftOrRight) -> Self {
        MixerBuffer {
            buffer1: [0; SOUND_BUFFER_SIZE],
            buffer2: [0; SOUND_BUFFER_SIZE],

            buffer_1_active: true,
            lr,
        }
    }

    fn swap(&mut self) {
        self.buffer_1_active = !self.buffer_1_active;

        if self.buffer_1_active {
            hw::enable_dma_for_sound(&self.buffer1, self.lr);
        } else {
            hw::enable_dma_for_sound(&self.buffer2, self.lr);
        }
    }

    fn clear(&mut self) {
        self.get_write_buffer().fill(0);
    }

    fn write_channel(&mut self, channel: &SoundChannel) {
        let data_to_copy = &channel.data[channel.pos.floor()..];
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
