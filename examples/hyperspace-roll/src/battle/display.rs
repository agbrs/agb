use agb::display::object::Object;
use agb::display::GraphicsFrame;
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

struct BattleScreenDisplayObjects {
    dice: Vec<Object>,
    dice_cooldowns: Vec<HealthBar>,
    player_shield: Vec<Object>,
    enemy_shield: Vec<Object>,

    player_healthbar: HealthBar,
    enemy_healthbar: HealthBar,
    player_health: FractionDisplay,
    enemy_health: FractionDisplay,

    enemy_attack_display: Vec<EnemyAttackDisplay>,
}

pub struct BattleScreenDisplay {
    objs: BattleScreenDisplayObjects,
    animations: Vec<AnimationStateHolder>,

    misc_sprites: Vec<Object>,
}

const HEALTH_BAR_WIDTH: usize = 48;

impl BattleScreenDisplay {
    pub fn new(current_battle_state: &CurrentBattleState) -> Self {
        let mut misc_sprites = vec![];
        let player_x = 12;
        let player_y = 8;
        let enemy_x = 167;

        let player_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Player);
        let enemy_sprite = SHIP_SPRITES.sprite_for_ship(if rng::next_i32() % 2 == 0 {
            Ship::Drone
        } else {
            Ship::PilotedShip
        });

        let mut player_obj = Object::new(player_sprite);
        let mut enemy_obj = Object::new(enemy_sprite);

        player_obj.set_position((player_x, player_y));
        enemy_obj.set_position((enemy_x, player_y));

        misc_sprites.push(player_obj);
        misc_sprites.push(enemy_obj);

        let dice: Vec<_> = current_battle_state
            .rolled_dice
            .faces_to_render()
            .enumerate()
            .map(|(i, (face, _))| {
                let mut die_obj = Object::new(FACE_SPRITES.sprite_for_face(face));

                die_obj.set_position((120, i as i32 * 40 + 28));

                die_obj
            })
            .collect();

        let dice_cooldowns: Vec<_> = dice
            .iter()
            .enumerate()
            .map(|(i, _)| HealthBar::new((i as i32 * 40 + 28, 120 - 8).into(), 24))
            .collect();

        let shield_sprite = SHIP_SPRITES.sprite_for_ship(Ship::Shield);

        let player_shield: Vec<_> = (0..5)
            .map(|i| {
                let mut shield_obj = Object::new(shield_sprite);
                shield_obj.set_position((player_x + 18 + 11 * i, player_y));

                shield_obj
            })
            .collect();

        let enemy_shield: Vec<_> = (0..5)
            .map(|i| {
                let mut shield_obj = Object::new(shield_sprite);
                shield_obj
                    .set_position((enemy_x - 16 - 11 * i, player_y))
                    .set_hflip(true);

                shield_obj
            })
            .collect();

        let player_healthbar_x = 18;
        let enemy_healthbar_x = 180;
        let player_healthbar =
            HealthBar::new((player_healthbar_x, player_y - 8).into(), HEALTH_BAR_WIDTH);
        let enemy_healthbar =
            HealthBar::new((enemy_healthbar_x, player_y - 8).into(), HEALTH_BAR_WIDTH);

        let player_health_display = FractionDisplay::new(
            (
                player_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
                player_y,
            )
                .into(),
            3,
        );
        let enemy_health_display = FractionDisplay::new(
            (
                enemy_healthbar_x + HEALTH_BAR_WIDTH as u16 / 2 - 16,
                player_y,
            )
                .into(),
            3,
        );

        let enemy_attack_display = (0..2)
            .map(|i| {
                let mut attack_obj =
                    Object::new(ENEMY_ATTACK_SPRITES.sprite_for_attack(EnemyAttackType::Attack));

                let attack_obj_position = Vector2D::new(120, 56 + 32 * i);
                attack_obj.set_position(attack_obj_position);

                let attack_cooldown = HealthBar::new(attack_obj_position + (32, 8).into(), 48);

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

            misc_sprites,
        }
    }

