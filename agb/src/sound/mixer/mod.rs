#![warn(missing_docs)]
//! # agb mixer
//!
//! The agb software mixer allows for high performance playing of background music
//! and sound effects.
//!
//! Most games will need some form of sound effects or background music. The GBA has
//! no hardware sound mixer, so in order to play more than one sound at once, you
//! have to use a software mixer.
//!
//! agb's software mixer allows for up to 8 simultaneous sounds played at once at
//! various speeds and volumes.
//!
//! # Concepts
//!
//! The mixer runs at a fixed frequency which is determined at initialisation time by
//! passing certain [`Frequency`] options.
//!
//! All wav files you use within your application / game must use this _exact_ frequency.
//! If you don't use this frequency, the sound will play either too slowly or too quickly.
//!
//! The mixer can play both mono and stereo sounds, but only mono sound effects can have
//! effects applied to them (such as changing the speed at which they play or the panning).
//! Since the sound mixer runs in software, you must do some sound mixing every frame.
//!
//! ## Creating the mixer
//!
//! To create a sound mixer, you will need to get it out of the [`Gba`](crate::Gba) struct
//! as follows:
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../../doctest_runner.rs");
//! use agb::sound::mixer::Frequency;
//! # fn test(mut gba: agb::Gba) {
//! let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
//! mixer.enable();
//! # }
//! ```
//!
//! Pass a frequency option. This option must be used for the entire lifetime of the `mixer`
//! variable. If you want to change frequency, you will need to drop this one and create a new
//! one.
//!
//! ## Doing the per-frame work
//!
//! Despite being high performance, the mixer still takes a sizeable portion of CPU time (6-10%
//! depending on number of channels and frequency) to do the per-frame tasks, so should be done
//! towards the end of the frame time (just before waiting for vblank) in order to give as much
//! time during vblank as possible for rendering related tasks.
//!
//! In order to avoid skipping audio, call the [`Mixer::frame()`] function at least once per frame
//! as shown below:
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../../doctest_runner.rs");
//! use agb::sound::mixer::Frequency;
//! # fn test(mut gba: agb::Gba) {
//! let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
//! let vblank = agb::interrupt::VBlank::get();
//! // Somewhere in your main loop:
//! mixer.frame();
//! vblank.wait_for_vblank();
//! # }
//! ```
//!
//! ## Loading a sample
//!
//! To load a sample, you must have it in `wav` format (both stereo and mono work) at exactly the
//! selected frequency based on the features enabled in the agb crate.
//!
//! Use the [`include_wav!`](crate::include_wav) macro in order to load the sound. This will produce
//! an error if your wav file is of the wrong frequency.
//!
//! ```rust
//! # #![no_std]
//! # #![no_main]
//! # core::include!("../../doctest_runner.rs");
//! # fn test(mut gba: agb::Gba) {
//! # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
//! # let vblank = agb::interrupt::VBlank::get();
//! # use agb::{*, sound::mixer::*};
//! static MY_CRAZY_SOUND: SoundData = include_wav!("examples/sfx/jump.wav");
//!
//! // Then to play the sound:
//! let mut channel = SoundChannel::new(MY_CRAZY_SOUND);
//! channel.stereo();
//! let _ = mixer.play_sound(channel); // we don't mind if this sound doesn't actually play
//! # }
//! ```
//!
//! See the [`SoundChannel`] struct for more details on how you can configure the sounds to play.
//!
//! Once you have run [`play_sound`](Mixer::play_sound), the mixer will play that sound until
//! it has finished.
mod hw;
mod sw_mixer;

use core::slice;

pub use sw_mixer::ChannelId;
pub use sw_mixer::Mixer;

use crate::fixnum::Num;

/// Controls access to the mixer and the underlying hardware it uses. A zero sized type that
/// ensures that mixer access is exclusive.
#[non_exhaustive]
pub struct MixerController {}

impl MixerController {
    pub(crate) const fn new() -> Self {
        MixerController {}
    }

    /// Get a [`Mixer`] in order to start producing sounds.
    pub fn mixer(&mut self, frequency: Frequency) -> Mixer<'_> {
        Mixer::new(frequency)
    }
}

#[derive(PartialEq, Eq)]
enum SoundPriority {
    High,
    Low,
}

/// The supported frequencies within AGB.
///
/// These are chosen to work well with/ the hardware. Note that the higher
/// the frequency, the better the quality of the sound but the more CPU time
///  sound mixing will take.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Frequency {
    /// 10512Hz
    Hz10512,
    /// 18157Hz
    Hz18157,
    /// 32768Hz
    Hz32768,
}

// list here: http://deku.gbadev.org/program/sound1.html
impl Frequency {
    pub(crate) fn frequency(self) -> i32 {
        use Frequency::*;

        match self {
            Hz10512 => 10512,
            Hz18157 => 18157,
            Hz32768 => 32768,
        }
    }

