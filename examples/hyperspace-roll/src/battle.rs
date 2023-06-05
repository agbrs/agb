use crate::level_generation::generate_enemy_health;
use crate::sfx::Sfx;
use crate::{
    graphics::SELECT_BOX, level_generation::generate_attack, Agb, EnemyAttackType, Face, PlayerDice,
};
use agb::display::tiled::{RegularMap, TiledMap};
use agb::{hash_map::HashMap, input::Button};
use alloc::vec;
use alloc::vec::Vec;

use self::display::BattleScreenDisplay;

mod display;

pub(super) const MALFUNCTION_COOLDOWN_FRAMES: u32 = 3 * 60;
const ROLL_TIME_FRAMES_ALL: u32 = 2 * 60;
const ROLL_TIME_FRAMES_ONE: u32 = 60 / 8;

/// A face of the rolled die and it's cooldown (should it be a malfunction)
#[derive(Debug)]
struct RolledDie {
    face: Face,
    cooldown: u32,
}

impl RolledDie {
    fn new(face: Face) -> Self {
        let cooldown = if face == Face::Malfunction {
            MALFUNCTION_COOLDOWN_FRAMES
        } else {
            0
        };

        Self { face, cooldown }
    }

    fn update(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }

    fn can_reroll(&self) -> bool {
        self.face != Face::Malfunction || self.cooldown == 0
    }

    fn can_reroll_after_accept(&self) -> bool {
        self.face != Face::Malfunction
    }

    fn cooldown(&self) -> Option<u32> {
        if self.face == Face::Malfunction && self.cooldown > 0 {
            Some(self.cooldown)
        } else {
            None
        }
    }
}

#[derive(Debug)]
enum DieState {
    Rolling(u32, Face, Face),
    Rolled(RolledDie),
}

#[derive(Debug, Clone)]
pub enum Action {
    PlayerActivateShield { amount: u32 },
    PlayerShoot { damage: u32, piercing: u32 },
    PlayerDisrupt { amount: u32 },
    PlayerHeal { amount: u32 },
    PlayerBurstShield { multiplier: u32 },
    PlayerSendBurstShield { damage: u32 },
    EnemyShoot { damage: u32 },
    EnemyShield { amount: u32 },
    EnemyHeal { amount: u32 },
}

#[derive(Debug)]
struct RolledDice {
    rolls: Vec<DieState>,
}

impl RolledDice {
    fn update(&mut self, player_dice: &PlayerDice) {
        self.rolls
            .iter_mut()
            .zip(player_dice.dice.iter())
            .for_each(|(die_state, player_die)| match die_state {
                DieState::Rolling(ref mut timeout, ref mut face, previous_face) => {
                    if *timeout == 0 {
                        let mut number_of_rolls = 0;
                        *die_state = DieState::Rolled(RolledDie::new(loop {
                            let next_face = player_die.roll();
                            number_of_rolls += 1;
                            if *previous_face != Face::Malfunction
                                || next_face != *previous_face
                                || number_of_rolls > 16
                            {
                                break next_face;
                            }
                        }));
                    } else {
                        if *timeout % 2 == 0 {
                            *face = player_die.roll();
                        }
                        *timeout -= 1;
                    }
                }
                DieState::Rolled(ref mut rolled_die) => rolled_die.update(),
            });
    }

