use agb::display::object::{OamManaged, Object};
use agb::fixnum::Vector2D;
use agb::rng;
use alloc::vec;
use alloc::vec::Vec;

use crate::graphics::{BURST_BULLET, DISRUPT_BULLET, SHIELD};
use crate::sfx::Sfx;
use crate::{
    graphics::{
        FractionDisplay, HealthBar, NumberDisplay, BULLET_SPRITE, ENEMY_ATTACK_SPRITES,
        FACE_SPRITES, SHIP_SPRITES,
    },
    EnemyAttackType, Ship,
};

use super::{Action, CurrentBattleState, EnemyAttackState, MALFUNCTION_COOLDOWN_FRAMES};

struct BattleScreenDisplayObjects<'a> {
    dice: Vec<Object<'a>>,
    dice_cooldowns: Vec<HealthBar<'a>>,
    player_shield: Vec<Object<'a>>,
    enemy_shield: Vec<Object<'a>>,

    player_healthbar: HealthBar<'a>,
    enemy_healthbar: HealthBar<'a>,
    player_health: FractionDisplay<'a>,
    enemy_health: FractionDisplay<'a>,

    enemy_attack_display: Vec<EnemyAttackDisplay<'a>>,
}

pub struct BattleScreenDisplay<'a> {
    objs: BattleScreenDisplayObjects<'a>,
    animations: Vec<AnimationStateHolder<'a>>,

    _misc_sprites: Vec<Object<'a>>,
}

const HEALTH_BAR_WIDTH: usize = 48;

impl<'a> BattleScreenDisplay<'a> {
    pub fn new(obj: &'a OamManaged, current_battle_state: &CurrentBattleState) -> Self {
        let mut misc_sprites = vec![];
        let player_x = 12;
        let player_y = 8;
        let enemy_x = 167;

        let player_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Player);
        let enemy_sprite = SHIP_SPRITES.sprite_for_ship(if rng::gen() % 2 == 0 {
            Ship::Drone
        } else {
            Ship::PilotedShip
        });

        let mut player_obj = obj.object_sprite(player_sprite);
        let mut enemy_obj = obj.object_sprite(enemy_sprite);

        player_obj.set_x(player_x).set_y(player_y).set_z(1).show();
        enemy_obj.set_x(enemy_x).set_y(player_y).set_z(1).show();

        misc_sprites.push(player_obj);
        misc_sprites.push(enemy_obj);

        let dice: Vec<_> = current_battle_state
            .rolled_dice
            .faces_to_render()
            .enumerate()
            .map(|(i, (face, _))| {
                let mut die_obj = obj.object_sprite(FACE_SPRITES.sprite_for_face(face));

                die_obj.set_y(120).set_x(i as u16 * 40 + 28).show();

                die_obj
            })
            .collect();