    pub(crate) fn buffer_size(self) -> usize {
        use Frequency::*;

        match self {
            Hz10512 => 176,
            Hz18157 => 304,
            Hz32768 => 560,
        }
    }
}

/// Some sound data to play.
///
/// You probably shouldn't construct this yourself, instead relying on [`include_wav!()`](crate::include_wav).
#[repr(align(4))]
#[derive(Clone, Copy)]
pub struct SoundData {
    data: *const u8,
    len: usize,
}

impl SoundData {
    /// # Safety
    ///
    /// data must be 4-byte aligned. This can't be checked at compile time sadly...
    #[doc(hidden)]
    #[must_use]
    pub const unsafe fn new(data: &'static [u8]) -> Self {
        let len = data.len();
        let ptr = data.as_ptr();

        // check that ptr is correctly aligned
        // assert!((ptr as usize) & 3 == 0);

        Self { data: ptr, len }
    }

    #[must_use]
    pub(crate) fn data(&self) -> &'static [u8] {
        assert_eq!(self.data as usize & 3, 0, "SoundData not correctly aligned");

        // SAFETY: safe by construction
        unsafe { slice::from_raw_parts(self.data, self.len) }
    }
}

unsafe impl Send for SoundData {}
unsafe impl Sync for SoundData {}

/// Describes one sound which should be playing. This could be a sound effect or
/// the background music. Use the factory methods on this to modify how it is played.
///
/// You _must_ set stereo sounds with [`.stereo()`](SoundChannel::stereo) or it will play as mono and at
/// half the intended speed.
///
/// SoundChannels are very cheap to create, so don't worry about creating a brand new
/// one for every single sound you want to play.
///
/// SoundChannels can be either 'low priority' or 'high priority'. A high priority
/// sound channel will override 'low priority' sound channels which are already playing
/// to ensure that it is always running. A 'low priority' sound channel will not override
/// any other channel.
///
/// This is because you can only play up to 8 channels at once, and so high priority channels
/// are prioritised over low priority channels to ensure that sounds that you always want
/// playing will always play.
///
/// # Example
///
/// ## Playing background music (stereo)
///
/// Background music is generally considered 'high priority' because you likely want it to
/// play regardless of whether you have lots of sound effects playing. You create a high
/// priority sound channel using [`new_high_priority`](SoundChannel::new_high_priority).
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # core::include!("../../doctest_runner.rs");
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// static MY_BGM: SoundData = include_wav!("examples/sfx/my_bgm.wav");
///
/// // somewhere in code
/// # fn test(mut gba: Gba) {
/// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
/// let mut bgm = SoundChannel::new_high_priority(MY_BGM);
/// bgm.stereo().should_loop();
/// let _ = mixer.play_sound(bgm);
/// # }
/// ```
///
/// ## Playing a sound effect
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # core::include!("../../doctest_runner.rs");
/// # use agb::sound::mixer::*;
/// # use agb::*;
/// // in global scope:
/// static JUMP_SOUND: SoundData = include_wav!("examples/sfx/jump.wav");
///
/// // somewhere in code
/// # fn test(mut gba: Gba) {
/// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
/// let jump_sound = SoundChannel::new(JUMP_SOUND);
/// let _ = mixer.play_sound(jump_sound);
/// # }
/// ```
pub struct SoundChannel {
    data: &'static [u8],
    pos: Num<u32, 8>,
    should_loop: bool,
    restart_point: Num<u32, 8>,

    is_playing: bool,
    playback_speed: Num<u32, 8>,
    volume: Num<i16, 8>, // between 0 and 1

    panning: Num<i16, 8>, // between -1 and 1
    is_done: bool,

    is_stereo: bool,

    priority: SoundPriority,
}

impl SoundChannel {
    /// Creates a new low priority [`SoundChannel`].
    ///
    /// A low priority sound channel will be overridden by a high priority one if
    /// the mixer runs out of channels.
    ///
    /// Low priority sound channels are intended for sound effects.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// # core::include!("../../doctest_runner.rs");
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn test(mut gba: Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// // in global scope:
    /// static JUMP_SOUND: SoundData = include_wav!("examples/sfx/jump.wav");
    ///
    /// // somewhere in code
    /// let jump_sound = SoundChannel::new(JUMP_SOUND);
    /// let _ = mixer.play_sound(jump_sound);
    /// # }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new(data: SoundData) -> Self {
        SoundChannel {
            data: data.data(),
            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            is_playing: true,
            panning: 0.into(),
            is_done: false,
            priority: SoundPriority::Low,
            volume: 1.into(),
            is_stereo: false,
            restart_point: 0.into(),
        }
    }