    fn faces_for_accepting(&self) -> impl Iterator<Item = Face> + '_ {
        self.rolls.iter().filter_map(|state| match state {
            DieState::Rolled(rolled_die) => Some(rolled_die.face),
            _ => None,
        })
    }

    fn faces_to_render(&self) -> impl Iterator<Item = (Face, Option<u32>)> + '_ {
        self.rolls.iter().map(|rolled_die| match rolled_die {
            DieState::Rolling(_, face, _previous_face) => (*face, None),
            DieState::Rolled(rolled_die) => (rolled_die.face, rolled_die.cooldown()),
        })
    }

    fn accept_rolls(&mut self, player_dice: &PlayerDice) -> Vec<Action> {
        let mut actions = vec![];

        let mut face_counts: HashMap<Face, u32> = HashMap::new();
        let mut shield_multiplier = 1;
        let mut shoot_multiplier = 1;
        for face in self.faces_for_accepting() {
            match face {
                Face::DoubleShot => *face_counts.entry(Face::Shoot).or_default() += 2,
                Face::TripleShot => *face_counts.entry(Face::Shoot).or_default() += 3,
                Face::DoubleShield => *face_counts.entry(Face::Shield).or_default() += 2,
                Face::TripleShield => *face_counts.entry(Face::Shield).or_default() += 3,
                Face::DoubleShieldValue => shield_multiplier *= 2,
                Face::DoubleShotValue => shoot_multiplier *= 2,
                Face::TripleShotValue => shoot_multiplier *= 3,
                other => *face_counts.entry(other).or_default() += 1,
            }
        }

        let invert = *face_counts.entry(Face::Invert).or_default() % 2 == 1;

        // shield
        let mut shield_amount = *face_counts.entry(Face::Shield).or_default() * shield_multiplier;

        // shooting
        let shoot = *face_counts.entry(Face::Shoot).or_default();
        let shoot_power = (shoot * (shoot + 1)) / 2;

        let malfunction_shots = *face_counts.entry(Face::MalfunctionShot).or_default();
        let malfunctions = *face_counts.entry(Face::Malfunction).or_default();

        let malfunction_shoot = (malfunction_shots * (malfunction_shots + 1)) / 2
            * (malfunctions * (malfunctions + 1))
            / 2;

        if malfunction_shoot != 0 {
            for roll in self.rolls.iter_mut().filter_map(|face| match face {
                DieState::Rolled(rolled_die) if rolled_die.face == Face::Malfunction => {
                    Some(rolled_die)
                }
                _ => None,
            }) {
                roll.face = Face::Blank;
            }
        }

        let mut shoot_power = (shoot_power + malfunction_shoot) * shoot_multiplier;

        if invert {
            (shoot_power, shield_amount) = (shield_amount, shoot_power);
        }

        if shoot_power > 0 {
            actions.push(Action::PlayerShoot {
                damage: shoot_power,
                piercing: *face_counts.entry(Face::Bypass).or_default(),
            });
        }

        if shield_amount > 0 {
            actions.push(Action::PlayerActivateShield {
                amount: shield_amount.min(5),
            });
        }

        // burst shield
        if face_counts.contains_key(&Face::BurstShield) {
            actions.push(Action::PlayerBurstShield {
                multiplier: shoot_multiplier,
            });
        }

        // disrupt
        let disrupt = *face_counts.entry(Face::Disrupt).or_default();
        let disrupt_power = (disrupt * (disrupt + 1)) / 2;

        if disrupt_power > 0 {
            actions.push(Action::PlayerDisrupt {
                amount: disrupt_power,
            });
        }

        let heal = *face_counts.entry(Face::Heal).or_default();
        if heal != 0 {
            actions.push(Action::PlayerHeal {
                amount: (heal * (heal + 1)) / 2,
            });
        }

        let mut malfunction_all = false;

        for roll in self.rolls.iter_mut().filter_map(|face| match face {
            DieState::Rolled(rolled_die) => Some(rolled_die),
            _ => None,
        }) {
            if roll.face == Face::DoubleShot
                || roll.face == Face::DoubleShield
                || roll.face == Face::DoubleShotValue
            {
                roll.cooldown = MALFUNCTION_COOLDOWN_FRAMES;
                roll.face = Face::Malfunction;
            }
            if roll.face == Face::TripleShot
                || roll.face == Face::TripleShield
                || roll.face == Face::TripleShotValue
                || roll.face == Face::BurstShield
            {
                malfunction_all = true;
            }
        }

        if malfunction_all {
            for roll in self.rolls.iter_mut().filter_map(|face| match face {
                DieState::Rolled(rolled_die) => Some(rolled_die),
                _ => None,
            }) {
                roll.cooldown = MALFUNCTION_COOLDOWN_FRAMES;
                roll.face = Face::Malfunction;
            }
        }

        // reroll non-malfunctions after accepting
        for i in 0..player_dice.dice.len() {
            self.roll_die(i, ROLL_TIME_FRAMES_ALL, true, player_dice);
        }

        actions
    }

    fn roll_die(
        &mut self,
        die_index: usize,
        time: u32,
        is_after_accept: bool,
        player_dice: &PlayerDice,
    ) {
        if let DieState::Rolled(ref selected_rolled_die) = self.rolls[die_index] {
            let can_reroll = if is_after_accept {
                selected_rolled_die.can_reroll_after_accept()
            } else {
                selected_rolled_die.can_reroll()
            };

            if can_reroll {
                self.rolls[die_index] = DieState::Rolling(
                    time,
                    player_dice.dice[die_index].roll(),
                    selected_rolled_die.face,
                );
            }
        }
    }
}

#[derive(Debug)]
struct PlayerState {
    shield_count: u32,
    health: u32,
    max_health: u32,
}

#[derive(Debug)]
pub enum EnemyAttack {
    Shoot(u32),
    Shield(u32),
    Heal(u32),
}

