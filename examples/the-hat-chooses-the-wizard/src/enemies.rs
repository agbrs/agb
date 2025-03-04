use super::{Entity, FixedNumberType, HatState, Level, sfx::SfxPlayer};
use agb::fixnum::Vector2D;

use crate::sprites;

enum UpdateState {
    Nothing,
    KillPlayer,
    Remove,
}

#[derive(Default)]
pub enum Enemy {
    Slime(Slime),
    Snail(Snail),
    #[default]
    Empty,
}

pub enum EnemyUpdateState {
    None,
    KillPlayer,
}

impl Enemy {
    pub fn new_slime(start_pos: Vector2D<FixedNumberType>) -> Self {
        Enemy::Slime(Slime::new(start_pos + (0, 1).into()))
    }

    pub fn new_snail(start_pos: Vector2D<FixedNumberType>) -> Self {
        Enemy::Snail(Snail::new(start_pos))
    }

    pub fn collides_with_hat(&self, position: Vector2D<FixedNumberType>) -> bool {
        match self {
            Enemy::Snail(snail) => snail.collides_with(position),
            _ => false,
        }
    }

    pub fn update(
        &mut self,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        hat_state: HatState,
        timer: i32,
        sfx_player: &mut SfxPlayer,
    ) -> EnemyUpdateState {
        let update_state = match self {
            Enemy::Slime(slime) => slime.update(level, player_pos, hat_state, timer, sfx_player),
            Enemy::Snail(snail) => snail.update(level, player_pos, hat_state, timer, sfx_player),
            Enemy::Empty => UpdateState::Nothing,
        };

        match update_state {
            UpdateState::Remove => {
                *self = Enemy::Empty;
                EnemyUpdateState::None
            }
            UpdateState::KillPlayer => EnemyUpdateState::KillPlayer,
            UpdateState::Nothing => EnemyUpdateState::None,
        }
    }

    pub fn entity(&mut self) -> Option<&mut Entity> {
        match self {
            Enemy::Slime(slime) => Some(&mut slime.enemy_info.entity),
            Enemy::Snail(snail) => Some(&mut snail.enemy_info.entity),
            Enemy::Empty => None,
        }
    }
}

struct EnemyInfo {
    entity: Entity,
}

impl EnemyInfo {
    fn new(start_pos: Vector2D<FixedNumberType>, collision: Vector2D<u16>) -> Self {
        let mut enemy_info = EnemyInfo {
            entity: Entity::new(collision),
        };
        enemy_info.entity.position = start_pos;
        enemy_info
    }

    fn update(&mut self, level: &Level) {
        for &enemy_stop in level.enemy_stops {
            if (self.entity.position + self.entity.velocity - enemy_stop.into())
                .manhattan_distance()
                < 8.into()
            {
                self.entity.velocity = (0, 0).into();
            }
        }

        self.entity.update_position(level);
    }
}

enum SlimeState {
    Idle,
    Jumping(i32), // the start frame of the jumping animation
    Dying(i32),   // the start frame of the dying animation
}

pub struct Slime {
    enemy_info: EnemyInfo,
    state: SlimeState,
}

impl Slime {
    fn new(start_pos: Vector2D<FixedNumberType>) -> Self {
        Slime {
            enemy_info: EnemyInfo::new(start_pos, (14u16, 14u16).into()),
            state: SlimeState::Idle,
        }
    }

    fn update(
        &mut self,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        hat_state: HatState,
        timer: i32,
        sfx_player: &mut SfxPlayer,
    ) -> UpdateState {
        let player_has_collided =
            (self.enemy_info.entity.position - player_pos).magnitude_squared() < (10 * 10).into();

        match self.state {
            SlimeState::Idle => {
                let offset = (timer / 16) as usize;

                let frame = sprites::SLIME_IDLE.animation_sprite(offset);

                self.enemy_info.entity.sprite.set_sprite(frame);

                if (self.enemy_info.entity.position - player_pos).magnitude_squared()
                    < (64 * 64).into()
                {
                    self.state = SlimeState::Jumping(timer);

                    let x_vel: FixedNumberType =
                        if self.enemy_info.entity.position.x > player_pos.x {
                            -1
                        } else {
                            1
                        }
                        .into();

                    self.enemy_info.entity.velocity = (x_vel / 4, 0.into()).into();
                }

                if player_has_collided {
                    if hat_state == HatState::WizardTowards {
                        self.state = SlimeState::Dying(timer);
                    } else {
                        return UpdateState::KillPlayer;
                    }
                }
            }
            SlimeState::Jumping(jumping_start_frame) => {
                let offset = (timer - jumping_start_frame) as usize / 4;

                if timer == jumping_start_frame + 1 {
                    sfx_player.slime_jump();
                }

                if offset >= 7 {
                    self.enemy_info.entity.velocity = (0, 0).into();
                    self.state = SlimeState::Idle;
                } else {
                    let frame = sprites::SLIME_JUMP.animation_sprite(offset);

                    self.enemy_info.entity.sprite.set_sprite(frame);
                }

                if player_has_collided {
                    if hat_state == HatState::WizardTowards {
                        self.state = SlimeState::Dying(timer);
                    } else {
                        return UpdateState::KillPlayer;
                    }
                }
            }
            SlimeState::Dying(dying_start_frame) => {
                if timer == dying_start_frame + 1 {
                    sfx_player.slime_death();
                }

                let offset = (timer - dying_start_frame) as usize / 4;
                self.enemy_info.entity.velocity = (0, 0).into();

                if offset >= 4 {
                    return UpdateState::Remove;
                }

                let frame = sprites::SLIME_SPLAT.animation_sprite(offset);

                self.enemy_info.entity.sprite.set_sprite(frame);
            }
        }

        self.enemy_info.update(level);

        UpdateState::Nothing
    }
}

