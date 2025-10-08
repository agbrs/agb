use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use agb::sound::mixer::{Frequency, MixerController, SoundChannel, SoundData};

/// Error type for sound operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundError;

impl core::fmt::Display for SoundError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Sound operation failed")
    }
}

/// Async wrapper for agb sound operations
pub struct AsyncMixer<'a> {
    mixer: agb::sound::mixer::Mixer<'a>,
}

impl<'a> AsyncMixer<'a> {
    pub(crate) fn new(mixer_controller: &'a mut MixerController, frequency: Frequency) -> Self {
        Self {
            mixer: mixer_controller.mixer(frequency),
        }
    }

    /// Process one frame of audio asynchronously
    ///
    /// This should be called once per frame, typically at 60Hz
    pub async fn frame(&mut self) {
        FrameFuture::new(&mut self.mixer).await
    }

    /// Play a sound and return its channel ID
    pub fn play_sound(
        &mut self,
        channel: SoundChannel,
    ) -> Result<agb::sound::mixer::ChannelId, SoundError> {
        self.mixer.play_sound(channel).ok_or(SoundError)
    }

    /// Get a reference to a playing channel
    pub fn channel(
        &mut self,
        id: &agb::sound::mixer::ChannelId,
    ) -> Option<&mut agb::sound::mixer::SoundChannel> {
        self.mixer.channel(id)
    }

    /// Get access to the underlying mixer for synchronous operations
    pub fn mixer(&mut self) -> &mut agb::sound::mixer::Mixer<'a> {
        &mut self.mixer
    }
}

/// Future that completes after processing one audio frame
struct FrameFuture<'a, 'b> {
    mixer: &'a mut agb::sound::mixer::Mixer<'b>,
    processed: bool,
}

impl<'a, 'b> FrameFuture<'a, 'b> {
    fn new(mixer: &'a mut agb::sound::mixer::Mixer<'b>) -> Self {
        Self {
            mixer,
            processed: false,
        }
    }
}

impl<'a, 'b> Future for FrameFuture<'a, 'b> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.processed {
            // Process one frame of audio
            self.mixer.frame();
            self.processed = true;
            Poll::Ready(())
        } else {
            Poll::Ready(())
        }
    }
}

/// Async sound channel for playing audio
pub struct AsyncSoundChannel {
    sound_data: &'static SoundData,
}

impl AsyncSoundChannel {
    /// Create a new async sound channel
    pub fn new(sound_data: &'static SoundData) -> Self {
        Self { sound_data }
    }

    /// Play this sound on the given mixer
    pub async fn play_on(
        &self,
        mixer: &mut AsyncMixer<'_>,
    ) -> Result<agb::sound::mixer::ChannelId, SoundError> {
        let channel = SoundChannel::new(*self.sound_data);
        mixer.play_sound(channel)
    }
}
