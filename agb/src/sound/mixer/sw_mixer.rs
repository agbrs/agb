use core::cell::RefCell;
use core::pin::Pin;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bare_metal::{CriticalSection, Mutex};

use super::hw::LeftOrRight;
use super::{hw, Frequency};
use super::{SoundChannel, SoundPriority};

use crate::InternalAllocator;
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

    fn agb_rs__mixer_add_stereo(
        sound_data: *const u8,
        sound_buffer: *mut Num<i16, 4>,
        volume: Num<i16, 4>,
    );

    fn agb_rs__mixer_collapse(sound_buffer: *mut i8, input_buffer: *const Num<i16, 4>);
}

/// The main software mixer struct.
///
/// Tracks which sound channels are currently playing and handles actually playing them.
/// You should not create this struct directly, instead creating it through the [`Gba`](crate::Gba)
/// struct as follows:
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// # fn foo(gba: &mut Gba) {
/// let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
/// # }
/// ```
///
/// # Example
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// # fn foo(gba: &mut Gba) {
/// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
/// # let vblank = agb::interrupt::VBlank::get();
/// // Outside your main function in global scope:
/// const MY_CRAZY_SOUND: &[u8] = include_wav!("examples/sfx/jump.wav");
///
/// // in your main function:
/// let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
/// let mut channel = SoundChannel::new(MY_CRAZY_SOUND);
/// channel.stereo();
/// let _ = mixer.play_sound(channel);
///
/// loop {
///    mixer.frame();
///    vblank.wait_for_vblank();
///    mixer.after_vblank();
/// }
/// # }
/// ```
pub struct Mixer {
    interrupt_timer: Timer,
    // SAFETY: Has to go before buffer because it holds a reference to it
    _interrupt_handler: InterruptHandler<'static>,

    buffer: Pin<Box<MixerBuffer>>,
    channels: [Option<SoundChannel>; 8],
    indices: [i32; 8],
    frequency: Frequency,

    fifo_timer: Timer,
}

/// A pointer to a currently playing channel.
///
/// This is used to modify a channel that is already playing.
///
/// # Example
///
/// ```rust,no_run
/// # #![no_std]
/// # #![no_main]
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// # fn foo(gba: &mut Gba) {
/// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
/// # const MY_BGM: &[u8] = include_wav!("examples/sfx/my_bgm.wav");
/// let mut channel = SoundChannel::new_high_priority(MY_BGM);
/// let bgm_channel_id = mixer.play_sound(channel).unwrap(); // will always be Some if high priority
///
/// // Later, stop that particular channel
/// mixer.channel(&bgm_channel_id).expect("Expected to still be playing").stop();
/// # }
/// ```
pub struct ChannelId(usize, i32);

impl Mixer {
    pub(super) fn new(frequency: Frequency) -> Self {
        let buffer = Box::pin(MixerBuffer::new(frequency));

        // SAFETY: you can only ever have 1 Mixer at a time
        let fifo_timer = unsafe { Timer::new(0) };

        // SAFETY: you can only ever have 1 Mixer at a time
        let mut interrupt_timer = unsafe { Timer::new(1) };
        interrupt_timer
            .set_cascade(true)
            .set_divider(Divider::Divider1)
            .set_interrupt(true)
            .set_overflow_amount(frequency.buffer_size() as u16);

        let buffer_pointer_for_interrupt_handler: &MixerBuffer = &buffer;

        // SAFETY: dropping the lifetime, sound because interrupt handler dropped before the buffer is
        //         In the case of the mixer being forgotten, both stay alive so okay
        let buffer_pointer_for_interrupt_handler: &MixerBuffer =
            unsafe { core::mem::transmute(buffer_pointer_for_interrupt_handler) };
        let interrupt_handler = add_interrupt_handler(interrupt_timer.interrupt(), |cs| {
            buffer_pointer_for_interrupt_handler.swap(cs);
        });

        Self {
            frequency,
            buffer,
            channels: Default::default(),
            indices: Default::default(),

            interrupt_timer,
            _interrupt_handler: interrupt_handler,

            fifo_timer,
        }
    }

    /// Enable sound output
    ///
    /// You must call this method in order to start playing sound. You can do as much set up before
    /// this as you like, but you will not get any sound out of the console until this method is called.
    pub fn enable(&mut self) {
        hw::set_timer_counter_for_frequency_and_enable(
            &mut self.fifo_timer,
            self.frequency.frequency(),
        );
        hw::set_sound_control_register_for_mixer();

        self.interrupt_timer.set_enabled(true);
    }

    /// Do the CPU intensive mixing for the next frame's worth of data.
    ///
    /// This is where almost all of the CPU time for the mixer is done, and must be done every frame
    /// or you will get crackling sounds.
    ///
    /// Normally you would run this during vdraw, just before the vblank interrupt.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn foo(gba: &mut Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// # let vblank = agb::interrupt::VBlank::get();
    /// loop {
    ///     mixer.frame();
    ///     vblank.wait_for_vblank();
    /// }
    /// # }
    /// ```
    pub fn frame(&mut self) {
        if !self.buffer.should_calculate() {
            return;
        }

        self.buffer
            .write_channels(self.channels.iter_mut().flatten());
    }