enum SnailState {
    Idle(i32),       // start frame (or 0 if newly created)
    Emerging(i32),   // start frame
    Retreating(i32), // start frame
    Moving(i32),     // start frame
    Death(i32),      // start frame
}

pub struct Snail {
    enemy_info: EnemyInfo,
    state: SnailState,
}

impl Snail {
    fn new(start_pos: Vector2D<FixedNumberType>) -> Self {
        Snail {
            enemy_info: EnemyInfo::new(start_pos, (16u16, 16u16).into()),
            state: SnailState::Idle(0),
        }
    }

    pub fn collides_with(&self, position: Vector2D<FixedNumberType>) -> bool {
        (self.enemy_info.entity.position - position).magnitude_squared() < (15 * 15).into()
    }

    fn update(
        &mut self,
        level: &Level,
        player_pos: Vector2D<FixedNumberType>,
        hat_state: HatState,
        timer: i32,
        sfx_player: &mut SfxPlayer,
    ) -> UpdateState {
        let player_has_collided =
            (self.enemy_info.entity.position - player_pos).magnitude_squared() < (10 * 10).into();

        match self.state {
            SnailState::Idle(wait_time) => {
                self.enemy_info.entity.velocity = (0, 0).into();

                if wait_time == 0 || timer - wait_time > 120 {
                    // wait at least 2 seconds after switching to this state
                    if (self.enemy_info.entity.position - player_pos).magnitude_squared()
                        < (48 * 48).into()
                    {
                        // player is close
                        self.state = SnailState::Emerging(timer);
                        sfx_player.snail_emerge();
                    }
                }

                let frame = sprites::SNAIL_IDLE.animation_sprite(0);

                self.enemy_info.entity.sprite.set_sprite(frame);
                if player_has_collided {
                    if hat_state != HatState::WizardTowards {
                        return UpdateState::KillPlayer;
                    } else {
                        self.state = SnailState::Death(timer);
                    }
                }
            }
            SnailState::Emerging(time) => {
                let offset = (timer - time) as usize / 4;

                if offset >= 5 {
                    self.state = SnailState::Moving(timer);
                }
                self.enemy_info.entity.velocity = (0, 0).into();

                let frame = sprites::SNAIL_EMERGE.animation_sprite(offset);

                self.enemy_info.entity.sprite.set_sprite(frame);

                if player_has_collided {
                    if hat_state != HatState::WizardTowards {
                        return UpdateState::KillPlayer;
                    } else if hat_state == HatState::WizardTowards {
                        self.state = SnailState::Death(timer);
                    }
                }
            }
            SnailState::Moving(time) => {
                if timer - time > 240 {
                    // only move for 4 seconds
                    self.state = SnailState::Retreating(timer);
                    sfx_player.snail_retreat();
                }

                let offset = (timer - time) as usize / 8;

                let frame = sprites::SNAIL_MOVE.animation_sprite(offset);

                self.enemy_info.entity.sprite.set_sprite(frame);

                if timer % 32 == 0 {
                    let x_vel: FixedNumberType =
                        if self.enemy_info.entity.position.x < player_pos.x {
                            self.enemy_info.entity.sprite.set_hflip(false);
                            1
                        } else {
                            self.enemy_info.entity.sprite.set_hflip(true);
                            -1
                        }
                        .into();

                    self.enemy_info.entity.velocity = (x_vel / 8, 0.into()).into();
                }

                if player_has_collided {
                    if hat_state != HatState::WizardTowards {
                        return UpdateState::KillPlayer;
                    } else if hat_state == HatState::WizardTowards {
                        self.state = SnailState::Death(timer);
                    }
                }
            }
            SnailState::Retreating(time) => {
                let offset = 5 - (timer - time) as usize / 4;

                if offset == 0 {
                    self.state = SnailState::Idle(timer);
                }

                let frame = sprites::SNAIL_EMERGE.animation_sprite(offset);

                self.enemy_info.entity.sprite.set_sprite(frame);
                self.enemy_info.entity.velocity = (0, 0).into();

                if player_has_collided {
                    if hat_state != HatState::WizardTowards {
                        return UpdateState::KillPlayer;
                    } else if hat_state == HatState::WizardTowards {
                        self.state = SnailState::Death(timer);
                    }
                }
            }
            SnailState::Death(time) => {
                if timer == time + 1 {
                    sfx_player.snail_death();
                }

                let offset = (timer - time) as usize / 4;
                let frame = if offset < 5 {
                    sprites::SNAIL_EMERGE.animation_sprite(5 - offset)
                } else if offset == 5 {
                    sprites::SNAIL_IDLE.animation_sprite(0)
                } else if offset < 5 + 7 {
                    sprites::SNAIL_DEATH.animation_sprite(offset - 5)
                } else {
                    return UpdateState::Remove;
                };

                self.enemy_info.entity.sprite.set_sprite(frame);
                self.enemy_info.entity.velocity = (0, 0).into();
            }
        }

        self.enemy_info.update(level);

        UpdateState::Nothing
    }
}