impl EnemyAttack {
    fn apply_effect(&self) -> Action {
        match self {
            EnemyAttack::Shoot(damage) => Action::EnemyShoot { damage: *damage },
            EnemyAttack::Shield(shield) => Action::EnemyShield { amount: *shield },
            EnemyAttack::Heal(amount) => Action::EnemyHeal { amount: *amount },
        }
    }
}

#[derive(Debug)]
struct EnemyAttackState {
    attack: EnemyAttack,
    cooldown: u32,
    max_cooldown: u32,
}

impl EnemyAttackState {
    fn attack_type(&self) -> EnemyAttackType {
        match self.attack {
            EnemyAttack::Shoot(_) => EnemyAttackType::Attack,
            EnemyAttack::Shield(_) => EnemyAttackType::Shield,
            EnemyAttack::Heal(_) => EnemyAttackType::Heal,
        }
    }

    fn value_to_show(&self) -> Option<u32> {
        match self.attack {
            EnemyAttack::Shoot(i) => Some(i),
            EnemyAttack::Heal(i) => Some(i),
            EnemyAttack::Shield(i) => Some(i),
        }
    }

    #[must_use]
    fn update(&mut self) -> Option<Action> {
        if self.cooldown == 0 {
            return Some(self.attack.apply_effect());
        }

        self.cooldown -= 1;

        None
    }
}

#[derive(Debug)]
struct EnemyState {
    shield_count: u32,
    health: u32,
    max_health: u32,
}

#[derive(Debug)]
pub struct CurrentBattleState {
    player: PlayerState,
    enemy: EnemyState,
    rolled_dice: RolledDice,
    player_dice: PlayerDice,
    attacks: [Option<EnemyAttackState>; 2],
    current_level: u32,
}

impl CurrentBattleState {
    fn accept_rolls(&mut self) -> Vec<Action> {
        self.rolled_dice.accept_rolls(&self.player_dice)
    }

    fn roll_die(&mut self, die_index: usize, time: u32, is_after_accept: bool) {
        self.rolled_dice
            .roll_die(die_index, time, is_after_accept, &self.player_dice);
    }

    fn update(&mut self) -> Vec<Action> {
        let mut actions = vec![];

        for attack in self.attacks.iter_mut() {
            if let Some(attack_state) = attack {
                if let Some(action) = attack_state.update() {
                    attack.take();
                    actions.push(action);
                }
            } else if let Some(generated_attack) = generate_attack(self.current_level) {
                attack.replace(EnemyAttackState {
                    attack: generated_attack.attack,
                    cooldown: generated_attack.cooldown,
                    max_cooldown: generated_attack.cooldown,
                });
            }
        }

        actions
    }

    fn update_dice(&mut self) {
        self.rolled_dice.update(&self.player_dice);
    }

