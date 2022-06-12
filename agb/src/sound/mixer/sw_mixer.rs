use core::cell::RefCell;
use core::mem;
use core::mem::MaybeUninit;

use bare_metal::{CriticalSection, Mutex};

use super::hw;
use super::hw::LeftOrRight;
use super::{SoundChannel, SoundPriority};

use crate::{
    fixnum::Num,
    interrupt::free,
    interrupt::{add_interrupt_handler, InterruptHandler},
    timer::Divider,
    timer::Timer,
};

// Defined in mixer.s
extern "C" {
    fn agb_rs__mixer_add(
        sound_data: *const u8,
        sound_buffer: *mut Num<i16, 4>,
        playback_speed: Num<usize, 8>,
        left_amount: Num<i16, 4>,
        right_amount: Num<i16, 4>,
    );

    fn agb_rs__mixer_add_stereo(sound_data: *const u8, sound_buffer: *mut Num<i16, 4>);

    fn agb_rs__mixer_collapse(sound_buffer: *mut i8, input_buffer: *const Num<i16, 4>);

    fn agb_rs__init_buffer(buffer: *mut MaybeUninit<Num<i16, 4>>, size_in_bytes: usize);
}

pub struct Mixer {
    buffer: MixerBuffer,
    channels: [Option<SoundChannel>; 8],
    indices: [i32; 8],

    timer: Timer,
}

pub struct ChannelId(usize, i32);

impl Mixer {
    pub(super) fn new() -> Self {
        Self {
            buffer: MixerBuffer::new(),
            channels: Default::default(),
            indices: Default::default(),

            timer: unsafe { Timer::new(0) },
        }
    }

    pub fn enable(&mut self) {
        hw::set_timer_counter_for_frequency_and_enable(&mut self.timer, constants::SOUND_FREQUENCY);
        hw::set_sound_control_register_for_mixer();
    }

    #[cfg(not(feature = "freq32768"))]
    pub fn after_vblank(&mut self) {
        free(|cs| self.buffer.swap(cs));
    }

    /// Note that if you set up an interrupt handler, you should not call `after_vblank` any more
    /// You are still required to call `frame`
    pub fn setup_interrupt_handler(&self) -> InterruptHandler<'_> {
        let mut timer1 = unsafe { Timer::new(1) };
        timer1
            .set_cascade(true)
            .set_divider(Divider::Divider1)
            .set_interrupt(true)
            .set_overflow_amount(constants::SOUND_BUFFER_SIZE as u16)
            .set_enabled(true);

        add_interrupt_handler(timer1.interrupt(), move |cs| self.buffer.swap(cs))
    }

    pub fn frame(&mut self) {
        if !self.buffer.should_calculate() {
            return;
        }

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

        panic!("Cannot play more than 8 sounds at once");
    }

    pub fn channel(&mut self, id: &ChannelId) -> Option<&'_ mut SoundChannel> {
        if let Some(channel) = &mut self.channels[id.0] {
            if self.indices[id.0] == id.1 && !channel.is_done {
                return Some(channel);
            }
        }

        None
    }
}

// These work perfectly with swapping the buffers every vblank
// list here: http://deku.gbadev.org/program/sound1.html
#[cfg(all(not(feature = "freq18157"), not(feature = "freq32768")))]
mod constants {
    pub const SOUND_FREQUENCY: i32 = 10512;
    pub const SOUND_BUFFER_SIZE: usize = 176;
}

#[cfg(feature = "freq18157")]
mod constants {
    pub const SOUND_FREQUENCY: i32 = 18157;
    pub const SOUND_BUFFER_SIZE: usize = 304;
}

#[cfg(feature = "freq32768")]
mod constants {
    pub const SOUND_FREQUENCY: i32 = 32768;
    pub const SOUND_BUFFER_SIZE: usize = 560;
}

fn set_asm_buffer_size() {
    extern "C" {
        static mut agb_rs__buffer_size: usize;
    }

    unsafe {
        agb_rs__buffer_size = constants::SOUND_BUFFER_SIZE;
    }
}

#[repr(C, align(4))]
struct SoundBuffer([i8; constants::SOUND_BUFFER_SIZE * 2]);

impl Default for SoundBuffer {
    fn default() -> Self {
        Self([0; constants::SOUND_BUFFER_SIZE * 2])
    }
}

struct MixerBuffer {
    buffers: [SoundBuffer; 3],

    state: Mutex<RefCell<MixerBufferState>>,
}

