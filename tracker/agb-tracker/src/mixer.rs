#![allow(missing_docs)]

use agb_fixnum::Num;

pub trait SoundChannel {
    fn new(data: &alloc::borrow::Cow<'static, [u8]>) -> Self;

    fn stop(&mut self);
    fn pause(&mut self) -> &mut Self;
    fn resume(&mut self) -> &mut Self;

    fn should_loop(&mut self) -> &mut Self;
    fn volume(&mut self, value: impl Into<Num<i16, 8>>) -> &mut Self;
    fn restart_point(&mut self, value: impl Into<Num<u32, 8>>) -> &mut Self;
    fn playback(&mut self, playback_speed: impl Into<Num<u32, 8>>) -> &mut Self;
    fn panning(&mut self, panning: impl Into<Num<i16, 8>>) -> &mut Self;
}

pub trait Mixer {
    type ChannelId;
    type SoundChannel: SoundChannel;

    fn channel(&mut self, channel_id: &Self::ChannelId) -> Option<&mut Self::SoundChannel>;
    fn play_sound(&mut self, channel: Self::SoundChannel) -> Option<Self::ChannelId>;
}