        let dice_cooldowns: Vec<_> = dice
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let mut cooldown_bar =
                    HealthBar::new((i as i32 * 40 + 28, 120 - 8).into(), 24, obj);
                cooldown_bar.hide();
                cooldown_bar
            })
            .collect();

        let shield_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Shield);

        let player_shield: Vec<_> = (0..5)
            .map(|i| {
                let mut shield_obj = obj.object_sprite(shield_sprite);
                shield_obj
                    .set_x(player_x + 18 + 11 * i)
                    .set_y(player_y)
                    .hide();

                shield_obj
            })
            .collect();

        let enemy_shield: Vec<_> = (0..5)
            .map(|i| {
                let mut shield_obj = obj.object_sprite(shield_sprite);
                shield_obj
                    .set_x(enemy_x - 16 - 11 * i)
                    .set_y(player_y)
                    .set_hflip(true)
                    .hide();

                shield_obj
            })
            .collect();

        let player_healthbar_x = 18;
        let enemy_healthbar_x = 180;
        let player_healthbar = HealthBar::new(
            (player_healthbar_x, player_y - 8).into(),
            HEALTH_BAR_WIDTH,
            obj,
        );
        let enemy_healthbar = HealthBar::new(
            (enemy_healthbar_x, player_y - 8).into(),
            HEALTH_BAR_WIDTH,
            obj,
        );

        let player_health_display = FractionDisplay::new(
            (
                player_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
                player_y,
            )
                .into(),
            3,
            obj,
        );
        let enemy_health_display = FractionDisplay::new(
            (
                enemy_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
                player_y,
            )
                .into(),
            3,
            obj,
        );

        let enemy_attack_display = (0..2)
            .map(|i| {
                let mut attack_obj = obj
                    .object_sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(EnemyAttackType::Attack));

                let attack_obj_position = Vector2D::new(120, 56 + 32 * i);
                attack_obj.set_position(attack_obj_position).hide();

                let mut attack_cooldown =
                    HealthBar::new(attack_obj_position + (32, 8).into(), 48, obj);
                attack_cooldown.hide();

                let attack_number_display =
                    NumberDisplay::new(attack_obj_position - (8, -10).into());

                EnemyAttackDisplay::new(attack_obj, attack_cooldown, attack_number_display)
            })
            .collect();

        let objs = BattleScreenDisplayObjects {
            dice,
            dice_cooldowns,
            player_shield,
            enemy_shield,

            player_healthbar,
            enemy_healthbar,
            player_health: player_health_display,
            enemy_health: enemy_health_display,

            enemy_attack_display,
        };

        Self {
            objs,

            animations: vec![],

            _misc_sprites: misc_sprites,
        }
    }

    pub fn update(
        &mut self,
        obj: &'a OamManaged,
        current_battle_state: &CurrentBattleState,
    ) -> Vec<Action> {
        for (i, player_shield) in self.objs.player_shield.iter_mut().enumerate() {
            if i < current_battle_state.player.shield_count as usize {
                player_shield
                    .show()
                    .set_sprite(obj.sprite(SHIELD.sprite(0)));
            } else {
                player_shield.hide();
            }
        }

        for (i, player_shield) in self.objs.enemy_shield.iter_mut().enumerate() {
            if i < current_battle_state.enemy.shield_count as usize {
                player_shield
                    .show()
                    .set_sprite(obj.sprite(SHIELD.sprite(0)));
            } else {
                player_shield.hide();
            }
        }

        self.objs.player_healthbar.set_value(
            ((current_battle_state.player.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.player.max_health) as usize,
            obj,
        );

        self.objs.enemy_healthbar.set_value(
            ((current_battle_state.enemy.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.enemy.max_health) as usize,
            obj,
        );

        self.objs.player_health.set_value(
            current_battle_state.player.health as usize,
            current_battle_state.player.max_health as usize,
            obj,
        );

        self.objs.enemy_health.set_value(
            current_battle_state.enemy.health as usize,
            current_battle_state.enemy.max_health as usize,
            obj,
        );

        for (i, attack) in current_battle_state.attacks.iter().enumerate() {
            self.objs.enemy_attack_display[i].update(attack, obj);
        }

        let mut actions_to_apply = vec![];

        // update the dice display to display the current values
        for ((die_obj, (current_face, cooldown)), cooldown_healthbar) in self
            .objs
            .dice
            .iter_mut()
            .zip(current_battle_state.rolled_dice.faces_to_render())
            .zip(self.objs.dice_cooldowns.iter_mut())
        {
            die_obj.set_sprite(obj.sprite(FACE_SPRITES.sprite_for_face(current_face)));

            if let Some(cooldown) = cooldown {
                cooldown_healthbar
                    .set_value((cooldown * 24 / MALFUNCTION_COOLDOWN_FRAMES) as usize, obj);
                cooldown_healthbar.show();
            } else {
                cooldown_healthbar.hide();
            }
        }

        let mut animations_to_remove = vec![];
        for (i, animation) in self.animations.iter_mut().enumerate() {
            match animation.update(&mut self.objs, obj, current_battle_state) {
                AnimationUpdateState::RemoveWithAction(a) => {
                    actions_to_apply.push(a);
                    animations_to_remove.push(i);
                }
                AnimationUpdateState::Continue => {}
            }
        }

        for &animation_to_remove in animations_to_remove.iter().rev() {
            self.animations.swap_remove(animation_to_remove);
        }

        actions_to_apply
    }

    pub fn add_action(&mut self, action: Action, obj: &'a OamManaged, sfx: &mut Sfx) {
        play_sound_for_action_start(&action, sfx);

        self.animations
            .push(AnimationStateHolder::for_action(action, obj));
    }
}

fn play_sound_for_action_start(action: &Action, sfx: &mut Sfx) {
    match action {
        Action::PlayerShoot { .. } | Action::EnemyShoot { .. } => sfx.shoot(),
        _ => {}
    }
}

struct EnemyAttackDisplay<'a> {
    face: Object<'a>,
    cooldown: HealthBar<'a>,
    number: NumberDisplay<'a>,
}

impl<'a> EnemyAttackDisplay<'a> {
    pub fn new(face: Object<'a>, cooldown: HealthBar<'a>, number: NumberDisplay<'a>) -> Self {
        Self {
            face,
            cooldown,
            number,
        }
    }

    pub fn update(&mut self, attack: &Option<EnemyAttackState>, obj: &'a OamManaged) {
        if let Some(attack) = attack {
            self.face.show().set_sprite(
                obj.sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(attack.attack_type())),
            );
            self.cooldown
                .set_value((attack.cooldown * 48 / attack.max_cooldown) as usize, obj);
            self.cooldown.show();

            self.number.set_value(attack.value_to_show(), obj);
        } else {
            self.face.hide();
            self.cooldown.hide();
            self.number.set_value(None, obj);
        }
    }
}

