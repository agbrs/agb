use core::cell::RefCell;
use core::marker::PhantomData;

use alloc::boxed::Box;
use alloc::vec::Vec;
use critical_section::{CriticalSection, Mutex};

use super::hw::LeftOrRight;
use super::{Frequency, hw};
use super::{SoundChannel, SoundPriority};

use crate::{
    InternalAllocator,
    dma::dma_copy16,
    fixnum::{Num, num},
    interrupt::{InterruptHandler, add_interrupt_handler},
    timer::Divider,
    timer::Timer,
};

macro_rules! add_mono_fn {
    ($name:ident) => {
        fn $name(
            sample_data: *const u8,
            sample_buffer: *mut i32,
            buffer_size: usize,
            restart_amount: Num<u32, 8>,
            channel_length: usize,
            current_pos: Num<u32, 8>,
            playback_speed: Num<u32, 8>,
            mul_amount: i32,
        ) -> Num<u32, 8>;
    };
}

// Defined in mixer.s
unsafe extern "C" {
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

    add_mono_fn!(agb_rs__mixer_add_mono_loop_first);
    add_mono_fn!(agb_rs__mixer_add_mono_loop);
    add_mono_fn!(agb_rs__mixer_add_mono_first);
    add_mono_fn!(agb_rs__mixer_add_mono);
}

/// The main software mixer struct.
///
/// Tracks which sound channels are currently playing and handles actually playing them.
/// You should not create this struct directly, instead creating it through the [`Gba`](crate::Gba)
/// struct as follows:
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// # #[agb::doctest]
/// # fn test(mut gba: Gba) {
/// let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
/// # }
/// ```
///
/// # Example
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// # #[agb::doctest]
/// # fn test(mut gba: Gba) {
/// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
/// # let mut gfx = gba.graphics.get();
/// // Outside your main function in global scope:
/// static MY_CRAZY_SOUND: SoundData = include_wav!("examples/sfx/jump.wav");
///
/// // in your main function:
/// let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
/// let mut channel = SoundChannel::new(MY_CRAZY_SOUND);
/// channel.stereo();
/// let _ = mixer.play_sound(channel);
///
/// loop {
///    let mut frame = gfx.frame();
///    // do your game updating and rendering
///    mixer.frame();
///    frame.commit();
/// # break;
/// }
/// # }
/// ```
pub struct Mixer<'gba> {
    interrupt_timer: Timer,
    // SAFETY: Has to go before buffer because it holds a reference to it
    _interrupt_handler: InterruptHandler,

    buffer: raw_box::RawBoxDrop<MixerBuffer, InternalAllocator>,
    channels: [Option<SoundChannel>; 8],
    indices: [i32; 8],
    frequency: Frequency,

    working_buffer: Box<[Num<i16, 4>], InternalAllocator>,
    /// Copy all the data into here first before acting on it if it is deemed to
    /// be faster to do so since using DMA3 to copy from ROM into IWRAM is faster
    /// than just reading from ROM.
    temp_storage: Box<[u8], InternalAllocator>,

    fifo_timer: Timer,

    phantom: PhantomData<&'gba ()>,
}

