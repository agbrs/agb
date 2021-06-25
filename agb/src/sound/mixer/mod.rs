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
    volume: Num<i16, 4>, // between 0 and 1

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
            volume: 1.into(),
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
            volume: 1.into(),
        }
    }

    pub fn should_loop(&mut self) -> &mut Self {
        self.should_loop = true;
        self
    }

    pub fn playback(&mut self, playback_speed: Num<usize, 8>) -> &mut Self {
        self.playback_speed = playback_speed;
        self
    }

    pub fn panning(&mut self, panning: Num<i16, 4>) -> &mut Self {
        debug_assert!(panning >= Num::new(-1), "panning value must be >= -1");
        debug_assert!(panning <= Num::new(1), "panning value must be <= 1");

        self.panning = panning;
        self
    }

    pub fn volume(&mut self, volume: Num<i16, 4>) -> &mut Self {
        assert!(volume <= Num::new(1), "volume must be <= 1");
        assert!(volume >= Num::new(0), "volume must be >= 0");

        self.volume = volume;
        self
    }

    pub fn stop(&mut self) {
        self.is_done = true
    }
}
