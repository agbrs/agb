use agb::{
    fixnum::num,
    include_wav,
    sound::mixer::{Mixer, SoundChannel},
};
use agb_tracker::{include_xm, Track, Tracker};

static MUSIC: Track = include_xm!("sfx/gwilym-theme2.xm");

static BAD_SELECTION: &[u8] = include_wav!("sfx/bad.wav");
static SELECT: &[u8] = include_wav!("sfx/select.wav");
static PLACE: &[u8] = include_wav!("sfx/place.wav");

static SLIME_DEATH: &[u8] = include_wav!("sfx/slime_death.wav");
static SWORD_PICKUP: &[u8] = include_wav!("sfx/sword_pickup.wav");
static WALL_HIT: &[u8] = include_wav!("sfx/wall_hit.wav");
static DOOR_OPEN: &[u8] = include_wav!("sfx/door_open.wav");

static SWICTH_TOGGLES: &[&[u8]] = &[include_wav!("sfx/switch_toggle1.wav")];

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
        self.play_effect(BAD_SELECTION);
    }

    pub fn select(&mut self) {
        self.play_effect(SELECT);
    }

    pub fn place(&mut self) {
        self.play_effect(PLACE);
    }

    pub fn play_sound_effect(&mut self, effect: Option<SoundEffect>) {
        if let Some(effect) = effect {
            match effect {
                SoundEffect::WallHit => {
                    self.play_effect(WALL_HIT);
                }
                SoundEffect::SlimeDie => {
                    self.play_effect(SLIME_DEATH);
                }
                SoundEffect::HeroDie => {}
                SoundEffect::SquidDie => {}
                SoundEffect::SwordPickup => {
                    self.play_effect(SWORD_PICKUP);
                }
                SoundEffect::SwordKill => {}
                SoundEffect::KeyPickup => {}
                SoundEffect::DoorOpen => {
                    self.play_effect(DOOR_OPEN);
                }
                SoundEffect::SwitchToggle => {
                    self.play_effect(SWICTH_TOGGLES[0]);
                }
                SoundEffect::KeyDrop => {}
                SoundEffect::SwordDrop => {}
                SoundEffect::SwitchedDoorToggle => {}
                SoundEffect::SpikesToggle => {}
                SoundEffect::TeleportEffect => {}
            }
        }
    }

    fn play_effect(&mut self, effect: &'static [u8]) {
        let mut channel = SoundChannel::new(effect);
        channel.playback(num!(0.5));
        self.mixer.play_sound(channel);
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
    TeleportEffect,
}
