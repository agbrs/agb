use agb_fixnum::Num;
use std::{borrow::Cow, num::Wrapping};

const BUFFER_SIZE: usize = 560;
const NUM_CHANNELS: usize = 8;

#[derive(Default)]
pub struct Mixer {
    channels: [Option<SoundChannel>; NUM_CHANNELS],
    indices: [Wrapping<usize>; NUM_CHANNELS],
}

impl Mixer {
    pub fn new() -> Self {
        Self {
            channels: Default::default(),
            indices: Default::default(),
        }
    }

    pub fn frame(&mut self) -> Vec<(i8, i8)> {
        let channels =
            self.channels.iter_mut().flatten().filter(|channel| {
                !channel.is_done && channel.volume != 0.into() && channel.is_playing
            });

        let mut buffer = vec![Num::new(0); BUFFER_SIZE * 2];

        for channel in channels {
            let right_amount = ((channel.panning + 1) / 2) * channel.volume;
            let left_amount = ((-channel.panning + 1) / 2) * channel.volume;

            let right_amount: Num<i16, 4> = right_amount.change_base();
            let left_amount: Num<i16, 4> = left_amount.change_base();

            let channel_len = Num::<u32, 8>::new(channel.data.len() as u32);
            let mut playback_speed = channel.playback_speed;

            while playback_speed >= channel_len - channel.restart_point {
                playback_speed -= channel_len;
            }

            let restart_subtract = channel_len - channel.restart_point;

            let mut current_pos = channel.pos;

            for i in 0..BUFFER_SIZE {
                let val = channel.data[current_pos.floor() as usize] as i8 as i16;

                buffer[2 * i] += left_amount * val;
                buffer[2 * i + 1] += right_amount * val;

                current_pos += playback_speed;

                if current_pos >= channel_len {
                    if channel.should_loop {
                        current_pos -= restart_subtract;
                    } else {
                        channel.is_done = true;
                        break;
                    }
                }
            }

            channel.pos = current_pos;
        }

        let mut ret = Vec::with_capacity(BUFFER_SIZE);
        for i in 0..BUFFER_SIZE {
            let l = buffer[2 * i].floor();
            let r = buffer[2 * i + 1].floor();

            ret.push((
                l.clamp(i8::MIN as i16, i8::MAX as i16) as i8,
                r.clamp(i8::MIN as i16, i8::MAX as i16) as i8,
            ));
        }

        ret
    }
}

pub struct SoundChannel {
    data: Cow<'static, [u8]>,
    pos: Num<u32, 8>,
    should_loop: bool,
    restart_point: Num<u32, 8>,

    is_playing: bool,
    playback_speed: Num<u32, 8>,
    volume: Num<i16, 8>,

    panning: Num<i16, 8>, // between -1 and 1
    is_done: bool,
}

impl std::fmt::Debug for SoundChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SoundChannel")
            .field("pos", &self.pos)
            .field("should_loop", &self.should_loop)
            .field("restart_point", &self.restart_point)
            .field("is_playing", &self.is_playing)
            .field("playback_speed", &self.playback_speed)
            .field("volume", &self.volume)
            .field("panning", &self.panning)
            .field("is_done", &self.is_done)
            .finish()
    }
}

impl SoundChannel {
    fn new(data: Cow<'static, [u8]>) -> Self {
        Self {
            data: data.clone(),

            pos: 0.into(),
            should_loop: false,
            playback_speed: 1.into(),
            is_playing: true,
            panning: 0.into(),
            is_done: false,
            volume: 1.into(),
            restart_point: 0.into(),
        }
    }
}

pub struct SoundChannelId(usize, Wrapping<usize>);

impl agb_tracker::SoundChannel for SoundChannel {
    fn new(data: &Cow<'static, [u8]>) -> Self {
        Self::new(data.clone())
    }

    fn stop(&mut self) {
        self.is_done = true;
    }

    fn pause(&mut self) -> &mut Self {
        self.is_playing = false;
        self
    }

    fn resume(&mut self) -> &mut Self {
        self.is_playing = true;
        self
    }

    fn should_loop(&mut self) -> &mut Self {
        self.should_loop = true;
        self
    }

    fn volume(&mut self, value: impl Into<Num<i16, 8>>) -> &mut Self {
        self.volume = value.into();
        self
    }

    fn restart_point(&mut self, value: impl Into<Num<u32, 8>>) -> &mut Self {
        self.restart_point = value.into();
        self
    }

    fn playback(&mut self, playback_speed: impl Into<Num<u32, 8>>) -> &mut Self {
        self.playback_speed = playback_speed.into();
        self
    }

    fn panning(&mut self, panning: impl Into<Num<i16, 8>>) -> &mut Self {
        self.panning = panning.into();
        self
    }

    fn set_pos(&mut self, pos: impl Into<Num<u32, 8>>) -> &mut Self {
        self.pos = pos.into();
        self
    }
}

impl agb_tracker::Mixer for Mixer {
    type ChannelId = SoundChannelId;

    type SoundChannel = SoundChannel;

    fn channel(&mut self, channel_id: &Self::ChannelId) -> Option<&mut Self::SoundChannel> {
        if let Some(channel) = &mut self.channels[channel_id.0]
            && self.indices[channel_id.0] == channel_id.1
            && !channel.is_done
        {
            return Some(channel);
        }

        None
    }

    fn play_sound(&mut self, new_channel: Self::SoundChannel) -> Option<Self::ChannelId> {
        for (i, channel) in self.channels.iter_mut().enumerate() {
            if let Some(channel) = channel
                && !channel.is_done
            {
                continue;
            }

            channel.replace(new_channel);
            self.indices[i] += 1;
            return Some(SoundChannelId(i, self.indices[i]));
        }

        None
    }
}