struct MixerBufferState {
    active_buffer: usize,
    playing_buffer: usize,
}

/// Only returns a valid result if 0 <= x <= 3
const fn mod3_estimate(x: usize) -> usize {
    match x & 0b11 {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 0,
        _ => unreachable!(),
    }
}

impl MixerBufferState {
    fn should_calculate(&self) -> bool {
        mod3_estimate(self.active_buffer + 1) != mod3_estimate(self.playing_buffer)
    }

    fn playing_advanced(&mut self) -> usize {
        self.playing_buffer = mod3_estimate(self.playing_buffer + 1);
        self.playing_buffer
    }

    fn active_advanced(&mut self) -> usize {
        self.active_buffer = mod3_estimate(self.active_buffer + 1);
        self.active_buffer
    }
}

impl MixerBuffer {
    fn new() -> Self {
        set_asm_buffer_size();

        MixerBuffer {
            buffers: Default::default(),

            state: Mutex::new(RefCell::new(MixerBufferState {
                active_buffer: 0,
                playing_buffer: 0,
            })),
        }
    }

    fn should_calculate(&self) -> bool {
        free(|cs| self.state.borrow(cs).borrow().should_calculate())
    }

    fn swap(&self, cs: CriticalSection) {
        let buffer = self.state.borrow(cs).borrow_mut().playing_advanced();

        let (left_buffer, right_buffer) = self.buffers[buffer]
            .0
            .split_at(constants::SOUND_BUFFER_SIZE);

        hw::enable_dma_for_sound(left_buffer, LeftOrRight::Left);
        hw::enable_dma_for_sound(right_buffer, LeftOrRight::Right);
    }

    fn write_channels<'a>(&mut self, channels: impl Iterator<Item = &'a mut SoundChannel>) {
        // This code is equivalent to:
        // let mut buffer: [Num<i16, 4>; constants::SOUND_BUFFER_SIZE * 2] =
        //     [Num::new(0); constants::SOUND_BUFFER_SIZE * 2];
        // but the above uses approximately 7% of the CPU time if running at 32kHz
        let mut buffer: [Num<i16, 4>; constants::SOUND_BUFFER_SIZE * 2] = {
            // Create an uninitialized array of `MaybeUninit`. The `assume_init` is
            // safe because the type we are claiming to have initialized here is a
            // bunch of `MaybeUninit`s, which do not require initialization.
            let mut data: [MaybeUninit<Num<i16, 4>>; constants::SOUND_BUFFER_SIZE * 2] =
                unsafe { MaybeUninit::uninit().assume_init() };

            // Actually init the array (by filling it with zeros) and then transmute it (which is safe because
            // we have now zeroed everything)
            unsafe {
                agb_rs__init_buffer(
                    data.as_mut_ptr(),
                    mem::size_of::<Num<i16, 4>>() * data.len(),
                );

                mem::transmute(data)
            }
        };

        for channel in channels {
            if channel.is_done {
                continue;
            }

            let playback_speed = if channel.is_stereo {
                2.into()
            } else {
                channel.playback_speed
            };

            if (channel.pos + playback_speed * constants::SOUND_BUFFER_SIZE).floor()
                >= channel.data.len()
            {
                // TODO: This should probably play what's left rather than skip the last bit
                if channel.should_loop {
                    channel.pos = 0.into();
                } else {
                    channel.is_done = true;
                    continue;
                }
            }

            if channel.is_stereo {
                unsafe {
                    agb_rs__mixer_add_stereo(
                        channel.data.as_ptr().add(channel.pos.floor()),
                        buffer.as_mut_ptr(),
                    );
                }
            } else {
                let right_amount = ((channel.panning + 1) / 2) * channel.volume;
                let left_amount = ((-channel.panning + 1) / 2) * channel.volume;

                unsafe {
                    agb_rs__mixer_add(
                        channel.data.as_ptr().add(channel.pos.floor()),
                        buffer.as_mut_ptr(),
                        playback_speed,
                        left_amount,
                        right_amount,
                    );
                }
            }

            channel.pos += playback_speed * constants::SOUND_BUFFER_SIZE;
        }

        let write_buffer_index = free(|cs| self.state.borrow(cs).borrow_mut().active_advanced());

        let write_buffer = &mut self.buffers[write_buffer_index].0;

        unsafe {
            agb_rs__mixer_collapse(write_buffer.as_mut_ptr(), buffer.as_ptr());
        }
    }
}
