use agb::fixnum::num;
use agb::rng;
use agb::sound::mixer::{ChannelId, Mixer, SoundChannel};

static BAT_DEATH: &[u8] = agb::include_wav!("sfx/BatDeath.wav");
static BAT_FLAP: &[u8] = agb::include_wav!("sfx/BatFlap.wav");
static JUMP1: &[u8] = agb::include_wav!("sfx/Jump1.wav");
static JUMP2: &[u8] = agb::include_wav!("sfx/Jump2.wav");
static JUMP3: &[u8] = agb::include_wav!("sfx/Jump3.wav");
static PLAYER_GETS_HIT: &[u8] = agb::include_wav!("sfx/PlayerGetsHit.wav");
static PLAYER_HEAL: &[u8] = agb::include_wav!("sfx/PlayerHeal.wav");
static PLAYER_LANDS: &[u8] = agb::include_wav!("sfx/PlayerLands.wav");
static SLIME_BOING: &[u8] = agb::include_wav!("sfx/SlimeBoing.wav");
static SLIME_DEATH: &[u8] = agb::include_wav!("sfx/SlimeDeath.wav");
static SWORD_SWING: &[u8] = agb::include_wav!("sfx/SwordSwing.wav");
static FLAME_CHARGE: &[u8] = agb::include_wav!("sfx/FlameCharge.wav");
static BOSS_FLAME_MOVE: &[u8] = agb::include_wav!("sfx/FlameMove.wav");
static BURNING_FLAME: &[u8] = agb::include_wav!("sfx/Burning.wav");

static EMU_CRASH: &[u8] = agb::include_wav!("sfx/EmuCrash.wav");
static EMU_STEP: &[u8] = agb::include_wav!("sfx/EmuStep.wav");
static EMU_DEATH: &[u8] = agb::include_wav!("sfx/EmuDeath.wav");

static PURPLE_NIGHT: &[u8] = agb::include_wav!("sfx/01 - The Purple Night (Main Loop).wav");
static SUNRISE: &[u8] = agb::include_wav!("sfx/02 - Sunrise (Main Loop).wav");
static BLUE_SPIRIT: &[u8] = agb::include_wav!("sfx/03 - Blue Spirit (Main Loop).wav");

pub struct Sfx<'a> {
    bgm: Option<ChannelId>,
    mixer: &'a mut Mixer<'a>,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer<'a>) -> Self {
        Self { mixer, bgm: None }
    }

    pub fn frame(&mut self) {
        self.mixer.frame();
    }

    pub fn stop_music(&mut self) {
        if let Some(bgm) = &self.bgm {
            let channel = self.mixer.channel(bgm).unwrap();
            channel.stop();
        }
        self.bgm = None;
    }

    pub fn purple_night(&mut self) {
        if let Some(bgm) = &self.bgm {
            let channel = self.mixer.channel(bgm).unwrap();
            channel.stop();
        }

        let mut channel = SoundChannel::new_high_priority(PURPLE_NIGHT);
        channel.stereo().should_loop();
        self.bgm = self.mixer.play_sound(channel);
    }

    pub fn sunrise(&mut self) {
        if let Some(bgm) = &self.bgm {
            let channel = self.mixer.channel(bgm).unwrap();
            channel.stop();
        }

        let mut channel = SoundChannel::new_high_priority(SUNRISE);
        channel.stereo().should_loop();
        self.bgm = self.mixer.play_sound(channel);
    }

    pub fn boss(&mut self) {
        if let Some(bgm) = &self.bgm {
            let channel = self.mixer.channel(bgm).unwrap();
            channel.stop();
        }

        let mut channel = SoundChannel::new_high_priority(BLUE_SPIRIT);
        channel.stereo().should_loop();
        self.bgm = self.mixer.play_sound(channel);
    }

    pub fn jump(&mut self) {
        let r = rng::gen() % 3;

        let channel = match r {
            0 => SoundChannel::new(JUMP1),
            1 => SoundChannel::new(JUMP2),
            _ => SoundChannel::new(JUMP3),
        };

        self.mixer.play_sound(channel);
    }

    pub fn sword(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SWORD_SWING));
    }

    pub fn slime_boing(&mut self) {
        let mut channel = SoundChannel::new(SLIME_BOING);
        channel.volume(num!(0.25));
        self.mixer.play_sound(channel);
    }

    pub fn slime_dead(&mut self) {
        let channel = SoundChannel::new(SLIME_DEATH);
        self.mixer.play_sound(channel);
    }

    pub fn player_hurt(&mut self) {
        self.mixer.play_sound(SoundChannel::new(PLAYER_GETS_HIT));
    }

    pub fn player_heal(&mut self) {
        self.mixer.play_sound(SoundChannel::new(PLAYER_HEAL));
    }

    pub fn player_land(&mut self) {
        self.mixer.play_sound(SoundChannel::new(PLAYER_LANDS));
    }

    pub fn bat_flap(&mut self) {
        self.mixer.play_sound(SoundChannel::new(BAT_FLAP));
    }

    pub fn bat_death(&mut self) {
        self.mixer.play_sound(SoundChannel::new(BAT_DEATH));
    }

    pub fn flame_charge(&mut self) {
        self.mixer.play_sound(SoundChannel::new(FLAME_CHARGE));
    }

    pub fn boss_move(&mut self) {
        self.mixer.play_sound(SoundChannel::new(BOSS_FLAME_MOVE));
    }

    pub fn burning(&mut self) {
        self.mixer.play_sound(SoundChannel::new(BURNING_FLAME));
    }

    pub fn emu_step(&mut self) {
        self.mixer.play_sound(SoundChannel::new(EMU_STEP));
    }

    pub fn emu_crash(&mut self) {
        self.mixer.play_sound(SoundChannel::new(EMU_CRASH));
    }

    pub fn emu_death(&mut self) {
        self.mixer.play_sound(SoundChannel::new(EMU_DEATH));
    }
}
