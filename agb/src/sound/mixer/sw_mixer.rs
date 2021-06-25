use super::hw;
use super::hw::LeftOrRight;
use super::{SoundChannel, SoundPriority};
use crate::number::Num;

pub struct Mixer {
    buffer: MixerBuffer,
    channels: [Option<SoundChannel>; 16],
    indices: [i32; 16],
}

pub struct ChannelId(usize, i32);

impl Mixer {
    pub(super) fn new() -> Self {
        Mixer {
            buffer: MixerBuffer::new(),
            channels: Default::default(),
            indices: Default::default(),
        }
    }

    pub fn enable(&self) {
        hw::set_timer_counter_for_frequency_and_enable(SOUND_FREQUENCY);
        hw::set_sound_control_register_for_mixer();
    }

    pub fn vblank(&mut self) {
        self.buffer.swap();
        self.buffer.clear();

        self.buffer
            .write_channels(self.channels.iter_mut().flatten());
    }

    pub fn play_sound(&mut self, new_channel: SoundChannel) -> Option<ChannelId> {
        for (i, channel) in self.channels.iter_mut().enumerate() {
            if let Some(some_channel) = channel {
                if !some_channel.is_done {
                    continue;
                }
            }

            channel.replace(new_channel);
            self.indices[i] += 1;
            return Some(ChannelId(i, self.indices[i]));
        }

        if new_channel.priority == SoundPriority::Low {
            return None; // don't bother even playing it
        }

        for (i, channel) in self.channels.iter_mut().enumerate() {
            if channel.as_ref().unwrap().priority == SoundPriority::High {
                continue;
            }

            channel.replace(new_channel);
            self.indices[i] += 1;
            return Some(ChannelId(i, self.indices[i]));
        }

        panic!("Cannot play more than 16 sounds at once");
    }

    pub fn get_channel(&mut self, id: &ChannelId) -> Option<&'_ mut SoundChannel> {
        if let Some(channel) = &mut self.channels[id.0] {
            if self.indices[id.0] == id.1 {
                return Some(channel);
            }
        }

        None
    }
}

// I've picked one frequency that works nicely. But there are others that work nicely
// which we may want to consider in the future: http://deku.gbadev.org/program/sound1.html
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

    fn write_channels<'a>(&mut self, channels: impl Iterator<Item = &'a mut SoundChannel>) {
        let mut buffer: [Num<i16, 4>; SOUND_BUFFER_SIZE * 2] = [Num::new(0); SOUND_BUFFER_SIZE * 2];

        for channel in channels {
            if channel.is_done {
                continue;
            }

            let right_amount = (channel.panning + 1) / 2;
            let left_amount = -right_amount + 1;

            if channel.pos + channel.playback_speed * SOUND_BUFFER_SIZE >= channel.data.len().into()
            {
                // TODO: This should probably play what's left rather than skip the last bit
                if channel.should_loop {
                    channel.pos -= channel.data.len();
                } else {
                    channel.is_done = true;
                    continue;
                }
            }

            for i in 0..SOUND_BUFFER_SIZE {
                let v = (channel.data[channel.pos.floor()] as i8) as i16;
                channel.pos += channel.playback_speed;

                buffer[i] += left_amount * v;
                buffer[i + SOUND_BUFFER_SIZE] += right_amount * v;
            }
        }

        let write_buffer = self.get_write_buffer();
        for i in 0..SOUND_BUFFER_SIZE * 2 {
            write_buffer[i] = buffer[i].floor().clamp(i8::MIN as i16, i8::MAX as i16) as i8
        }
    }

    fn get_write_buffer(&mut self) -> &mut [i8; SOUND_BUFFER_SIZE * 2] {
        if self.buffer_1_active {
            &mut self.buffer2
        } else {
            &mut self.buffer1
        }
    }
}