    fn apply_action(&mut self, action: Action, sfx: &mut Sfx) -> Option<Action> {
        match action {
            Action::PlayerActivateShield { amount } => {
                if amount > self.player.shield_count {
                    sfx.shield_up();
                }

                self.player.shield_count = self.player.shield_count.max(amount);
                None
            }
            Action::PlayerShoot { damage, piercing } => {
                if self.enemy.shield_count <= piercing {
                    self.enemy.health = self.enemy.health.saturating_sub(damage);
                    sfx.shot_hit();
                } else if self.enemy.shield_count <= damage {
                    self.enemy.shield_count = 0; // TODO: Dispatch action of drop shield to animate that
                    sfx.shield_down();
                } else {
                    sfx.shield_defend();
                }

                None
            }
            Action::PlayerDisrupt { amount } => {
                for attack in self.attacks.iter_mut().flatten() {
                    attack.cooldown += amount * 240;
                    attack.max_cooldown = attack.cooldown.max(attack.max_cooldown);
                }

                sfx.disrupt();

                None
            }
            Action::PlayerHeal { amount } => {
                self.player.health = self.player.max_health.min(self.player.health + amount);
                sfx.heal();
                None
            }
            Action::EnemyShoot { damage } => {
                if self.player.shield_count == 0 {
                    self.player.health = self.player.health.saturating_sub(damage);
                    sfx.shot_hit();
                } else if self.player.shield_count <= damage {
                    self.player.shield_count = 0; // TODO: Dispatch action of drop shield to animate that
                    sfx.shield_down();
                } else {
                    sfx.shield_defend();
                }

                None
            }
            Action::EnemyShield { amount } => {
                if amount > self.enemy.shield_count {
                    sfx.shield_up();
                }

                self.enemy.shield_count = self.enemy.shield_count.max(amount);
                None
            }
            Action::EnemyHeal { amount } => {
                self.enemy.health = self.enemy.max_health.min(self.enemy.health + amount);
                sfx.heal();
                None
            }
            Action::PlayerBurstShield { multiplier } => {
                let damage =
                    self.player.shield_count * (self.player.shield_count + 1) * multiplier / 2;
                self.player.shield_count = 0;
                sfx.send_burst_shield();

                Some(Action::PlayerSendBurstShield { damage })
            }
            Action::PlayerSendBurstShield { damage } => {
                self.enemy.shield_count = 0;
                self.enemy.health = self.enemy.health.saturating_sub(damage);

                sfx.burst_shield_hit();
                sfx.shield_down();

                None
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum BattleResult {
    Win,
    Loss,
}

pub(crate) fn battle_screen(
    agb: &mut Agb,
    player_dice: PlayerDice,
    current_level: u32,
    help_background: &mut RegularMap,
) -> BattleResult {
    agb.sfx.battle();
    agb.sfx.frame();

    help_background.set_scroll_pos((-16i16, -97i16).into());
    crate::background::load_help_text(&mut agb.vram, help_background, 1, (0, 0));
    crate::background::load_help_text(&mut agb.vram, help_background, 2, (0, 1));

    let obj = &agb.obj;

    let mut select_box_obj = agb.obj.object_sprite(SELECT_BOX.sprite(0));
    select_box_obj.show();

    let num_dice = player_dice.dice.len();

    let enemy_health = generate_enemy_health(current_level);

    let mut current_battle_state = CurrentBattleState {
        player: PlayerState {
            shield_count: 0,
            health: 20,
            max_health: 20,
        },
        enemy: EnemyState {
            shield_count: 0,
            health: enemy_health,
            max_health: enemy_health,
        },
        rolled_dice: RolledDice {
            rolls: player_dice
                .dice
                .iter()
                .map(|die| DieState::Rolling(ROLL_TIME_FRAMES_ALL, die.roll(), Face::Blank))
                .collect(),
        },
        player_dice: player_dice.clone(),
        attacks: [None, None],
        current_level,
    };

    let mut battle_screen_display = BattleScreenDisplay::new(obj, &current_battle_state);
    agb.sfx.frame();

    let mut selected_die = 0usize;
    let mut input = agb::input::ButtonController::new();
    let mut counter = 0usize;

    loop {
        counter = counter.wrapping_add(1);

        for action_to_apply in battle_screen_display.update(obj, &current_battle_state) {
            if let Some(action_to_return) =
                current_battle_state.apply_action(action_to_apply, &mut agb.sfx)
            {
                battle_screen_display.add_action(action_to_return, obj, &mut agb.sfx);
            }
        }

        for action in current_battle_state.update() {
            battle_screen_display.add_action(action, obj, &mut agb.sfx);
        }

        current_battle_state.update_dice();

        input.update();

        if input.is_just_pressed(Button::LEFT) {
            if selected_die == 0 {
                selected_die = num_dice - 1;
            } else {
                selected_die -= 1;
            }

            agb.sfx.move_cursor();
        }

        if input.is_just_pressed(Button::RIGHT) {
            if selected_die == num_dice - 1 {
                selected_die = 0;
            } else {
                selected_die += 1;
            }

            agb.sfx.move_cursor();
        }

        if input.is_just_pressed(Button::A) {
            current_battle_state.roll_die(selected_die, ROLL_TIME_FRAMES_ONE, false);
            agb.sfx.roll();
        }

        if input.is_just_pressed(Button::START) {
            for action in current_battle_state.accept_rolls() {
                battle_screen_display.add_action(action, obj, &mut agb.sfx);
            }
            agb.sfx.roll_multi();
        }

        select_box_obj
            .set_y(120 - 4)
            .set_x(selected_die as u16 * 40 + 28 - 4)
            .set_sprite(agb.obj.sprite(SELECT_BOX.animation_sprite(counter / 10)));

        agb.star_background.update();
        agb.sfx.frame();
        agb.vblank.wait_for_vblank();
        help_background.commit(&mut agb.vram);
        help_background.show();

        if current_battle_state.enemy.health == 0 {
            agb.sfx.ship_explode();
            help_background.hide();
            crate::background::load_help_text(&mut agb.vram, help_background, 3, (0, 0));
            crate::background::load_help_text(&mut agb.vram, help_background, 3, (0, 1));
            return BattleResult::Win;
        }

        if current_battle_state.player.health == 0 {
            agb.sfx.ship_explode();
            help_background.hide();
            crate::background::load_help_text(&mut agb.vram, help_background, 3, (0, 0));
            crate::background::load_help_text(&mut agb.vram, help_background, 3, (0, 1));
            return BattleResult::Loss;
        }

        agb.obj.commit();
        agb.star_background.commit(&mut agb.vram);
    }
}
