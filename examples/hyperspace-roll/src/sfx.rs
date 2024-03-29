use agb::fixnum::num;
use agb::sound::mixer::{ChannelId, Mixer, SoundChannel};
use agb::{include_wav, rng};

static DICE_ROLLS: &[&[u8]] = &[
    include_wav!("sfx/SingleRoll_1.wav"),
    include_wav!("sfx/SingleRoll_2.wav"),
    include_wav!("sfx/SingleRoll_3.wav"),
    include_wav!("sfx/SingleRoll_4.wav"),
    include_wav!("sfx/SingleRoll_5.wav"),
];

static MULTI_ROLLS: &[&[u8]] = &[
    include_wav!("sfx/MultiRoll_1.wav"),
    include_wav!("sfx/MultiRoll_2.wav"),
    include_wav!("sfx/MultiRoll_3.wav"),
    include_wav!("sfx/MultiRoll_4.wav"),
    include_wav!("sfx/MultiRoll_5.wav"),
];

static MENU_BGM: &[u8] = include_wav!("sfx/BGM_Menu.wav");
static BATTLE_BGM: &[u8] = include_wav!("sfx/BGM_Fight.wav");
static TITLE_BGM: &[u8] = include_wav!("sfx/BGM_Title.wav");

static SHOOT: &[u8] = include_wav!("sfx/shoot.wav");
static SHOT_HIT: &[u8] = include_wav!("sfx/shot_hit.wav");
static SHIP_EXPLODE: &[u8] = include_wav!("sfx/ship_explode.wav");
static MOVE_CURSOR: &[u8] = include_wav!("sfx/move_cursor.wav");
static SELECT: &[u8] = include_wav!("sfx/select.wav");
static BACK: &[u8] = include_wav!("sfx/back.wav");
static ACCEPT: &[u8] = include_wav!("sfx/accept.wav");
static SHIELD_DOWN: &[u8] = include_wav!("sfx/shield_down.wav");
static SHIELD_UP: &[u8] = include_wav!("sfx/shield_up.wav");
static SHIELD_DEFEND: &[u8] = include_wav!("sfx/shield_defend.wav");
static DISRUPT: &[u8] = include_wav!("sfx/disrupt.wav");
static HEAL: &[u8] = include_wav!("sfx/heal.wav");
static SEND_BURST_SHIELD: &[u8] = include_wav!("sfx/send_burst_shield.wav");
static BURST_SHIELD_HIT: &[u8] = include_wav!("sfx/burst_shield_hit.wav");

#[derive(Clone, Copy, PartialEq, Eq)]
enum BattleOrMenu {
    Battle,
    Menu,
    Title,
}

pub struct Sfx<'a> {
    mixer: &'a mut Mixer<'a>,
    state: BattleOrMenu,

    current_bgm: ChannelId,
}

impl<'a> Sfx<'a> {
    pub fn new(mixer: &'a mut Mixer<'a>) -> Self {
        let mut title_music = SoundChannel::new_high_priority(TITLE_BGM);
        title_music.should_loop();
        let title_channel = mixer.play_sound(title_music).unwrap();

        Self {
            mixer,
            state: BattleOrMenu::Title,

            current_bgm: title_channel,
        }
    }

    pub fn frame(&mut self) {
        self.mixer.frame();
    }

    pub fn battle(&mut self) {
        if self.state == BattleOrMenu::Battle {
            return;
        }

        self.state = BattleOrMenu::Battle;

        let current_channel = self.mixer.channel(&self.current_bgm).unwrap();
        let pos = current_channel.pos();
        current_channel.stop();

        let mut battle_music = SoundChannel::new_high_priority(BATTLE_BGM);
        battle_music.should_loop().set_pos(pos);
        self.current_bgm = self.mixer.play_sound(battle_music).unwrap();
    }

    pub fn customise(&mut self) {
        if self.state == BattleOrMenu::Menu {
            return;
        }

        let should_restart = self.state == BattleOrMenu::Title;

        self.state = BattleOrMenu::Menu;
        let current_channel = self.mixer.channel(&self.current_bgm).unwrap();
        let pos = current_channel.pos();
        current_channel.stop();

        let mut menu_music = SoundChannel::new_high_priority(MENU_BGM);
        menu_music
            .should_loop()
            .set_pos(if should_restart { 0.into() } else { pos });
        self.current_bgm = self.mixer.play_sound(menu_music).unwrap();
    }

    pub fn title_screen(&mut self) {
        if self.state == BattleOrMenu::Title {
            return;
        }

        self.state = BattleOrMenu::Title;
        self.mixer.channel(&self.current_bgm).unwrap().stop();

        let mut title_music = SoundChannel::new_high_priority(TITLE_BGM);
        title_music.should_loop();
        self.current_bgm = self.mixer.play_sound(title_music).unwrap();
    }

    pub fn roll(&mut self) {
        let roll_sound_to_use = rng::gen().rem_euclid(DICE_ROLLS.len() as i32);
        let sound_channel = SoundChannel::new(DICE_ROLLS[roll_sound_to_use as usize]);

        self.mixer.play_sound(sound_channel);
    }

    pub fn roll_multi(&mut self) {
        let roll_sound_to_use = rng::gen().rem_euclid(MULTI_ROLLS.len() as i32);
        let sound_channel = SoundChannel::new(MULTI_ROLLS[roll_sound_to_use as usize]);

        self.mixer.play_sound(sound_channel);
    }

    pub fn shoot(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHOOT));
    }

    pub fn shot_hit(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHOT_HIT));
    }

    pub fn ship_explode(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHIP_EXPLODE));
    }

    pub fn move_cursor(&mut self) {
        let mut channel = SoundChannel::new(MOVE_CURSOR);
        channel.volume(num!(0.5));

        self.mixer.play_sound(channel);
    }

    pub fn select(&mut self) {
        let mut channel = SoundChannel::new(SELECT);
        channel.volume(num!(0.75));

        self.mixer.play_sound(channel);
    }

    pub fn back(&mut self) {
        let mut channel = SoundChannel::new(BACK);
        channel.volume(num!(0.5));

        self.mixer.play_sound(channel);
    }

    pub fn accept(&mut self) {
        let mut channel = SoundChannel::new(ACCEPT);
        channel.volume(num!(0.5));

        self.mixer.play_sound(channel);
    }

    pub fn shield_down(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHIELD_DOWN));
    }

    pub fn shield_up(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SHIELD_UP));
    }

    pub fn shield_defend(&mut self) {
        let mut channel = SoundChannel::new(SHIELD_DEFEND);
        channel.volume(num!(0.5));
        self.mixer.play_sound(channel);
    }

    pub fn disrupt(&mut self) {
        self.mixer.play_sound(SoundChannel::new(DISRUPT));
    }

    pub fn heal(&mut self) {
        self.mixer.play_sound(SoundChannel::new(HEAL));
    }

    pub fn send_burst_shield(&mut self) {
        self.mixer.play_sound(SoundChannel::new(SEND_BURST_SHIELD));
    }

    pub fn burst_shield_hit(&mut self) {
        self.mixer.play_sound(SoundChannel::new(BURST_SHIELD_HIT));
    }
}
