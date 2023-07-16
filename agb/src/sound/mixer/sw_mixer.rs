use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::ControlFlow;
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
        playback_speed: Num<u32, 8>,
        left_amount: Num<i16, 4>,
        right_amount: Num<i16, 4>,
        buffer_size: usize,
    );

    fn agb_rs__mixer_add_first(
        sound_data: *const u8,
        sound_buffer: *mut Num<i16, 4>,
        playback_speed: Num<u32, 8>,
        left_amount: Num<i16, 4>,
        right_amount: Num<i16, 4>,
        buffer_size: usize,
    );

    fn agb_rs__mixer_add_stereo(
        sound_data: *const u8,
        sound_buffer: *mut Num<i16, 4>,
        volume: Num<i16, 4>,
        buffer_size: usize,
    );

    fn agb_rs__mixer_add_stereo_first(
        sound_data: *const u8,
        sound_buffer: *mut Num<i16, 4>,
        volume: Num<i16, 4>,
        buffer_size: usize,
    );

    fn agb_rs__mixer_collapse(
        sound_buffer: *mut i8,
        input_buffer: *const Num<i16, 4>,
        num_samples: usize,
    );
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
/// }
/// # }
/// ```
pub struct Mixer<'gba> {
    interrupt_timer: Timer,
    // SAFETY: Has to go before buffer because it holds a reference to it
    _interrupt_handler: InterruptHandler,

    buffer: Pin<Box<MixerBuffer, InternalAllocator>>,
    channels: [Option<SoundChannel>; 8],
    indices: [i32; 8],
    frequency: Frequency,

    working_buffer: Box<[Num<i16, 4>], InternalAllocator>,

    fifo_timer: Timer,

    phantom: PhantomData<&'gba ()>,
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

impl Mixer<'_> {
    pub(super) fn new(frequency: Frequency) -> Self {
        let buffer = Box::pin_in(MixerBuffer::new(frequency), InternalAllocator);

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
        let interrupt_handler = unsafe {
            add_interrupt_handler(interrupt_timer.interrupt(), |cs| {
                buffer_pointer_for_interrupt_handler.swap(cs);
            })
        };

        let mut working_buffer =
            Vec::with_capacity_in(frequency.buffer_size() * 2, InternalAllocator);
        working_buffer.resize(frequency.buffer_size() * 2, 0.into());

        Self {
            frequency,
            buffer,
            channels: Default::default(),
            indices: Default::default(),

            interrupt_timer,
            _interrupt_handler: interrupt_handler,

            working_buffer: working_buffer.into_boxed_slice(),
            fifo_timer,

            phantom: PhantomData,
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
            .write_channels(&mut self.working_buffer, self.channels.iter_mut().flatten());
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
    frequency: Frequency,

    state: Mutex<RefCell<MixerBufferState>>,
}

struct MixerBufferState {
    active_buffer: usize,
    playing_buffer: usize,
    buffers: [SoundBuffer; 3],
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
        mod3_estimate(self.active_buffer + 1) != self.playing_buffer
    }

    fn playing_advanced(&mut self) -> *const i8 {
        self.playing_buffer = mod3_estimate(self.playing_buffer + 1);
        self.buffers[self.playing_buffer].0.as_ptr()
    }

    fn active_advanced(&mut self) -> *mut i8 {
        self.active_buffer = mod3_estimate(self.active_buffer + 1);
        self.buffers[self.active_buffer].0.as_mut_ptr()
    }
}

impl MixerBuffer {
    fn new(frequency: Frequency) -> Self {
        MixerBuffer {
            state: Mutex::new(RefCell::new(MixerBufferState {
                active_buffer: 0,
                playing_buffer: 0,
                buffers: [
                    SoundBuffer::new(frequency),
                    SoundBuffer::new(frequency),
                    SoundBuffer::new(frequency),
                ],
            })),

            frequency,
        }
    }

    fn should_calculate(&self) -> bool {
        free(|cs| self.state.borrow(cs).borrow().should_calculate())
    }

    fn swap(&self, cs: CriticalSection) {
        let buffer = self.state.borrow(cs).borrow_mut().playing_advanced();

        let left_buffer = buffer;
        // SAFETY: starting pointer is fine, resulting pointer also fine because buffer has length buffer_size() * 2 by construction
        let right_buffer = unsafe { buffer.add(self.frequency.buffer_size()) };

        hw::enable_dma_for_sound(left_buffer, LeftOrRight::Left);
        hw::enable_dma_for_sound(right_buffer, LeftOrRight::Right);
    }

