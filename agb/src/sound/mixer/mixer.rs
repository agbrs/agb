use super::hw;
use super::hw::LeftOrRight;
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
            if let Some(some_channel) = channel {
                if self.buffer.write_channel(some_channel) {
                    channel.take();
                }
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
    buffer1: [i8; SOUND_BUFFER_SIZE * 2], // first half is left, second is right
    buffer2: [i8; SOUND_BUFFER_SIZE * 2],

    buffer_1_active: bool,
}

impl MixerBuffer {
    fn new() -> Self {
        MixerBuffer {
            buffer1: [0; SOUND_BUFFER_SIZE * 2],
            buffer2: [0; SOUND_BUFFER_SIZE * 2],

            buffer_1_active: true,
        }
    }

    fn swap(&mut self) {
        let (left_buffer, right_buffer) = self.get_write_buffer().split_at(SOUND_BUFFER_SIZE);

        hw::enable_dma_for_sound(left_buffer, LeftOrRight::Left);
        hw::enable_dma_for_sound(right_buffer, LeftOrRight::Right);

        self.buffer_1_active = !self.buffer_1_active;
    }

    fn clear(&mut self) {
        self.get_write_buffer().fill(0);
    }

    fn write_channel(&mut self, channel: &mut SoundChannel) -> bool {
        let place_to_write_to = self.get_write_buffer();
        let mut current_point = channel.pos;

        for i in 0..SOUND_BUFFER_SIZE {
            let v = channel.data[current_point.floor()];
            current_point += channel.playback_speed;

            if current_point.floor() >= channel.data.len() {
                if channel.should_loop {
                    channel.pos -= channel.data.len();
                } else {
                    return true;
                }
            }

            place_to_write_to[i] = place_to_write_to[i].saturating_add(v as i8);
            place_to_write_to[i + SOUND_BUFFER_SIZE] =
                place_to_write_to[i + SOUND_BUFFER_SIZE].saturating_add(v as i8);
        }

        false
    }

    fn get_write_buffer(&mut self) -> &mut [i8; SOUND_BUFFER_SIZE * 2] {
        if self.buffer_1_active {
            &mut self.buffer2
        } else {
            &mut self.buffer1
        }
    }
}
