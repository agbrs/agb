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
//! The mixer runs at a fixed frequency which is determined at compile time by enabling
//! certain features within the crate. The following features are currently available:
//!
//! | Feature | Frequency |
//! |---------|-----------|
//! | none    | 10512Hz   |
//! | freq18157 | 18157Hz |
//! | freq32768[^32768Hz] | 32768Hz |
//!
//! All wav files you use within your application / game must use this _exact_ frequency.
//! You will get a compile error if you use the incorrect frequency for your file.
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
//! ```
//! let mut mixer = gba.mixer.mixer();
//! mixer.enable();
//! ```
//!
//! ## Doing the per-frame work
//!
//! Then, you have a choice of whether you want to use interrupts or do the buffer swapping
//! yourself after a vblank interrupt. If you are using 32768Hz as the frequency of your
//! files, you _must_ use the interrupt version.
//!
//! Without interrupts:
//!
//! ```
//! // Somewhere in your main loop:
//! mixer.frame();
//! vblank.wait_for_vblank();
//! mixer.after_vblank();
//! ```
//!
//! Or with interrupts:
//!
//! ```
//! // outside your main loop, close to initialisation
//! let _mixer_interrupt = mixer.setup_interrupt_handler();
//!
//! // inside your main loop
//! mixer.frame();
//! vblank.wait_for_vblank();
//! ```
//!
//! Despite being high performance, the mixer still takes a sizable portion of CPU time (6-10% 
//! depending on number of channels and frequency) to do the per-frame tasks, so should be done
//! towards the end of the frame time (just before waiting for vblank) in order to give as much 
//! time during vblank as possible for rendering related tasks.
//!
//! ## Loading a sample
//!
//! To load a sample, you must have it in `wav` format (both stereo and mono work) at exactly the
//! selected frequency based on the features enabled in the agb crate.
//!
//! Use the [`include_wav!`](crate::include_wav) macro in order to load the sound. This will produce
//! an error if your wav file is of the wrong frequency.
//!
//! ```
//! // Outside your main function in global scope:
//! const MY_CRAZY_SOUND: &[u8] = include_wav!("sfx/my_crazy_sound.wav");
//! 
//! // Then to play the sound:
//! let mut channel = SoundChannel::new(MY_CRAZY_SOUND);
//! channel.stereo();
//! let _ = mixer.play_sound(channel); // we don't mind if this sound doesn't actually play
//! ```
//!
//! See the [`SoundChannel`] struct for more details on how you can configure the sounds to play.
//!
//! Once you have run [`play_sound`](Mixer::play_sound), the mixer will play that sound until
//! it has finished.
//!
//! [^32768Hz]: You must use interrupts when using 32768Hz

mod hw;
mod sw_mixer;

pub use sw_mixer::ChannelId;
pub use sw_mixer::Mixer;

use crate::fixnum::Num;

#[non_exhaustive]
pub struct MixerController {}

impl MixerController {
    pub(crate) const fn new() -> Self {
        MixerController {}
    }

    pub fn mixer(&mut self) -> Mixer {
        Mixer::new()
    }
}

#[derive(PartialEq, Eq)]
enum SoundPriority {
    High,
    Low,
}

pub struct SoundChannel {
    data: &'static [u8],
    pos: Num<usize, 8>,
    should_loop: bool,

    playback_speed: Num<usize, 8>,
    volume: Num<i16, 4>, // between 0 and 1

    panning: Num<i16, 4>, // between -1 and 1
    is_done: bool,

    is_stereo: bool,

    priority: SoundPriority,
}

impl SoundChannel {
    #[inline(always)]
    #[must_use]
    pub fn new(data: &'static [u8]) -> Self {
        SoundChannel {
            data,
            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            panning: 0.into(),
            is_done: false,
            priority: SoundPriority::Low,
            volume: 1.into(),
            is_stereo: false,
        }
    }

    #[inline(always)]
    #[must_use]
    pub fn new_high_priority(data: &'static [u8]) -> Self {
        SoundChannel {
            data,
            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            panning: 0.into(),
            is_done: false,
            priority: SoundPriority::High,
            volume: 1.into(),
            is_stereo: false,
        }
    }

    #[inline(always)]
    pub fn should_loop(&mut self) -> &mut Self {
        self.should_loop = true;
        self
    }

    #[inline(always)]
    pub fn playback(&mut self, playback_speed: Num<usize, 8>) -> &mut Self {
        self.playback_speed = playback_speed;
        self
    }

    #[inline(always)]
    pub fn panning(&mut self, panning: Num<i16, 4>) -> &mut Self {
        debug_assert!(panning >= Num::new(-1), "panning value must be >= -1");
        debug_assert!(panning <= Num::new(1), "panning value must be <= 1");

        self.panning = panning;
        self
    }

    #[inline(always)]
    pub fn volume(&mut self, volume: Num<i16, 4>) -> &mut Self {
        assert!(volume >= Num::new(0), "volume must be >= 0");

        self.volume = volume;
        self
    }

    #[inline(always)]
    pub fn stereo(&mut self) -> &mut Self {
        self.is_stereo = true;

        self
    }

    #[inline(always)]
    pub fn stop(&mut self) {
        self.is_done = true;
    }
}