    pub fn update(
        &mut self,
        current_battle_state: &CurrentBattleState,
        frame: &mut GraphicsFrame,
    ) -> Vec<Action> {
        for player_shield in self
            .objs
            .player_shield
            .iter_mut()
            .take(current_battle_state.player.shield_count as usize)
        {
            player_shield.set_sprite(SHIELD.sprite(0));
            player_shield.show(frame);
        }

        for player_shield in self
            .objs
            .enemy_shield
            .iter_mut()
            .take(current_battle_state.enemy.shield_count as usize)
        {
            player_shield.set_sprite(SHIELD.sprite(0));
            player_shield.show(frame);
        }

        self.objs.player_healthbar.set_value(
            ((current_battle_state.player.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.player.max_health) as usize,
        );
        self.objs.player_healthbar.show(frame);

        self.objs.enemy_healthbar.set_value(
            ((current_battle_state.enemy.health * HEALTH_BAR_WIDTH as u32)
                / current_battle_state.enemy.max_health) as usize,
        );
        self.objs.enemy_healthbar.show(frame);

        self.objs.player_health.set_value(
            current_battle_state.player.health as usize,
            current_battle_state.player.max_health as usize,
        );
        self.objs.player_health.show(frame);

        self.objs.enemy_health.set_value(
            current_battle_state.enemy.health as usize,
            current_battle_state.enemy.max_health as usize,
        );
        self.objs.enemy_health.show(frame);

        for (i, attack) in current_battle_state.attacks.iter().enumerate() {
            self.objs.enemy_attack_display[i].update(attack, frame);
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
            die_obj.set_sprite(FACE_SPRITES.sprite_for_face(current_face));
            die_obj.show(frame);

            if let Some(cooldown) = cooldown {
                cooldown_healthbar
                    .set_value((cooldown * 24 / MALFUNCTION_COOLDOWN_FRAMES) as usize);
                cooldown_healthbar.show(frame);
            }
        }

        let mut animations_to_remove = vec![];
        for (i, animation) in self.animations.iter_mut().enumerate() {
            match animation.update(&mut self.objs, current_battle_state, frame) {
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

        for obj in self.misc_sprites.iter() {
            obj.show(frame);
        }

        actions_to_apply
    }

    pub fn add_action(&mut self, action: Action, sfx: &mut Sfx) {
        play_sound_for_action_start(&action, sfx);

        self.animations
            .push(AnimationStateHolder::for_action(action));
    }
}

fn play_sound_for_action_start(action: &Action, sfx: &mut Sfx) {
    match action {
        Action::PlayerShoot { .. } | Action::EnemyShoot { .. } => sfx.shoot(),
        _ => {}
    }
}

struct EnemyAttackDisplay {
    face: Object,
    cooldown: HealthBar,
    number: NumberDisplay,
}

impl EnemyAttackDisplay {
    pub fn new(face: Object, cooldown: HealthBar, number: NumberDisplay) -> Self {
        Self {
            face,
            cooldown,
            number,
        }
    }

    pub fn update(&mut self, attack: &Option<EnemyAttackState>, frame: &mut GraphicsFrame) {
        if let Some(attack) = attack {
            self.face
                .set_sprite(ENEMY_ATTACK_SPRITES.sprite_for_attack(attack.attack_type()));
            self.face.show(frame);
            self.cooldown
                .set_value((attack.cooldown * 48 / attack.max_cooldown) as usize);
            self.cooldown.show(frame);

            self.number.set_value(attack.value_to_show());
        } else {
            self.number.set_value(None);
        }

        self.number.show(frame);
    }
}

enum AnimationState {
    PlayerShoot { bullet: Object, x: i32 },
    PlayerActivateShield { amount: u32, frame: usize },
    PlayerDisrupt { bullet: Object, x: i32 },
    PlayerBurstShield { frame: usize },
    PlayerSendBurstShield { bullet: Object, x: i32 },
    PlayerHeal {},
    EnemyShoot { bullet: Object, x: i32 },
    EnemyShield { amount: u32, frame: usize },
    EnemyHeal {},
}

struct AnimationStateHolder {
    action: Action,
    state: AnimationState,
}

enum AnimationUpdateState {
    RemoveWithAction(Action),
    Continue,
}

impl AnimationStateHolder {
    fn for_action(a: Action) -> Self {
        let state = match a {
            Action::PlayerActivateShield { amount, .. } => {
                AnimationState::PlayerActivateShield { amount, frame: 0 }
            }
            Action::PlayerShoot { .. } => AnimationState::PlayerShoot {
                bullet: Object::new(BULLET_SPRITE),
                x: 64,
            },
            Action::PlayerDisrupt { .. } => AnimationState::PlayerDisrupt {
                bullet: Object::new(DISRUPT_BULLET),
                x: 64,
            },
            Action::PlayerHeal { .. } => AnimationState::PlayerHeal {},
            Action::PlayerBurstShield { .. } => AnimationState::PlayerBurstShield { frame: 0 },
            Action::PlayerSendBurstShield { .. } => AnimationState::PlayerSendBurstShield {
                bullet: Object::new(BURST_BULLET),
                x: 64,
            },
            Action::EnemyShoot { .. } => AnimationState::EnemyShoot {
                bullet: Object::new(BULLET_SPRITE),
                x: 175,
            },
            Action::EnemyShield { amount, .. } => AnimationState::EnemyShield { amount, frame: 0 },
            Action::EnemyHeal { .. } => AnimationState::EnemyHeal {},
        };

        Self { action: a, state }
    }

    fn update(
        &mut self,
        objs: &mut BattleScreenDisplayObjects,
        current_battle_state: &CurrentBattleState,
        frame: &mut GraphicsFrame,
    ) -> AnimationUpdateState {
        match &mut self.state {
            AnimationState::PlayerShoot { bullet, x } => {
                bullet.set_position((*x, 36));
                bullet.show(frame);

                *x += 4;

                if *x > 180 {
                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                } else {
                    AnimationUpdateState::Continue
                }
            }
            AnimationState::PlayerDisrupt { bullet, x } => {
                bullet.set_position((*x, 36));
                bullet.show(frame);

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
                        objs.player_shield[i as usize].set_sprite(SHIELD.sprite(3 - *frame / 2));
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
                bullet.set_hflip(true).set_position((*x, 36));
                bullet.show(frame);

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
                        objs.enemy_shield[i as usize].set_sprite(SHIELD.sprite(3 - *frame / 2));
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
                        shield.set_sprite(SHIELD.sprite(*frame / 2));
                    }

                    *frame += 1;

                    AnimationUpdateState::Continue
                } else {
                    for shield in objs.player_shield.iter_mut() {
                        shield.set_sprite(SHIELD.sprite(0));
                    }

                    AnimationUpdateState::RemoveWithAction(self.action.clone())
                }
            }
            AnimationState::PlayerSendBurstShield { bullet, x } => {
                bullet.set_position((*x, 36));
                bullet.show(frame);

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