    /// Start playing a given [`SoundChannel`].
    ///
    /// Returns a [`ChannelId`] which you can later use to modify the playing sound.
    ///
    /// Will first try to play the sound in an unused channel (of the 8 possible channels)
    /// followed by overriding a low priority sound (if the sound channel being passed in
    /// is high priority).
    ///
    /// Returns Some if the channel is now playing (which is guaranteed if the channel is
    /// high priority) or None if it failed to find a slot.
    ///
    /// Panics if you try to play a high priority sound and there are no free channels.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn foo(gba: &mut Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// # const MY_BGM: &[u8] = include_wav!("examples/sfx/my_bgm.wav");
    /// let mut channel = SoundChannel::new_high_priority(MY_BGM);
    /// let bgm_channel_id = mixer.play_sound(channel).unwrap(); // will always be Some if high priority
    /// # }
    /// ```
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

    /// Lets you modify an already playing channel.
    ///
    /// Allows you to change the volume, panning or stop an already playing channel.
    /// Will return Some if the channel is still playing, or None if it has already finished.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn foo(gba: &mut Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// # const MY_BGM: &[u8] = include_wav!("examples/sfx/my_bgm.wav");
    /// let mut channel = SoundChannel::new_high_priority(MY_BGM);
    /// let bgm_channel_id = mixer.play_sound(channel).unwrap(); // will always be Some if high priority
    ///
    /// // Later, stop that particular channel
    /// mixer.channel(&bgm_channel_id).expect("Expected still to be playing").stop();
    /// # }
    /// ```
    pub fn channel(&mut self, id: &ChannelId) -> Option<&'_ mut SoundChannel> {
        if let Some(channel) = &mut self.channels[id.0] {
            if self.indices[id.0] == id.1 && !channel.is_done {
                return Some(channel);
            }
        }

        None
    }
}

fn set_asm_buffer_size(frequency: Frequency) {
    extern "C" {
        static mut agb_rs__buffer_size: usize;
    }

    unsafe {
        agb_rs__buffer_size = frequency.buffer_size();
    }
}

struct SoundBuffer(Box<[i8], InternalAllocator>);

impl SoundBuffer {
    fn new(frequency: Frequency) -> Self {
        let my_size = frequency.buffer_size() * 2;
        let mut v = Vec::with_capacity_in(my_size, InternalAllocator);
        v.resize(my_size, 0);

        SoundBuffer(v.into_boxed_slice())
    }
}

struct MixerBuffer {
    buffers: [SoundBuffer; 3],
    working_buffer: Box<[Num<i16, 4>], InternalAllocator>,
    frequency: Frequency,

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
    fn new(frequency: Frequency) -> Self {
        let mut working_buffer =
            Vec::with_capacity_in(frequency.buffer_size() * 2, InternalAllocator);
        working_buffer.resize(frequency.buffer_size() * 2, 0.into());

        MixerBuffer {
            buffers: [
                SoundBuffer::new(frequency),
                SoundBuffer::new(frequency),
                SoundBuffer::new(frequency),
            ],

            working_buffer: working_buffer.into_boxed_slice(),

            state: Mutex::new(RefCell::new(MixerBufferState {
                active_buffer: 0,
                playing_buffer: 0,
            })),

            frequency,
        }
    }

    fn should_calculate(&self) -> bool {
        free(|cs| self.state.borrow(cs).borrow().should_calculate())
    }

    fn swap(&self, cs: CriticalSection) {
        let buffer = self.state.borrow(cs).borrow_mut().playing_advanced();

        let (left_buffer, right_buffer) = self.buffers[buffer]
            .0
            .split_at(self.frequency.buffer_size());

        hw::enable_dma_for_sound(left_buffer, LeftOrRight::Left);
        hw::enable_dma_for_sound(right_buffer, LeftOrRight::Right);
    }

    fn write_channels<'a>(&mut self, channels: impl Iterator<Item = &'a mut SoundChannel>) {
        set_asm_buffer_size(self.frequency);

        self.working_buffer.fill(0.into());

        for channel in channels {
            if channel.is_done {
                continue;
            }

            let playback_speed = if channel.is_stereo {
                2.into()
            } else {
                channel.playback_speed
            };

            if (channel.pos + playback_speed * self.frequency.buffer_size()).floor()
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
                        self.working_buffer.as_mut_ptr(),
                        channel.volume,
                    );
                }
            } else {
                let right_amount = ((channel.panning + 1) / 2) * channel.volume;
                let left_amount = ((-channel.panning + 1) / 2) * channel.volume;

                unsafe {
                    agb_rs__mixer_add(
                        channel.data.as_ptr().add(channel.pos.floor()),
                        self.working_buffer.as_mut_ptr(),
                        playback_speed,
                        left_amount,
                        right_amount,
                    );
                }
            }

            channel.pos += playback_speed * self.frequency.buffer_size();
        }

        let write_buffer_index = free(|cs| self.state.borrow(cs).borrow_mut().active_advanced());

        let write_buffer = &mut self.buffers[write_buffer_index].0;

        unsafe {
            agb_rs__mixer_collapse(write_buffer.as_mut_ptr(), self.working_buffer.as_ptr());
        }
    }
}