/// A pointer to a currently playing channel.
///
/// This is used to modify a channel that is already playing.
///
/// # Example
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// # #[agb::doctest]
/// # fn test(mut gba: Gba) {
/// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
/// # static MY_BGM: SoundData = include_wav!("examples/sfx/my_bgm.wav");
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
        let buffer =
            raw_box::RawBoxDrop::new(Box::new_in(MixerBuffer::new(frequency), InternalAllocator));

        // SAFETY: you can only ever have 1 Mixer at a time
        let fifo_timer = unsafe { Timer::new(0) };

        // SAFETY: you can only ever have 1 Mixer at a time
        let mut interrupt_timer = unsafe { Timer::new(1) };
        interrupt_timer
            .set_cascade(true)
            .set_divider(Divider::Divider1)
            .set_interrupt(true)
            .set_overflow_amount(frequency.buffer_size() as u16);

        struct SendPtr<T>(*const T);
        unsafe impl<T> Send for SendPtr<T> {}
        unsafe impl<T> Sync for SendPtr<T> {}

        let ptr_for_interrupt_handler = SendPtr(&*buffer);

        // SAFETY: the interrupt handler will be dropped before the buffer is so this never accesses
        //         freed memory. Also the dereference happens in a critical section to ensure that
        //         we don't end up accessing while dropping
        let interrupt_handler = unsafe {
            add_interrupt_handler(interrupt_timer.interrupt(), move |cs| {
                // needed to ensure that rust doesn't only capture the field
                let _ = &ptr_for_interrupt_handler;

                (*ptr_for_interrupt_handler.0).swap(cs);
            })
        };

        let mut working_buffer =
            Vec::with_capacity_in(frequency.buffer_size() * 2, InternalAllocator);
        working_buffer.resize(frequency.buffer_size() * 2, 0.into());

        let mut temp_storage =
            Vec::with_capacity_in(frequency.buffer_size() * 3 / 2 + 1, InternalAllocator);
        temp_storage.resize(temp_storage.capacity(), 0);

        let mut result = Self {
            frequency,
            buffer,
            channels: Default::default(),
            indices: Default::default(),

            interrupt_timer,
            _interrupt_handler: interrupt_handler,

            working_buffer: working_buffer.into_boxed_slice(),
            temp_storage: temp_storage.into_boxed_slice(),
            fifo_timer,

            phantom: PhantomData,
        };
        result.enable();
        result
    }

    fn enable(&mut self) {
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
    /// It is safe to call this more than once per frame, but it is very important to call it at least once per frame.
    /// Calling it more than once in a single frame will result in the second call being ignored.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # #[agb::doctest]
    /// # fn test(mut gba: Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// # let mut gfx = gba.graphics.get();
    /// # let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
    /// loop {
    ///    let mut frame = gfx.frame();
    ///    // do your game updating and rendering
    ///    mixer.frame();
    ///    frame.commit();
    /// # break;
    /// }
    /// # }
    /// ```
    pub fn frame(&mut self) {
        if !self.buffer.should_calculate() {
            return;
        }

        self.buffer.write_channels(
            &mut self.working_buffer,
            &mut self.temp_storage,
            self.channels.iter_mut().flatten(),
        );
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
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # #[agb::doctest]
    /// # fn test(mut gba: Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// # static MY_BGM: SoundData = include_wav!("examples/sfx/my_bgm.wav");
    /// let mut channel = SoundChannel::new_high_priority(MY_BGM);
    /// let bgm_channel_id = mixer.play_sound(channel).unwrap(); // will always be Some if high priority
    /// # }
    /// ```
    pub fn play_sound(&mut self, new_channel: SoundChannel) -> Option<ChannelId> {
        for (i, channel) in self.channels.iter_mut().enumerate() {
            if let Some(some_channel) = channel
                && !some_channel.is_done
            {
                continue;
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
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # #[agb::doctest]
    /// # fn test(mut gba: Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// # static MY_BGM: SoundData = include_wav!("examples/sfx/my_bgm.wav");
    /// let mut channel = SoundChannel::new_high_priority(MY_BGM);
    /// let bgm_channel_id = mixer.play_sound(channel).unwrap(); // will always be Some if high priority
    ///
    /// // Later, stop that particular channel
    /// mixer.channel(&bgm_channel_id).expect("Expected still to be playing").stop();
    /// # }
    /// ```
    pub fn channel(&mut self, id: &ChannelId) -> Option<&'_ mut SoundChannel> {
        if let Some(channel) = &mut self.channels[id.0]
            && self.indices[id.0] == id.1
            && !channel.is_done
        {
            return Some(channel);
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
        critical_section::with(|cs| self.state.borrow_ref_mut(cs).should_calculate())
    }

    fn swap(&self, cs: CriticalSection) {
        let buffer = self.state.borrow_ref_mut(cs).playing_advanced();

        let left_buffer = buffer;
        // SAFETY: starting pointer is fine, resulting pointer also fine because buffer has length buffer_size() * 2 by construction
        let right_buffer = unsafe { buffer.add(self.frequency.buffer_size()) };

        hw::enable_dma_for_sound(left_buffer, LeftOrRight::Left);
        hw::enable_dma_for_sound(right_buffer, LeftOrRight::Right);
    }

    fn write_channels<'a>(
        &self,
        working_buffer: &mut [Num<i16, 4>],
        temp_storage: &mut [u8],
        channels: impl Iterator<Item = &'a mut SoundChannel>,
    ) {
        let mut channels = channels
            .filter(|channel| !channel.is_done && channel.volume != 0.into() && channel.is_playing);

        if let Some(channel) = channels.next() {
            if channel.is_stereo {
                self.write_stereo(channel, working_buffer, true);
            } else {
                self.write_mono(channel, working_buffer, temp_storage, true);
            }
        } else {
            working_buffer.fill(0.into());
        }

        for channel in channels {
            if channel.is_stereo {
                self.write_stereo(channel, working_buffer, false);
            } else {
                self.write_mono(channel, working_buffer, temp_storage, false);
            }
        }

        let write_buffer =
            critical_section::with(|cs| self.state.borrow_ref_mut(cs).active_advanced());

        unsafe {
            agb_rs__mixer_collapse(
                write_buffer,
                working_buffer.as_ptr(),
                self.frequency.buffer_size(),
            );
        }
    }

    fn write_stereo(
        &self,
        channel: &mut SoundChannel,
        working_buffer: &mut [Num<i16, 4>],
        is_first: bool,
    ) {
        if (channel.pos + 2 * self.frequency.buffer_size() as u32).floor()
            >= channel.data.len() as u32
        {
            if channel.should_loop {
                channel.pos = channel.restart_point * 2;
            } else {
                channel.is_done = true;
                if is_first {
                    working_buffer.fill(0.into());
                }
                return;
            }
        }
        unsafe {
            if is_first {
                agb_rs__mixer_add_stereo_first(
                    channel.data.as_ptr().add(channel.pos.floor() as usize),
                    working_buffer.as_mut_ptr(),
                    channel.volume.change_base(),
                    self.frequency.buffer_size(),
                );
            } else {
                agb_rs__mixer_add_stereo(
                    channel.data.as_ptr().add(channel.pos.floor() as usize),
                    working_buffer.as_mut_ptr(),
                    channel.volume.change_base(),
                    self.frequency.buffer_size(),
                );
            }
        }

        channel.pos += 2 * self.frequency.buffer_size() as u32;
    }

    fn write_mono(
        &self,
        channel: &mut SoundChannel,
        working_buffer: &mut [Num<i16, 4>],
        temp_storage: &mut [u8],
        is_first: bool,
    ) {
        let right_amount = ((channel.panning + 1) / 2) * channel.volume;
        let left_amount = ((-channel.panning + 1) / 2) * channel.volume;

        let right_amount: Num<i16, 4> = right_amount.change_base();
        let left_amount: Num<i16, 4> = left_amount.change_base();

        let channel_len = Num::<u32, 8>::new(channel.data.len() as u32);

        // SAFETY: always aligned correctly by construction
        let working_buffer_i32: &mut [i32] = unsafe {
            core::slice::from_raw_parts_mut(
                working_buffer.as_mut_ptr().cast(),
                working_buffer.len() / 2,
            )
        };

        let mul_amount =
            ((left_amount.to_raw() as i32) << 16) | (right_amount.to_raw() as i32 & 0x0000ffff);

        let playback_buffer =
            playback_buffer::PlaybackBuffer::new(channel, self.frequency, temp_storage);

        macro_rules! call_mono_fn {
            ($fn_name:ident) => {
                channel.pos = unsafe {
                    $fn_name(
                        playback_buffer.as_ptr(),
                        working_buffer_i32.as_mut_ptr(),
                        working_buffer_i32.len(),
                        channel_len - channel.restart_point,
                        channel.data.len(),
                        channel.pos,
                        channel.playback_speed,
                        mul_amount,
                    )
                }
            };
        }

        match (is_first, channel.should_loop) {
            (true, true) => call_mono_fn!(agb_rs__mixer_add_mono_loop_first),
            (false, true) => call_mono_fn!(agb_rs__mixer_add_mono_loop),
            (true, false) => {
                call_mono_fn!(agb_rs__mixer_add_mono_first);
                channel.is_done = channel.pos >= channel_len;
            }
            (false, false) => {
                call_mono_fn!(agb_rs__mixer_add_mono);
                channel.is_done = channel.pos >= channel_len;
            }
        }
    }
}

mod playback_buffer {
    use super::*;

    /// Sometimes it is faster to copy the sound data out of ROM first into RAM and then play
    /// it from there. This is because sequential reads from ROM are much faster than random reads
    /// and DMA3 can do sequential reads the whole way across, and in IWRAM, random reads are
    /// exactly the same speed as sequential reads.
    ///
    /// The mixer mainly has to do random reads because it has to handle playback speeds which
    /// aren't exactly 1. So in cases where copying the data first is faster than just reading it
    /// the once, we copy into a temporary buffer set aside specifically for this purpose.
    pub(super) enum PlaybackBuffer<'a> {
        /// This is the temporary buffer set aside for copying the data into.
        ///
        /// The second field in the enum is the offset to subtract from it which we might need to
        /// do because we could be playing back somewhere in the middle of this section and we want
        /// to pretend that we're actually playing from somewhere else.
        TempStorage(&'a [u8], usize),
        /// Just read the data from ROM.
        Rom(&'static [u8]),
    }

    impl<'a> PlaybackBuffer<'a> {
        pub(super) fn new(
            channel: &SoundChannel,
            frequency: Frequency,
            temp_storage: &'a mut [u8],
        ) -> Self {
            let channel_len = Num::new(channel.data.len() as u32);

            // 1.5 is approximately the multiple we can work with before it would be faster
            // to read from ROM rather than do the copy. We also allow copying the entire channel
            // to the temporary buffer because we'll end up looping it multiple times if the playback
            // speed is so high.
            //
            // If increasing this size, make sure to also increase the size of the temp_storage
            // allocation since this guards overrunning that.
            if channel.playback_speed > num!(1.5) && channel.data.len() > temp_storage.len() {
                return PlaybackBuffer::Rom(channel.data);
            }

            let total_to_play =
                (channel.playback_speed * frequency.buffer_size() as u32).floor() + 1;

            if channel_len <= total_to_play.into() {
                // We're going to play the entire sample (at least once) so copy the entire
                // sample into memory.
                assert!((channel_len.floor() as usize / 2) * 2 <= temp_storage.len());

                unsafe {
                    dma_copy16(
                        channel.data.as_ptr().cast(),
                        temp_storage.as_mut_ptr().cast(),
                        channel_len.floor() as usize / 2,
                    );
                }

                PlaybackBuffer::TempStorage(temp_storage, 0)
            } else if channel.pos + total_to_play > channel_len {
                // The playback is going to loop. We don't handle this case (yet) but
                // fortunately it doesn't come up as often as the other two cases.
                PlaybackBuffer::Rom(channel.data)
            } else {
                // We're not going to loop, and not going to play the entire sample. So
                // we'll copy as much over as we can.
                assert!((total_to_play as usize / 2 + 1) * 2 <= temp_storage.len());

                unsafe {
                    dma_copy16(
                        channel.data[channel.pos.floor() as usize..].as_ptr().cast(),
                        temp_storage.as_mut_ptr().cast(),
                        total_to_play as usize / 2 + 1,
                    );
                }

                // The offset here is so we can pretend that the whole channel exists,
                // but the copy methods will immediately add the pos on and won't know
                // that around the small bit is actuall junk.
                PlaybackBuffer::TempStorage(temp_storage, channel.pos.floor() as usize)
            }
        }

        pub(super) fn as_ptr(&self) -> *const u8 {
            match self {
                PlaybackBuffer::TempStorage(items, offset) => {
                    items.as_ptr().wrapping_byte_sub(*offset)
                }
                PlaybackBuffer::Rom(items) => items.as_ptr(),
            }
        }
    }
}

mod raw_box {
    use core::ops::Deref;

    use alloc::boxed::Box;

    pub struct RawBoxDrop<T, A: Clone + alloc::alloc::Allocator>(*mut T, A);

    impl<T, A: Clone + alloc::alloc::Allocator> RawBoxDrop<T, A> {
        pub fn new(inner: Box<T, A>) -> Self {
            let (ptr, allocator) = Box::into_raw_with_allocator(inner);
            Self(ptr, allocator)
        }
    }

    impl<T, A: Clone + alloc::alloc::Allocator> Deref for RawBoxDrop<T, A> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            unsafe { &*self.0 }
        }
    }

    impl<T, A: Clone + alloc::alloc::Allocator> Drop for RawBoxDrop<T, A> {
        fn drop(&mut self) {
            unsafe { Box::from_raw_in(self.0, self.1.clone()) };
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

    #[test_case]
    fn mono_add_loop_first_should_work(_: &mut crate::Gba) {
        let mut buffer = vec![0i32; 16];
        let sample_data: [i8; 9] = [5, 10, 0, 100, -18, 55, 8, -120, 19];
        let restart_amount = num!(9.0);
        let current_pos = num!(0.0);
        let playback_speed = num!(1.0);

        let mul_amount = 10;

        let result = unsafe {
            agb_rs__mixer_add_mono_loop_first(
                sample_data.as_ptr().cast(),
                buffer.as_mut_ptr(),
                buffer.len(),
                restart_amount,
                sample_data.len(),
                current_pos,
                playback_speed,
                mul_amount,
            )
        };

        assert_eq!(
            buffer,
            &[
                50, 100, 0, 1000, -180, 550, 80, -1200, 190, 50, 100, 0, 1000, -180, 550, 80
            ]
        );
        assert_eq!(result, num!(7.0));
    }
}
