use core::cell::RefCell;
use core::intrinsics::transmute;

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
/// let mut mixer = gba.mixer.mixer();
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
/// # let mut mixer = gba.mixer.mixer();
/// # let vblank = agb::interrupt::VBlank::get();
/// // Outside your main function in global scope:
/// const MY_CRAZY_SOUND: &[u8] = include_wav!("examples/sfx/jump.wav");
///
/// // in your main function:
/// let mut mixer = gba.mixer.mixer();
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
    buffer: MixerBuffer,
    channels: [Option<SoundChannel>; 8],
    indices: [i32; 8],

    timer: Timer,
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
/// # let mut mixer = gba.mixer.mixer();
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
    pub(super) fn new() -> Self {
        Self {
            buffer: MixerBuffer::new(),
            channels: Default::default(),
            indices: Default::default(),

            timer: unsafe { Timer::new(0) },
        }
    }

    /// Enable sound output
    ///
    /// You must call this method in order to start playing sound. You can do as much set up before
    /// this as you like, but you will not get any sound out of the console until this method is called.
    pub fn enable(&mut self) {
        hw::set_timer_counter_for_frequency_and_enable(&mut self.timer, constants::SOUND_FREQUENCY);
        hw::set_sound_control_register_for_mixer();
    }

    /// Do post-vblank work. You can use either this or [`setup_interrupt_handler()`](Mixer::setup_interrupt_handler),
    /// but not both. Note that this is not available if using 32768Hz sounds since those require more irregular timings.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn foo(gba: &mut Gba) {
    /// # let mut mixer = gba.mixer.mixer();
    /// # let vblank = agb::interrupt::VBlank::get();
    /// loop {
    ///     mixer.frame();
    ///     vblank.wait_for_vblank();
    ///     mixer.after_vblank();   
    /// }
    /// # }
    /// ```
    #[cfg(not(feature = "freq32768"))]
    pub fn after_vblank(&mut self) {
        free(|cs| self.buffer.swap(cs));
    }

    /// Use timer interrupts to do the timing required for ensuring the music runs smoothly.
    ///
    /// Note that if you set up an interrupt handler, you should not call [`after_vblank`](Mixer::after_vblank) any more
    /// You are still required to call [`frame`](Mixer::frame).
    ///
    /// This is required if using 32768Hz music, but optional for other frequencies.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![no_std]
    /// # #![no_main]
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn foo(gba: &mut Gba) {
    /// # let mut mixer = gba.mixer.mixer();
    /// # let vblank = agb::interrupt::VBlank::get();
    /// // you must set this to a named variable to ensure that the scope is long enough
    /// let _mixer_interrupt = mixer.setup_interrupt_handler();
    ///
    /// loop {
    ///    mixer.frame();
    ///    vblank.wait_for_vblank();
    /// }
    /// # }
    /// ```
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
    /// # let mut mixer = gba.mixer.mixer();
    /// # let vblank = agb::interrupt::VBlank::get();
    /// loop {
    ///     mixer.frame();
    ///     vblank.wait_for_vblank();
    ///     mixer.after_vblank(); // optional, only if not using interrupts
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
    /// # let mut mixer = gba.mixer.mixer();
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
    /// # let mut mixer = gba.mixer.mixer();
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
        let mut buffer: [Num<i16, 4>; constants::SOUND_BUFFER_SIZE * 2] =
            unsafe { transmute([0i16; constants::SOUND_BUFFER_SIZE * 2]) };

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