    fn write_channels<'a>(
        &self,
        working_buffer: &mut [Num<i16, 4>],
        channels: impl Iterator<Item = &'a mut SoundChannel>,
    ) {
        working_buffer.fill(0.into());

        for channel in channels.filter(|channel| !channel.is_done) {
            if channel.volume != 0.into() {
                if channel.is_stereo {
                    self.write_stereo(channel, working_buffer);
                } else {
                    self.write_mono(channel, working_buffer);
                }
            }
        }

        let write_buffer = free(|cs| self.state.borrow(cs).borrow_mut().active_advanced());

        unsafe {
            agb_rs__mixer_collapse(
                write_buffer,
                working_buffer.as_ptr(),
                self.frequency.buffer_size(),
            );
        }
    }

    fn write_stereo(&self, channel: &mut SoundChannel, working_buffer: &mut [Num<i16, 4>]) {
        if (channel.pos + 2 * self.frequency.buffer_size() as u32).floor()
            >= channel.data.len() as u32
        {
            if channel.should_loop {
                channel.pos = channel.restart_point * 2;
            } else {
                channel.is_done = true;
                return;
            }
        }
        unsafe {
            agb_rs__mixer_add_stereo(
                channel.data.as_ptr().add(channel.pos.floor() as usize),
                working_buffer.as_mut_ptr(),
                channel.volume,
                self.frequency.buffer_size(),
            );
        }

        channel.pos += 2 * self.frequency.buffer_size() as u32;
    }

    fn write_mono(&self, channel: &mut SoundChannel, working_buffer: &mut [Num<i16, 4>]) {
        let right_amount = ((channel.panning + 1) / 2) * channel.volume;
        let left_amount = ((-channel.panning + 1) / 2) * channel.volume;

        let channel_len = Num::<u32, 8>::new(channel.data.len() as u32);
        let mut playback_speed = channel.playback_speed;

        while playback_speed >= channel_len - channel.restart_point {
            playback_speed -= channel_len;
        }

        // SAFETY: always aligned correctly by construction
        let working_buffer_i32: &mut [i32] = unsafe {
            core::slice::from_raw_parts_mut(
                working_buffer.as_mut_ptr().cast(),
                working_buffer.len() / 2,
            )
        };

        let mul_amount =
            ((left_amount.to_raw() as i32) << 16) | (right_amount.to_raw() as i32 & 0x0000ffff);

        for i in 0..self.frequency.buffer_size() {
            if channel.pos >= channel_len {
                if channel.should_loop {
                    channel.pos -= channel_len + channel.restart_point;
                } else {
                    channel.is_done = true;
                    break;
                }
            }

            // SAFETY: channel.pos < channel_len by the above if statement and the fact we reduce the playback speed
            let value =
                unsafe { *channel.data.get_unchecked(channel.pos.floor() as usize) } as i8 as i32;

            // SAFETY: working buffer length = self.frequency.buffer_size()
            unsafe { *working_buffer_i32.get_unchecked_mut(i) += value * mul_amount };
            channel.pos += playback_speed;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::fixnum::num;
    use alloc::vec;

    use super::*;

    #[test_case]
    fn collapse_should_correctly_reduce_size_of_input(_: &mut crate::Gba) {
        #[repr(align(4))]
        struct AlignedNumbers<const N: usize>([Num<i16, 4>; N]);

        let input = &AlignedNumbers([
            num!(10.0),
            num!(10.0),
            num!(5.0),
            num!(5.0),
            num!(-10.0),
            num!(-10.5),
            num!(-5.9),
            num!(-5.2),
            num!(0.0),
            num!(1.1),
            num!(2.2),
            num!(3.3),
            num!(155.4),
            num!(-230.5),
            num!(400.6),
            num!(-700.7),
            num!(10.0),
            num!(10.0),
            num!(5.0),
            num!(5.0),
            num!(-10.0),
            num!(-10.5),
            num!(-5.9),
            num!(-5.2),
            num!(0.0),
            num!(1.1),
            num!(2.2),
            num!(3.3),
            num!(155.4),
            num!(-230.5),
            num!(400.6),
            num!(-700.7),
        ]);

        let input = &input.0;

        let mut output_buffer = vec![0i32; input.len() / 4];

        unsafe {
            agb_rs__mixer_collapse(
                output_buffer.as_mut_ptr().cast(),
                input.as_ptr(),
                input.len() / 2,
            );
        }

        // output will be unzipped, so input is LRLRLRLRLRLRLR... and output is LLLLLLRRRRRR
        assert_eq!(
            output_buffer
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .map(|x| x as i8)
                .collect::<alloc::vec::Vec<_>>(),
            &[
                10, 5, -10, -6, 0, 2, 127, 127, 10, 5, -10, -6, 0, 2, 127, 127, 10, 5, -11, -6, 1,
                3, -128, -128, 10, 5, -11, -6, 1, 3, -128, -128
            ]
        );
    }
}
