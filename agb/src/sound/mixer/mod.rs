mod hw;
mod sw_mixer;

pub use sw_mixer::Mixer;

use crate::number::Num;

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

    panning: Num<i16, 4>, // between -1 and 1
    is_done: bool,

    priority: SoundPriority,
}

impl SoundChannel {
    pub fn new(data: &'static [u8]) -> Self {
        SoundChannel {
            data,
            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            panning: 0.into(),
            is_done: false,
            priority: SoundPriority::Low,
        }
    }

    pub fn new_high_priority(data: &'static [u8]) -> Self {
        SoundChannel {
            data,
            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            panning: 0.into(),
            is_done: false,
            priority: SoundPriority::High,
        }
    }

    pub fn should_loop<'a>(&'a mut self) -> &'a mut Self {
        self.should_loop = true;
        self
    }

    pub fn playback<'a>(&'a mut self, playback_speed: Num<usize, 8>) -> &'a mut Self {
        self.playback_speed = playback_speed;
        self
    }

    pub fn panning<'a>(&'a mut self, panning: Num<i16, 4>) -> &'a mut Self {
        debug_assert!(panning >= Num::new(-1), "panning value must be >= -1");
        debug_assert!(panning <= Num::new(1), "panning value must be <= 1");

        self.panning = panning;
        self
    }
}