enum AnimationState<'a> {
    PlayerShoot { bullet: Object<'a>, x: i32 },
    PlayerActivateShield { amount: u32, frame: usize },
    PlayerDisrupt { bullet: Object<'a>, x: i32 },
    PlayerBurstShield { frame: usize },
    PlayerSendBurstShield { bullet: Object<'a>, x: i32 },
    PlayerHeal {},
    EnemyShoot { bullet: Object<'a>, x: i32 },
    EnemyShield { amount: u32, frame: usize },
    EnemyHeal {},
}

struct AnimationStateHolder<'a> {
    action: Action,
    state: AnimationState<'a>,
}

enum AnimationUpdateState {
    RemoveWithAction(Action),
    Continue,
}

impl<'a> AnimationStateHolder<'a> {
    fn for_action(a: Action, obj: &'a OamManaged) -> Self {
        let state = match a {
            Action::PlayerActivateShield { amount, .. } => {
                AnimationState::PlayerActivateShield { amount, frame: 0 }
            }
            Action::PlayerShoot { .. } => AnimationState::PlayerShoot {
                bullet: obj.object_sprite(BULLET_SPRITE),
                x: 64,
            },
            Action::PlayerDisrupt { .. } => AnimationState::PlayerDisrupt {
                bullet: obj.object_sprite(DISRUPT_BULLET),
                x: 64,
            },
            Action::PlayerHeal { .. } => AnimationState::PlayerHeal {},
            Action::PlayerBurstShield { .. } => AnimationState::PlayerBurstShield { frame: 0 },
            Action::PlayerSendBurstShield { .. } => AnimationState::PlayerSendBurstShield {
                bullet: obj.object_sprite(BURST_BULLET),
                x: 64,
            },
            Action::EnemyShoot { .. } => AnimationState::EnemyShoot {
                bullet: obj.object_sprite(BULLET_SPRITE),
                x: 175,
            },
            Action::EnemyShield { amount, .. } => AnimationState::EnemyShield { amount, frame: 0 },
            Action::EnemyHeal { .. } => AnimationState::EnemyHeal {},
        };

        Self { action: a, state }
    }

    fn update(
        &mut self,
        objs: &mut BattleScreenDisplayObjects<'a>,
        obj: &'a OamManaged,
        current_battle_state: &CurrentBattleState,
    ) -> AnimationUpdateState {
        match &mut self.state {
            AnimationState::PlayerShoot { bullet, x } => {
                bullet.show().set_x(*x as u16).set_y(36);
                *x += 4;

                if *x > 180 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
            AnimationState::PlayerDisrupt { bullet, x } => {
                bullet.show().set_x(*x as u16).set_y(36);
                *x += 2;

                if *x > 180 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
            AnimationState::PlayerActivateShield { amount, frame } => {
                // find all the shields that need animating
                let current_player_shields = current_battle_state.player.shield_count;
                if current_player_shields < *amount {
                    for i in current_player_shields..*amount {
                        objs.player_shield[i as usize]
                            .show()
                            .set_sprite(obj.sprite(SHIELD.sprite(3 - *frame / 2)));
                    }
                } else {
                    return AnimationUpdateState::RemoveWithAction(self.action.clone());
                }

                *frame += 1;

                if *frame >= 6 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
            AnimationState::EnemyShoot { bullet, x } => {
                bullet.show().set_hflip(true).set_x(*x as u16).set_y(36);
                *x -= 4;

                if *x < 50 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
            AnimationState::EnemyShield { amount, frame } => {
                // find all the shields that need animating
                let current_enemy_shields = current_battle_state.enemy.shield_count;
                if current_enemy_shields < *amount {
                    for i in current_enemy_shields..*amount {
                        objs.enemy_shield[i as usize]
                            .show()
                            .set_sprite(obj.sprite(SHIELD.sprite(3 - *frame / 2)));
                    }
                } else {
                    return AnimationUpdateState::RemoveWithAction(self.action.clone());
                }

                *frame += 1;

                if *frame > 6 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
            AnimationState::EnemyHeal {} => {
                AnimationUpdateState::RemoveWithAction(self.action.clone()) // TODO: Animation for healing
            }
            AnimationState::PlayerHeal {} => {
                AnimationUpdateState::RemoveWithAction(self.action.clone()) // TODO: Animation for healing
            }
            AnimationState::PlayerBurstShield { frame } => {
                if *frame < 10 {
                    for shield in objs.player_shield.iter_mut() {
                        shield.set_sprite(obj.sprite(SHIELD.sprite(*frame / 2)));
                    }

                    *frame += 1;

                    AnimationUpdateState::Continue
                } else {
                    for shield in objs.player_shield.iter_mut() {
                        shield.set_sprite(obj.sprite(SHIELD.sprite(0)));
                    }

                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                }
            }
            AnimationState::PlayerSendBurstShield { bullet, x } => {
                bullet.show().set_x(*x as u16).set_y(36);
                *x += 1;

                if *x > 180 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
        }
    }
}
