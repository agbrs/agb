use agb::{
    include_wav,
    sound::mixer::{Mixer, SoundChannel},
};
use agb_tracker::{include_xm, Track, Tracker};

const MUSIC: Track = include_xm!("sfx/gwilym-theme2.xm");

const BAD_SELECTION: &[u8] = include_wav!("sfx/bad.wav");
const SELECT: &[u8] = include_wav!("sfx/select.wav");
const PLACE: &[u8] = include_wav!("sfx/place.wav");

const SLIME_DEATH: &[u8] = include_wav!("sfx/slime_death.wav");
const SWORD_PICKUP: &[u8] = include_wav!("sfx/sword_pickup.wav");
const WALL_HIT: &[u8] = include_wav!("sfx/wall_hit.wav");
const DOOR_OPEN: &[u8] = include_wav!("sfx/door_open.wav");

const SWICTH_TOGGLES: &[&[u8]] = &[include_wav!("sfx/switch_toggle1.wav")];

pub struct Sfx<'a> {
    mixer: &'a mut Mixer<'a>,
    tracker: Tracker,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer<'a>) -> Self {
        mixer.enable();

        let tracker = Tracker::new(&MUSIC);

        Self { mixer, tracker }
    }

    pub fn frame(&mut self) {
        self.tracker.step(self.mixer);
        self.mixer.frame();
    }

    pub fn bad_selection(&mut self) {
        self.mixer.play_sound(SoundChannel::new(BAD_SELECTION));
    }

    pub fn select(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SELECT));
    }

    pub fn place(&mut self) {
        self.mixer.play_sound(SoundChannel::new(PLACE));
    }

    pub fn play_sound_effect(&mut self, effect: Option<SoundEffect>) {
        if let Some(effect) = effect {
            match effect {
                SoundEffect::WallHit => {
                    self.mixer.play_sound(SoundChannel::new(WALL_HIT));
                }
                SoundEffect::SlimeDie => {
                    self.mixer.play_sound(SoundChannel::new(SLIME_DEATH));
                }
                SoundEffect::HeroDie => {}
                SoundEffect::SquidDie => {}
                SoundEffect::SwordPickup => {
                    self.mixer.play_sound(SoundChannel::new(SWORD_PICKUP));
                }
                SoundEffect::SwordKill => {}
                SoundEffect::KeyPickup => {}
                SoundEffect::DoorOpen => {
                    self.mixer.play_sound(SoundChannel::new(DOOR_OPEN));
                }
                SoundEffect::SwitchToggle => {
                    self.mixer.play_sound(SoundChannel::new(SWICTH_TOGGLES[0]));
                }
                SoundEffect::KeyDrop => {}
                SoundEffect::SwordDrop => {}
                SoundEffect::SwitchedDoorToggle => {}
                SoundEffect::SpikesToggle => {}
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SoundEffect {
    SlimeDie,
    HeroDie,
    SquidDie,
    SwordPickup,
    SwordKill,
    KeyPickup,
    DoorOpen,
    SwitchToggle,
    KeyDrop,
    SwordDrop,
    SwitchedDoorToggle,
    SpikesToggle,
    WallHit,
}