    /// Creates a new high priority [`SoundChannel`].
    ///
    /// A high priority sound channel will override low priority ones if
    /// the mixer runs out of channels. They will also never be overridden
    /// by other high priority channels.
    ///
    /// High priority channels are intended for background music and for
    /// important, game breaking sound effects if you have any.
    ///
    /// # Example
    ///
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// # core::include!("../../doctest_runner.rs");
    /// # use agb::sound::mixer::*;
    /// # use agb::*;
    /// # fn test(mut gba: Gba) {
    /// # let mut mixer = gba.mixer.mixer(agb::sound::mixer::Frequency::Hz10512);
    /// // in global scope:
    /// static MY_BGM: SoundData = include_wav!("examples/sfx/my_bgm.wav");
    ///
    /// // somewhere in code
    /// let mut bgm = SoundChannel::new_high_priority(MY_BGM);
    /// bgm.stereo().should_loop();
    /// let _ = mixer.play_sound(bgm);
    /// # }
    /// ```
    #[inline(always)]
    #[must_use]
    pub fn new_high_priority(data: SoundData) -> Self {
        SoundChannel {
            data: data.data(),
            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            is_playing: true,
            panning: 0.into(),
            is_done: false,
            priority: SoundPriority::High,
            volume: 1.into(),
            is_stereo: false,
            restart_point: 0.into(),
        }
    }

    /// Sets that a sound channel should loop back to the start once it has
    /// finished playing rather than stopping.
    #[inline(always)]
    pub fn should_loop(&mut self) -> &mut Self {
        self.should_loop = true;
        self
    }

    /// Sets the point at which the sample should restart once it loops. Does nothing
    /// unless you also call [`should_loop()`](SoundChannel::should_loop()).
    ///
    /// Useful if your song has an introduction or similar.
    #[inline(always)]
    pub fn restart_point(&mut self, restart_point: impl Into<Num<u32, 8>>) -> &mut Self {
        self.restart_point = restart_point.into();
        assert!(
            self.restart_point.floor() as usize <= self.data.len(),
            "restart point must be shorter than the length of the sample"
        );
        self
    }

    /// Sets the speed at which this should channel should be played. Defaults
    /// to 1 with values between 0 and 1 being slower above 1 being faster.
    ///
    /// Note that this only works for mono sounds. Stereo sounds will not change
    /// how fast they play.
    #[inline(always)]
    pub fn playback(&mut self, playback_speed: impl Into<Num<u32, 8>>) -> &mut Self {
        self.playback_speed = playback_speed.into();
        self
    }

    /// Sets how far left or right the sound effect should be played.
    /// Must be a value between -1 and 1 (inclusive). -1 means fully played
    /// on the left, 1 fully on the right and values in between allowing for
    /// partial levels.
    ///
    /// Defaults to 0 (meaning equal on left and right) and doesn't affect stereo
    /// sounds.
    #[inline(always)]
    pub fn panning(&mut self, panning: impl Into<Num<i16, 8>>) -> &mut Self {
        let panning = panning.into();

        debug_assert!(panning >= Num::new(-1), "panning value must be >= -1");
        debug_assert!(panning <= Num::new(1), "panning value must be <= 1");

        self.panning = panning;
        self
    }

    /// Sets the volume for how loud the sound should be played. Note that if
    /// you play it too loud, the sound will clip sounding pretty terrible.
    ///
    /// Must be a value >= 0 and defaults to 1.
    #[inline(always)]
    pub fn volume(&mut self, volume: impl Into<Num<i16, 8>>) -> &mut Self {
        let volume = volume.into();

        assert!(volume >= Num::new(0), "volume must be >= 0");

        self.volume = volume;
        self
    }

    /// Sets that the sound effect should be played in stereo. Not setting this
    /// will result in the sound playing at half speed and mono. Setting this on
    /// a mono sound will cause some interesting results (and play it at double speed).
    #[inline(always)]
    pub fn stereo(&mut self) -> &mut Self {
        self.is_stereo = true;

        self
    }

    /// Stops the sound from playing.
    #[inline(always)]
    pub fn stop(&mut self) {
        self.is_done = true;
    }

    /// Gets how far along the sound has played.
    #[inline]
    #[must_use]
    pub fn pos(&self) -> Num<u32, 8> {
        self.pos
    }

    /// Sets the playback position
    #[inline]
    pub fn set_pos(&mut self, pos: impl Into<Num<u32, 8>>) -> &mut Self {
        self.pos = pos.into();
        self
    }

    /// Pause this channel. You can resume later by using [`.resume()`](SoundChannel::resume())
    #[inline]
    pub fn pause(&mut self) -> &mut Self {
        self.is_playing = false;
        self
    }

    /// Resume a paused channel paused by [`.pause()`](SoundChannel::pause())
    #[inline]
    pub fn resume(&mut self) -> &mut Self {
        self.is_playing = true;
        self
    }
}
