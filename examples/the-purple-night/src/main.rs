#![no_std]
#![no_main]

extern crate agb;
extern crate alloc;

mod rng;
mod sfx;

use alloc::vec::Vec;

use rng::get_random;

use agb::{
    display::{
        background::{BackgroundDistributor, BackgroundRegular},
        object::{ObjectControl, ObjectStandard},
        Priority, HEIGHT, WIDTH,
    },
    input::{Button, ButtonController, Tri},
    number::{FixedNum, Rect, Vector2D},
};
use generational_arena::Arena;

agb::include_gfx!("gfx/objects.toml");
agb::include_gfx!("gfx/background.toml");

type Number = FixedNum<8>;

struct Level {
    background: BackgroundRegular<'static>,
    foreground: BackgroundRegular<'static>,
    clouds: BackgroundRegular<'static>,

    slime_spawns: Vec<(u16, u16)>,
    bat_spawns: Vec<(u16, u16)>,
    emu_spawns: Vec<(u16, u16)>,
}

impl Level {
    fn load_level(
        mut backdrop: BackgroundRegular<'static>,
        mut foreground: BackgroundRegular<'static>,
        mut clouds: BackgroundRegular<'static>,
    ) -> Self {
        backdrop.set_position(Vector2D::new(0, 0));
        backdrop.set_map(agb::display::background::Map::new(
            tilemap::BACKGROUND_MAP,
            Vector2D::new(tilemap::WIDTH, tilemap::HEIGHT),
            0,
        ));
        backdrop.set_priority(Priority::P2);

        foreground.set_position(Vector2D::new(0, 0));
        foreground.set_map(agb::display::background::Map::new(
            tilemap::FOREGROUND_MAP,
            Vector2D::new(tilemap::WIDTH, tilemap::HEIGHT),
            0,
        ));
        foreground.set_priority(Priority::P0);

        clouds.set_position(Vector2D::new(0, -5));
        clouds.set_map(agb::display::background::Map::new(
            tilemap::CLOUD_MAP,
            Vector2D::new(tilemap::WIDTH, tilemap::HEIGHT),
            0,
        ));
        clouds.set_priority(Priority::P3);

        backdrop.commit();
        foreground.commit();
        clouds.commit();

        backdrop.show();
        foreground.show();
        clouds.show();

        let slime_spawns = tilemap::SLIME_SPAWNS_X
            .iter()
            .enumerate()
            .map(|(i, x)| (*x, tilemap::SLIME_SPAWNS_Y[i]))
            .collect();

        let bat_spawns = tilemap::BAT_SPAWNS_X
            .iter()
            .enumerate()
            .map(|(i, x)| (*x, tilemap::BAT_SPAWNS_Y[i]))
            .collect();

        let emu_spawns = tilemap::EMU_SPAWNS_X
            .iter()
            .enumerate()
            .map(|(i, x)| (*x, tilemap::EMU_SPAWNS_Y[i]))
            .collect();

        Self {
            background: backdrop,
            foreground,
            clouds,

            slime_spawns,
            bat_spawns,
            emu_spawns,
        }
    }

    fn collides(&self, v: Vector2D<Number>) -> Option<Rect<Number>> {
        let factor: Number = Number::new(1) / Number::new(8);
        let (x, y) = (v * factor).floor().get();

        if (x < 0 || x > tilemap::WIDTH as i32) || (y < 0 || y > tilemap::HEIGHT as i32) {
            return Some(Rect::new((x * 8, y * 8).into(), (8, 8).into()));
        }
        let position = tilemap::WIDTH as usize * y as usize + x as usize;
        let tile_foreground = tilemap::FOREGROUND_MAP[position];
        let tile_background = tilemap::BACKGROUND_MAP[position];
        let tile_foreground_property = tilemap::TILE_TYPES[tile_foreground as usize];
        let tile_background_property = tilemap::TILE_TYPES[tile_background as usize];

        if tile_foreground_property == 1 || tile_background_property == 1 {
            Some(Rect::new((x * 8, y * 8).into(), (8, 8).into()))
        } else {
            None
        }
    }
}

struct Entity<'a> {
    sprite: ObjectStandard<'a>,
    position: Vector2D<Number>,
    velocity: Vector2D<Number>,
    collision_mask: Rect<u16>,
    visible: bool,
}

impl<'a> Entity<'a> {
    fn new(object_controller: &'a ObjectControl, collision_mask: Rect<u16>) -> Self {
        let mut sprite = object_controller.get_object_standard();
        sprite.set_priority(Priority::P1);
        Entity {
            sprite,
            collision_mask,
            position: (0, 0).into(),
            velocity: (0, 0).into(),
            visible: true,
        }
    }

    fn update_position(&mut self, level: &Level) -> Vector2D<Number> {
        let initial_position = self.position;

        let y = self.velocity.y.to_raw().signum();
        if y != 0 {
            let (delta, collided) =
                self.collision_in_direction((0, y).into(), self.velocity.y.abs(), |v| {
                    level.collides(v)
                });
            self.position += delta;
            if collided {
                self.velocity.y = 0.into();
            }
        }
        let x = self.velocity.x.to_raw().signum();
        if x != 0 {
            let (delta, collided) =
                self.collision_in_direction((x, 0).into(), self.velocity.x.abs(), |v| {
                    level.collides(v)
                });
            self.position += delta;
            if collided {
                self.velocity.x = 0.into();
            }
        }

        self.position - initial_position
    }

    fn update_position_without_collision(&mut self) -> Vector2D<Number> {
        self.position += self.velocity;

        self.velocity
    }

    fn collider(&self) -> Rect<Number> {
        let mut number_collision: Rect<Number> = Rect::new(
            (
                self.collision_mask.position.x as i32,
                self.collision_mask.position.y as i32,
            )
                .into(),
            (
                self.collision_mask.size.x as i32,
                self.collision_mask.size.y as i32,
            )
                .into(),
        );
        number_collision.position =
            self.position + number_collision.position - number_collision.size / 2;
        number_collision
    }

    fn collision_in_direction(
        &mut self,
        direction: Vector2D<Number>,
        distance: Number,
        collision: impl Fn(Vector2D<Number>) -> Option<Rect<Number>>,
    ) -> (Vector2D<Number>, bool) {
        let number_collision = self.collider();

        let center_collision_point: Vector2D<Number> = number_collision.position
            + number_collision.size / 2
            + number_collision.size.hadamard(direction) / 2;

        let direction_transpose: Vector2D<Number> = direction.swap();
        let small = direction_transpose * Number::new(4) / 64;
        let triple_collider: [Vector2D<Number>; 2] = [
            center_collision_point + number_collision.size.hadamard(direction_transpose) / 2
                - small,
            center_collision_point - number_collision.size.hadamard(direction_transpose) / 2
                + small,
        ];

        let original_distance = direction * distance;
        let mut final_distance = original_distance;

        let mut has_collided = false;

        for edge_point in triple_collider {
            let point = edge_point + original_distance;
            if let Some(collider) = collision(point) {
                let center = collider.position + collider.size / 2;
                let edge = center - collider.size.hadamard(direction) / 2;
                let new_distance = (edge - center_collision_point)
                    .hadamard((direction.x.abs(), direction.y.abs()).into());
                if final_distance.manhattan_distance() > new_distance.manhattan_distance() {
                    final_distance = new_distance;
                }
                has_collided = true;
            }
        }

        (final_distance, has_collided)
    }

    fn commit_with_fudge(&mut self, offset: Vector2D<Number>, fudge: Vector2D<i32>) {
        if !self.visible {
            self.sprite.hide();
        } else {
            let position = (self.position - offset).floor() + fudge;
            self.sprite.set_position(position - (8, 8).into());
            if position.x < -8
                || position.x > WIDTH + 8
                || position.y < -8
                || position.y > HEIGHT + 8
            {
                self.sprite.hide();
            } else {
                self.sprite.show();
            }
        }
        self.sprite.commit();
    }

    fn commit_with_size(&mut self, offset: Vector2D<Number>, size: Vector2D<i32>) {
        if !self.visible {
            self.sprite.hide();
        } else {
            let position = (self.position - offset).floor();
            self.sprite.set_position(position - size / 2);
            if position.x < -8
                || position.x > WIDTH + 8
                || position.y < -8
                || position.y > HEIGHT + 8
            {
                self.sprite.hide();
            } else {
                self.sprite.show();
            }
        }
        self.sprite.commit();
    }
}

#[derive(PartialEq, Eq)]
enum PlayerState {
    OnGround,
    InAir,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SwordState {
    LongSword,
    ShortSword,
    Dagger,
    Swordless,
}

impl SwordState {
    fn ground_walk_force(self) -> Number {
        match self {
            SwordState::LongSword => Number::new(4) / 16,
            SwordState::ShortSword => Number::new(5) / 16,
            SwordState::Dagger => Number::new(6) / 16,
            SwordState::Swordless => Number::new(6) / 16,
        }
    }
    fn jump_impulse(self) -> Number {
        match self {
            SwordState::LongSword => Number::new(32) / 16,
            SwordState::ShortSword => Number::new(35) / 16,
            SwordState::Dagger => Number::new(36) / 16,
            SwordState::Swordless => Number::new(42) / 16,
        }
    }
    fn air_move_force(self) -> Number {
        match self {
            SwordState::LongSword => Number::new(4) / 256,
            SwordState::ShortSword => Number::new(5) / 256,
            SwordState::Dagger => Number::new(6) / 256,
            SwordState::Swordless => Number::new(6) / 256,
        }
    }
    fn idle_animation(self, counter: &mut u16) -> u16 {
        if *counter >= 4 * 8 {
            *counter = 0;
        }
        match self {
            SwordState::LongSword => (0 + *counter / 8) * 4,
            SwordState::ShortSword => (41 + *counter / 8) * 4,
            SwordState::Dagger => (96 + *counter / 8) * 4,
            SwordState::Swordless => (154 + *counter / 8) * 4,
        }
    }
    fn jump_offset(self) -> u16 {
        match self {
            SwordState::LongSword => 10,
            SwordState::ShortSword => 51,
            SwordState::Dagger => 106,
            SwordState::Swordless => 164,
        }
    }
    fn walk_animation(self, counter: &mut u16) -> u16 {
        if *counter >= 6 * 4 {
            *counter = 0;
        }
        match self {
            SwordState::LongSword => (4 + *counter / 4) * 4,
            SwordState::ShortSword => (45 + *counter / 4) * 4,
            SwordState::Dagger => (100 + *counter / 4) * 4,
            SwordState::Swordless => (158 + *counter / 4) * 4,
        }
    }
    fn attack_duration(self) -> u16 {
        match self {
            SwordState::LongSword => 60,
            SwordState::ShortSword => 40,
            SwordState::Dagger => 20,
            SwordState::Swordless => 0,
        }
    }
    fn jump_attack_duration(self) -> u16 {
        match self {
            SwordState::LongSword => 34,
            SwordState::ShortSword => 28,
            SwordState::Dagger => 20,
            SwordState::Swordless => 0,
        }
    }
    fn attack_frame(self, timer: u16) -> u16 {
        match self {
            SwordState::LongSword => (self.attack_duration() - timer) / 8,
            SwordState::ShortSword => (self.attack_duration() - timer) / 8,
            SwordState::Dagger => (self.attack_duration() - timer) / 8,
            SwordState::Swordless => (self.attack_duration() - timer) / 8,
        }
    }
    fn jump_attack_frame(self, timer: u16) -> u16 {
        (self.jump_attack_duration() - timer) / 8
    }
    fn hold_frame(self) -> u16 {
        7
    }
    fn jump_attack_hold_frame(self) -> u16 {
        match self {
            SwordState::LongSword => 13,
            SwordState::ShortSword => 54,
            SwordState::Dagger => 109,
            SwordState::Swordless => 0,
        }
    }

    fn cooldown_time(self) -> u16 {
        match self {
            SwordState::LongSword => 20,
            SwordState::ShortSword => 10,
            SwordState::Dagger => 1,
            SwordState::Swordless => 0,
        }
    }
    fn to_sprite_id(self, frame: u16) -> u16 {
        match self {
            SwordState::LongSword => (16 + frame) * 4,
            SwordState::ShortSword => (57 + frame) * 4,
            SwordState::Dagger => (112 + frame) * 4,
            SwordState::Swordless => 0,
        }
    }
    fn to_jump_sprite_id(self, frame: u16) -> u16 {
        if frame == self.jump_attack_hold_frame() {
            frame * 4
        } else {
            match self {
                SwordState::LongSword => (24 + frame) * 4,
                SwordState::ShortSword => (65 + frame) * 4,
                SwordState::Dagger => (120 + frame) * 4,
                SwordState::Swordless => 0,
            }
        }
    }
    fn fudge(self, frame: u16) -> i32 {
        match self {
            SwordState::LongSword => long_sword_fudge(frame),
            SwordState::ShortSword => short_sword_fudge(frame),
            SwordState::Dagger => 0,
            SwordState::Swordless => 0,
        }
    }
    // origin at top left pre fudge boxes
    fn ground_attack_hurtbox(self, frame: u16) -> Option<Rect<Number>> {
        match self {
            SwordState::LongSword => long_sword_hurtbox(frame),
            SwordState::ShortSword => short_sword_hurtbox(frame),
            SwordState::Dagger => dagger_hurtbox(frame),
            SwordState::Swordless => None,
        }
    }
    fn air_attack_hurtbox(self, _frame: u16) -> Option<Rect<Number>> {
        Some(Rect::new((0, 0).into(), (16, 16).into()))
    }
}

fn dagger_hurtbox(_frame: u16) -> Option<Rect<Number>> {
    Some(Rect::new((9, 5).into(), (7, 9).into()))
}

fn long_sword_hurtbox(frame: u16) -> Option<Rect<Number>> {
    match frame {
        0 => Some(Rect::new((1, 10).into(), (6, 3).into())),
        1 => Some(Rect::new((0, 9).into(), (7, 2).into())),
        2 => Some(Rect::new((0, 1).into(), (6, 8).into())),
        3 => Some(Rect::new((3, 0).into(), (6, 8).into())),
        4 => Some(Rect::new((6, 3).into(), (10, 8).into())),
        5 => Some(Rect::new((6, 5).into(), (10, 9).into())),
        6 => Some(Rect::new((6, 5).into(), (10, 9).into())),
        7 => Some(Rect::new((6, 5).into(), (10, 9).into())),
        _ => None,
    }
}

fn short_sword_hurtbox(frame: u16) -> Option<Rect<Number>> {
    match frame {
        0 => None,
        1 => Some(Rect::new((10, 5).into(), (3, 5).into())),
        2 => Some(Rect::new((8, 5).into(), (6, 6).into())),
        3 => Some(Rect::new((8, 6).into(), (8, 8).into())),
        4 => Some(Rect::new((8, 7).into(), (5, 7).into())),
        5 => Some(Rect::new((8, 7).into(), (7, 7).into())),
        6 => Some(Rect::new((8, 5).into(), (7, 8).into())),
        7 => Some(Rect::new((8, 4).into(), (4, 7).into())),
        _ => None,
    }
}

fn short_sword_fudge(frame: u16) -> i32 {
    match frame {
        0 => 0,
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 3,
        5 => 3,
        6 => 3,
        7 => 3,
        _ => 0,
    }
}

fn long_sword_fudge(frame: u16) -> i32 {
    match frame {
        0 => 0,
        1 => 0,
        2 => 1,
        3 => 4,
        4 => 5,
        5 => 5,
        6 => 5,
        7 => 4,
        _ => 0,
    }
}

enum AttackTimer {
    Idle,
    Attack(u16),
    Cooldown(u16),
}

struct Player<'a> {
    entity: Entity<'a>,
    facing: Tri,
    state: PlayerState,
    sprite_offset: u16,
    attack_timer: AttackTimer,
    damage_cooldown: u16,
    sword: SwordState,
    fudge_factor: Vector2D<i32>,
    hurtbox: Option<Rect<Number>>,
    controllable: bool,
}

impl<'a> Player<'a> {
    fn new(object_controller: &'a ObjectControl) -> Player {
        let mut entity = Entity::new(
            object_controller,
            Rect::new((0_u16, 0_u16).into(), (4_u16, 12_u16).into()),
        );
        entity
            .sprite
            .set_sprite_size(agb::display::object::Size::S16x16);
        entity.sprite.set_tile_id(0);
        entity.sprite.show();
        entity.position = (144, 0).into();
        entity.sprite.commit();

        Player {
            entity,
            facing: Tri::Positive,
            state: PlayerState::OnGround,
            sword: SwordState::LongSword,
            sprite_offset: 0,
            attack_timer: AttackTimer::Idle,
            fudge_factor: (0, 0).into(),
            hurtbox: None,
            damage_cooldown: 0,
            controllable: true,
        }
    }

    fn update(
        &mut self,
        buttons: &ButtonController,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        let mut instruction = UpdateInstruction::None;

        let x = if self.controllable {
            buttons.x_tri()
        } else {
            Tri::Zero
        };

        let b_press = buttons.is_just_pressed(Button::B) && self.controllable;
        let a_press = buttons.is_just_pressed(Button::A) && self.controllable;

        self.fudge_factor = (0, 0).into();
        let mut hurtbox = None;

        match self.state {
            PlayerState::OnGround => {
                self.entity.velocity.y = 0.into();
                self.entity.velocity.x = self.entity.velocity.x * 40 / 64;

                match &mut self.attack_timer {
                    AttackTimer::Idle => {
                        if x != Tri::Zero {
                            self.facing = x;
                        }
                        self.entity.sprite.set_hflip(self.facing == Tri::Negative);
                        self.entity.velocity.x += self.sword.ground_walk_force() * x as i32;
                        if self.entity.velocity.x.abs() > Number::new(1) / 10 {
                            self.entity
                                .sprite
                                .set_tile_id(self.sword.walk_animation(&mut self.sprite_offset));
                        } else {
                            self.entity
                                .sprite
                                .set_tile_id(self.sword.idle_animation(&mut self.sprite_offset));
                        }

                        if b_press && self.sword != SwordState::Swordless {
                            self.attack_timer = AttackTimer::Attack(self.sword.attack_duration());
                            sfx.sword();
                        } else if a_press {
                            self.entity.velocity.y -= self.sword.jump_impulse();
                            self.state = PlayerState::InAir;
                            self.sprite_offset = 0;

                            sfx.jump();
                        }
                    }
                    AttackTimer::Attack(a) => {
                        *a -= 1;
                        let frame = self.sword.attack_frame(*a);
                        self.fudge_factor.x = self.sword.fudge(frame) * self.facing as i32;
                        self.entity
                            .sprite
                            .set_tile_id(self.sword.to_sprite_id(frame));

                        hurtbox = self.sword.ground_attack_hurtbox(frame);

                        if *a == 0 {
                            self.attack_timer = AttackTimer::Cooldown(self.sword.cooldown_time());
                        }
                    }
                    AttackTimer::Cooldown(a) => {
                        *a -= 1;
                        let frame = self.sword.hold_frame();
                        self.fudge_factor.x = self.sword.fudge(frame) * self.facing as i32;
                        self.entity
                            .sprite
                            .set_tile_id(self.sword.to_sprite_id(frame));
                        if *a == 0 {
                            self.attack_timer = AttackTimer::Idle;
                        }
                    }
                }
            }
            PlayerState::InAir => {
                self.entity.velocity.x = self.entity.velocity.x * 63 / 64;

                match &mut self.attack_timer {
                    AttackTimer::Idle => {
                        let sprite = if self.sprite_offset < 3 * 4 {
                            self.sprite_offset / 4
                        } else if self.entity.velocity.y.abs() < Number::new(1) / 5 {
                            3
                        } else if self.entity.velocity.y > 1.into() {
                            5
                        } else if self.entity.velocity.y > 0.into() {
                            4
                        } else {
                            2
                        };
                        self.entity
                            .sprite
                            .set_tile_id((sprite + self.sword.jump_offset()) * 4);

                        if x != Tri::Zero {
                            self.facing = x;
                        }
                        self.entity.sprite.set_hflip(self.facing == Tri::Negative);
                        self.entity.velocity.x += self.sword.air_move_force() * x as i32;

                        if b_press
                            && self.sword != SwordState::LongSword
                            && self.sword != SwordState::Swordless
                        {
                            sfx.sword();
                            self.attack_timer =
                                AttackTimer::Attack(self.sword.jump_attack_duration());
                        }
                    }
                    AttackTimer::Attack(a) => {
                        *a -= 1;
                        let frame = self.sword.jump_attack_frame(*a);
                        self.entity
                            .sprite
                            .set_tile_id(self.sword.to_jump_sprite_id(frame));

                        hurtbox = self.sword.air_attack_hurtbox(frame);

                        if *a == 0 {
                            self.attack_timer = AttackTimer::Idle;
                        }
                    }
                    AttackTimer::Cooldown(_) => {
                        self.attack_timer = AttackTimer::Idle;
                    }
                }
            }
        }
        let gravity: Number = 1.into();
        let gravity = gravity / 16;
        self.entity.velocity.y += gravity;

        let fudge_number = (self.fudge_factor.x, self.fudge_factor.y).into();

        // convert the hurtbox to a location in the game
        self.hurtbox = hurtbox.map(|h| {
            let mut b = Rect::new(h.position - (8, 8).into(), h.size);
            if self.facing == Tri::Negative {
                b.position.x = -b.position.x - b.size.x;
            }
            b.position += self.entity.position + fudge_number;
            b
        });

        let prior_y_velocity = self.entity.velocity.y;
        self.entity.update_position(level);
        let (_, collided_down) = self
            .entity
            .collision_in_direction((0, 1).into(), 1.into(), |v| level.collides(v));

        if collided_down {
            if self.state == PlayerState::InAir && prior_y_velocity > 2.into() {
                instruction = UpdateInstruction::CreateParticle(
                    ParticleData::new_dust(),
                    self.entity.position + (2 * self.facing as i32, 0).into(),
                );

                sfx.player_land();
            }

            self.state = PlayerState::OnGround;
        } else {
            self.state = PlayerState::InAir;
        }

        if self.damage_cooldown > 0 {
            self.damage_cooldown -= 1;
        }

        self.sprite_offset += 1;

        instruction
    }

    // retuns true if the player is alive and false otherwise
    fn damage(&mut self) -> (bool, bool) {
        if self.damage_cooldown != 0 {
            return (true, false);
        }

        self.damage_cooldown = 120;
        let new_sword = match self.sword {
            SwordState::LongSword => Some(SwordState::ShortSword),
            SwordState::ShortSword => Some(SwordState::Dagger),
            SwordState::Dagger => None,
            SwordState::Swordless => Some(SwordState::Swordless),
        };
        if let Some(sword) = new_sword {
            self.sword = sword;
            (true, true)
        } else {
            (false, true)
        }
    }

    fn heal(&mut self) {
        let new_sword = match self.sword {
            SwordState::LongSword => None,
            SwordState::ShortSword => Some(SwordState::LongSword),
            SwordState::Dagger => Some(SwordState::ShortSword),
            SwordState::Swordless => Some(SwordState::Swordless),
        };

        if let Some(sword) = new_sword {
            self.sword = sword;
        }

        self.damage_cooldown = 30;
    }

    fn commit(&mut self, offset: Vector2D<Number>) {
        self.entity.commit_with_fudge(offset, self.fudge_factor);
    }
}

enum EnemyData {
    Slime(SlimeData),
    Bat(BatData),
    MiniFlame(MiniFlameData),
    Emu(EmuData),
}

struct BatData {
    sprite_offset: u16,
    bat_state: BatState,
}

enum BatState {
    Idle,
    Chasing(u16),
    Dead,
}

struct SlimeData {
    sprite_offset: u16,
    slime_state: SlimeState,
}

impl BatData {
    fn new() -> Self {
        Self {
            sprite_offset: 0,
            bat_state: BatState::Idle,
        }
    }

    fn update(
        &mut self,
        entity: &mut Entity,
        player: &Player,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        let mut instruction = UpdateInstruction::None;
        let should_die = player
            .hurtbox
            .as_ref()
            .map(|hurtbox| hurtbox.touches(entity.collider()))
            .unwrap_or(false);
        let should_damage = entity.collider().touches(player.entity.collider());

        match &mut self.bat_state {
            BatState::Idle => {
                self.sprite_offset += 1;
                if self.sprite_offset >= 9 * 8 {
                    self.sprite_offset = 0;
                }

                if self.sprite_offset == 8 * 5 {
                    sfx.bat_flap();
                }

                entity.sprite.set_tile_id((78 + self.sprite_offset / 8) * 4);

                if (entity.position - player.entity.position).manhattan_distance() < 50.into() {
                    self.bat_state = BatState::Chasing(300);
                    self.sprite_offset /= 4;
                }

                if should_die {
                    self.bat_state = BatState::Dead;
                    sfx.bat_death();
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }

                entity.velocity *= Number::new(15) / 16;
                entity.update_position(level);
            }
            BatState::Chasing(count) => {
                self.sprite_offset += 1;

                let speed = Number::new(1) / Number::new(4);
                let target_velocity = player.entity.position - entity.position;
                if target_velocity.manhattan_distance() > 1.into() {
                    entity.velocity = target_velocity.normalise() * speed;
                } else {
                    entity.velocity = (0, 0).into();
                }

                if self.sprite_offset >= 9 * 2 {
                    self.sprite_offset = 0;
                }
                entity.sprite.set_tile_id((78 + self.sprite_offset / 2) * 4);

                if self.sprite_offset == 2 * 5 {
                    sfx.bat_flap();
                }

                entity.update_position(level);

                if *count == 0 {
                    self.bat_state = BatState::Idle;
                    self.sprite_offset *= 4;
                } else {
                    *count -= 1;
                }

                if should_die {
                    self.bat_state = BatState::Dead;
                    sfx.bat_death();
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }
            }
            BatState::Dead => {
                entity.sprite.set_tile_id(87 * 4);
                let gravity: Number = 1.into();
                let gravity = gravity / 16;
                entity.velocity.x = 0.into();

                entity.velocity.y += gravity;

                let original_y_velocity = entity.velocity.y;
                let move_amount = entity.update_position(level);

                let just_landed = move_amount.y != 0.into() && original_y_velocity != move_amount.y;

                if just_landed {
                    instruction = UpdateInstruction::CreateParticle(
                        ParticleData::new_health(),
                        entity.position,
                    );
                }
            }
        }
        instruction
    }
}

enum SlimeState {
    Idle,
    Chasing(Tri),
    Dead(u16),
}

impl SlimeData {
    fn new() -> Self {
        Self {
            sprite_offset: 0,
            slime_state: SlimeState::Idle,
        }
    }

    fn update(
        &mut self,
        entity: &mut Entity,
        player: &Player,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        let mut instruction = UpdateInstruction::None;

        let should_die = player
            .hurtbox
            .as_ref()
            .map(|h| h.touches(entity.collider()))
            .unwrap_or(false);
        let should_damage = entity.collider().touches(player.entity.collider());

        match &mut self.slime_state {
            SlimeState::Idle => {
                self.sprite_offset += 1;
                if self.sprite_offset >= 32 {
                    self.sprite_offset = 0;
                }

                entity
                    .sprite
                    .set_tile_id((29 + self.sprite_offset / 16) * 4);

                if (player.entity.position - entity.position).manhattan_distance() < 40.into() {
                    let direction = if player.entity.position.x > entity.position.x {
                        Tri::Positive
                    } else if player.entity.position.x < entity.position.x {
                        Tri::Negative
                    } else {
                        Tri::Zero
                    };

                    self.slime_state = SlimeState::Chasing(direction);
                    self.sprite_offset = 0;
                }
                if should_die {
                    self.slime_state = SlimeState::Dead(0);
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer
                }

                let gravity: Number = 1.into();
                let gravity = gravity / 16;
                entity.velocity.y += gravity;
                entity.velocity *= Number::new(15) / 16;
                entity.update_position(level);
            }
            SlimeState::Chasing(direction) => {
                self.sprite_offset += 1;
                if self.sprite_offset >= 7 * 6 {
                    self.slime_state = SlimeState::Idle;
                } else {
                    let frame = ping_pong(self.sprite_offset / 6, 5);

                    if frame == 0 {
                        sfx.slime_boing();
                    }

                    entity.sprite.set_tile_id((frame + 31) * 4);

                    entity.velocity.x = match frame {
                        2 | 3 | 4 => (Number::new(1) / 5) * Number::new(*direction as i32),
                        _ => 0.into(),
                    };

                    let gravity: Number = 1.into();
                    let gravity = gravity / 16;
                    entity.velocity.y += gravity;

                    let updated_position = entity.update_position(level);
                    if updated_position.y > 0.into() && self.sprite_offset > 2 * 6 {
                        // we're falling
                        self.sprite_offset = 6 * 6;
                    }
                }
                if should_die {
                    self.slime_state = SlimeState::Dead(0);
                    sfx.slime_dead();
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer
                }
            }
            SlimeState::Dead(count) => {
                if *count < 5 * 4 {
                    entity.sprite.set_tile_id((36 + *count / 4) * 4);
                    *count += 1;
                } else {
                    return UpdateInstruction::Remove;
                }
            }
        }
        instruction
    }
}

enum MiniFlameState {
    Idle(u16),
    Chasing(u16),
    Dead,
}

struct MiniFlameData {
    state: MiniFlameState,
    sprite_offset: u16,
}

impl MiniFlameData {
    fn new() -> Self {
        Self {
            state: MiniFlameState::Chasing(90),
            sprite_offset: 0,
        }
    }

    fn update(
        &mut self,
        entity: &mut Entity,
        player: &Player,
        _level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        let mut instruction = UpdateInstruction::None;

        let should_die = player
            .hurtbox
            .as_ref()
            .map(|h| h.touches(entity.collider()))
            .unwrap_or(false);
        let should_damage = entity.collider().touches(player.entity.collider());

        self.sprite_offset += 1;

        match &mut self.state {
            MiniFlameState::Idle(frames) => {
                *frames -= 1;

                if *frames == 0 {
                    let resulting_direction = player.entity.position - entity.position;
                    if resulting_direction.manhattan_distance() < 1.into() {
                        self.state = MiniFlameState::Idle(30);
                    } else {
                        sfx.flame_charge();
                        self.state = MiniFlameState::Chasing(90);
                        entity.velocity = resulting_direction.normalise() * Number::new(2);
                    }
                } else {
                    if self.sprite_offset >= 12 * 8 {
                        self.sprite_offset = 0;
                    }

                    entity
                        .sprite
                        .set_tile_id((137 + self.sprite_offset / 8) * 4);

                    entity.velocity = (0.into(), Number::new(-1) / Number::new(4)).into();
                }

                if should_die {
                    self.sprite_offset = 0;
                    self.state = MiniFlameState::Dead;

                    if get_random() % 4 == 0 {
                        instruction = UpdateInstruction::CreateParticle(
                            ParticleData::new_health(),
                            entity.position,
                        );
                    }
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }
            }
            MiniFlameState::Chasing(frame) => {
                entity.velocity *= Number::new(63) / Number::new(64);

                if *frame == 0 {
                    self.state = MiniFlameState::Idle(30);
                } else {
                    *frame -= 1;
                }

                if should_die {
                    self.sprite_offset = 0;
                    self.state = MiniFlameState::Dead;

                    if get_random() % 4 == 0 {
                        instruction = UpdateInstruction::CreateParticle(
                            ParticleData::new_health(),
                            entity.position,
                        );
                    }
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }

                if self.sprite_offset >= 12 * 2 {
                    self.sprite_offset = 0;
                }

                if entity.velocity.manhattan_distance() < Number::new(1) / Number::new(4) {
                    self.state = MiniFlameState::Idle(90);
                }

                entity
                    .sprite
                    .set_tile_id((137 + self.sprite_offset / 2) * 4);
            }
            MiniFlameState::Dead => {
                entity.velocity = (0, 0).into();
                if self.sprite_offset >= 6 * 12 {
                    instruction = UpdateInstruction::Remove;
                }

                entity
                    .sprite
                    .set_tile_id((148 + self.sprite_offset / 12) * 4);

                self.sprite_offset += 1;
            }
        };

        entity.update_position_without_collision();

        instruction
    }
}

enum EmuState {
    Idle,
    Charging(Tri),
    Knockback,
    Dead,
}

struct EmuData {
    state: EmuState,
    sprite_offset: u16,
}

impl EmuData {
    fn new() -> Self {
        Self {
            state: EmuState::Idle,
            sprite_offset: 0,
        }
    }

    fn update(
        &mut self,
        entity: &mut Entity,
        player: &Player,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        let mut instruction = UpdateInstruction::None;

        let should_die = player
            .hurtbox
            .as_ref()
            .map(|h| h.touches(entity.collider()))
            .unwrap_or(false);
        let should_damage = entity.collider().touches(player.entity.collider());

        match &mut self.state {
            EmuState::Idle => {
                self.sprite_offset += 1;

                if self.sprite_offset >= 3 * 16 {
                    self.sprite_offset = 0;
                }

                entity
                    .sprite
                    .set_tile_id((170 + self.sprite_offset / 16) * 4);

                if (entity.position.y - player.entity.position.y).abs() < 10.into() {
                    let velocity = Number::new(1)
                        * (player.entity.position.x - entity.position.x)
                            .to_raw()
                            .signum();
                    entity.velocity.x = velocity;

                    if velocity > 0.into() {
                        entity.sprite.set_hflip(true);
                        self.state = EmuState::Charging(Tri::Positive);
                    } else if velocity < 0.into() {
                        self.state = EmuState::Charging(Tri::Negative);
                        entity.sprite.set_hflip(false);
                    } else {
                        self.state = EmuState::Idle;
                    }
                }

                if should_die {
                    self.sprite_offset = 0;
                    self.state = EmuState::Dead;
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }
            }
            EmuState::Charging(direction) => {
                let direction = Number::new(*direction as i32);
                self.sprite_offset += 1;

                if self.sprite_offset >= 4 * 2 {
                    self.sprite_offset = 0;
                }

                if self.sprite_offset == 2 * 2 {
                    sfx.emu_step();
                }

                entity
                    .sprite
                    .set_tile_id((173 + self.sprite_offset / 2) * 4);

                let gravity: Number = 1.into();
                let gravity = gravity / 16;
                entity.velocity.y += gravity;

                let distance_travelled = entity.update_position(level);

                if distance_travelled.x == 0.into() {
                    sfx.emu_crash();
                    self.state = EmuState::Knockback;
                    entity.velocity = (-direction / 2, Number::new(-1)).into();
                }

                if should_die {
                    self.sprite_offset = 0;
                    self.state = EmuState::Dead;
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }
            }
            EmuState::Knockback => {
                let gravity: Number = 1.into();
                let gravity = gravity / 16;
                entity.velocity.y += gravity;

                entity.update_position(level);
                let (_, is_collision) =
                    entity.collision_in_direction((0, 1).into(), gravity, |x| level.collides(x));

                if is_collision {
                    entity.velocity.x = 0.into();
                    self.state = EmuState::Idle;
                }

                if should_die {
                    self.sprite_offset = 0;
                    self.state = EmuState::Dead;
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }
            }
            EmuState::Dead => {
                if self.sprite_offset == 0 {
                    sfx.emu_death();
                }

                if self.sprite_offset >= 8 * 4 {
                    instruction = UpdateInstruction::Remove;
                }

                entity
                    .sprite
                    .set_tile_id((177 + self.sprite_offset / 4) * 4);
                self.sprite_offset += 1;
            }
        }

        instruction
    }
}

enum UpdateInstruction {
    None,
    HealBossAndRemove,
    HealPlayerAndRemove,
    Remove,
    DamagePlayer,
    CreateParticle(ParticleData, Vector2D<Number>),
}

impl EnemyData {
    fn collision_mask(&self) -> Rect<u16> {
        match self {
            EnemyData::Slime(_) => Rect::new((0u16, 0u16).into(), (4u16, 11u16).into()),
            EnemyData::Bat(_) => Rect::new((0u16, 0u16).into(), (12u16, 4u16).into()),
            EnemyData::MiniFlame(_) => Rect::new((0u16, 0u16).into(), (12u16, 12u16).into()),
            EnemyData::Emu(_) => Rect::new((0u16, 0u16).into(), (7u16, 11u16).into()),
        }
    }

    fn tile_id(&self) -> u16 {
        match self {
            EnemyData::Slime(_) => 29,
            EnemyData::Bat(_) => 78,
            EnemyData::MiniFlame(_) => 137,
            EnemyData::Emu(_) => 170,
        }
    }

    fn update(
        &mut self,
        entity: &mut Entity,
        player: &Player,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        match self {
            EnemyData::Slime(data) => data.update(entity, player, level, sfx),
            EnemyData::Bat(data) => data.update(entity, player, level, sfx),
            EnemyData::MiniFlame(data) => data.update(entity, player, level, sfx),
            EnemyData::Emu(data) => data.update(entity, player, level, sfx),
        }
    }
}

struct Enemy<'a> {
    entity: Entity<'a>,
    enemy_data: EnemyData,
}

impl<'a> Enemy<'a> {
    fn new(object_controller: &'a ObjectControl, enemy_data: EnemyData) -> Self {
        let mut entity = Entity::new(object_controller, enemy_data.collision_mask());

        entity
            .sprite
            .set_sprite_size(agb::display::object::Size::S16x16);
        entity.sprite.set_tile_id(enemy_data.tile_id());
        entity.sprite.show();

        entity.sprite.commit();

        Self { entity, enemy_data }
    }

    fn update(&mut self, player: &Player, level: &Level, sfx: &mut sfx::Sfx) -> UpdateInstruction {
        self.enemy_data.update(&mut self.entity, player, level, sfx)
    }
}

enum ParticleData {
    Dust(u16),
    Health(u16),
    BossHealer(u16, Vector2D<Number>),
}

impl ParticleData {
    fn new_dust() -> Self {
        Self::Dust(0)
    }

    fn new_health() -> Self {
        Self::Health(0)
    }

    fn new_boss_healer(target: Vector2D<Number>) -> Self {
        Self::BossHealer(0, target)
    }

    fn tile_id(&self) -> u16 {
        match self {
            ParticleData::Dust(_) => 70,
            ParticleData::Health(_) => 88,
            ParticleData::BossHealer(_, _) => 88,
        }
    }

    fn update(
        &mut self,
        entity: &mut Entity,
        player: &Player,
        _level: &Level,
    ) -> UpdateInstruction {
        match self {
            ParticleData::Dust(frame) => {
                if *frame == 8 * 3 {
                    return UpdateInstruction::Remove;
                }

                entity.sprite.set_tile_id((70 + *frame / 3) * 4);

                *frame += 1;
                UpdateInstruction::None
            }
            ParticleData::Health(frame) => {
                if *frame > 8 * 3 * 6 {
                    return UpdateInstruction::Remove; // have played the animation 6 times
                }

                entity.sprite.set_tile_id((88 + (*frame / 3) % 8) * 4);

                if *frame < 8 * 3 * 3 {
                    entity.velocity.y = Number::new(-1) / 2;
                } else {
                    let speed = Number::new(2);
                    let target_velocity = player.entity.position - entity.position;

                    if target_velocity.manhattan_distance() < 5.into() {
                        return UpdateInstruction::HealPlayerAndRemove;
                    }

                    entity.velocity = target_velocity.normalise() * speed;
                }

                entity.update_position_without_collision();

                *frame += 1;

                UpdateInstruction::None
            }
            ParticleData::BossHealer(frame, target) => {
                entity.sprite.set_tile_id((88 + (*frame / 3) % 8) * 4);

                if *frame < 8 * 3 * 3 {
                    entity.velocity.y = Number::new(-1) / 2;
                } else if *frame < 8 * 3 * 6 {
                    entity.velocity = (0, 0).into();
                } else {
                    let speed = Number::new(4);
                    let target_velocity = *target - entity.position;

                    if target_velocity.manhattan_distance() < 5.into() {
                        return UpdateInstruction::HealBossAndRemove;
                    }

                    entity.velocity = target_velocity.normalise() * speed;
                }

                entity.update_position_without_collision();

                *frame += 1;
                UpdateInstruction::None
            }
        }
    }
}

struct Particle<'a> {
    entity: Entity<'a>,
    particle_data: ParticleData,
}

impl<'a> Particle<'a> {
    fn new(
        object_controller: &'a ObjectControl,
        particle_data: ParticleData,
        position: Vector2D<Number>,
    ) -> Self {
        let mut entity = Entity::new(
            object_controller,
            Rect::new((0u16, 0u16).into(), (0u16, 0u16).into()),
        );

        entity
            .sprite
            .set_sprite_size(agb::display::object::Size::S16x16);
        entity.sprite.set_tile_id(particle_data.tile_id() * 4);
        entity.sprite.show();
        entity.position = position;

        Self {
            entity,
            particle_data,
        }
    }

    fn update(&mut self, player: &Player, level: &Level) -> UpdateInstruction {
        self.particle_data.update(&mut self.entity, player, level)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum GameStatus {
    Continue,
    Lost,
    RespawnAtBoss,
}

enum BossState<'a> {
    NotSpawned,
    Active(Boss<'a>),
    Following(FollowingBoss<'a>),
}

impl<'a> BossState<'a> {
    fn update(
        &mut self,
        enemies: &mut Arena<Enemy<'a>>,
        object_controller: &'a ObjectControl,
        player: &Player,
        sfx: &mut sfx::Sfx,
    ) -> BossInstruction {
        match self {
            BossState::Active(boss) => boss.update(enemies, object_controller, player, sfx),
            BossState::Following(boss) => {
                boss.update(player);
                BossInstruction::None
            }
            BossState::NotSpawned => BossInstruction::None,
        }
    }
    fn commit(&mut self, offset: Vector2D<Number>) {
        match self {
            BossState::Active(boss) => {
                boss.commit(offset);
            }
            BossState::Following(boss) => {
                boss.commit(offset);
            }
            BossState::NotSpawned => {}
        }
    }
}

struct FollowingBoss<'a> {
    entity: Entity<'a>,
    following: bool,
    to_hole: bool,
    timer: u32,
    gone: bool,
}

impl<'a> FollowingBoss<'a> {
    fn new(object_controller: &'a ObjectControl, position: Vector2D<Number>) -> Self {
        let mut entity = Entity::new(
            object_controller,
            Rect::new((0_u16, 0_u16).into(), (0_u16, 0_u16).into()),
        );
        entity.position = position;
        entity
            .sprite
            .set_sprite_size(agb::display::object::Size::S16x16);
        Self {
            entity,
            following: true,
            timer: 0,
            to_hole: false,
            gone: false,
        }
    }
    fn update(&mut self, player: &Player) {
        let difference = player.entity.position - self.entity.position;
        self.timer += 1;

        if self.to_hole {
            let target: Vector2D<Number> = (17 * 8, -3 * 8).into();
            let difference = target - self.entity.position;
            if difference.manhattan_distance() < 1.into() {
                self.gone = true;
            } else {
                self.entity.velocity = difference.normalise() * 2;
            }

            let frame = (self.timer / 8) % 12;
            self.entity.sprite.set_tile_id((125 + frame as u16) * 4)
        } else if self.timer < 120 {
            let frame = (self.timer / 20) % 12;
            self.entity.sprite.set_tile_id((125 + frame as u16) * 4)
        } else if self.following {
            self.entity.velocity = difference / 16;
            if difference.manhattan_distance() < 20.into() {
                self.following = false;
            }
            let frame = (self.timer / 8) % 12;
            self.entity.sprite.set_tile_id((125 + frame as u16) * 4)
        } else {
            self.entity.velocity = (0, 0).into();
            if difference.manhattan_distance() > 60.into() {
                self.following = true;
            }
            let frame = (self.timer / 16) % 12;
            self.entity.sprite.set_tile_id((125 + frame as u16) * 4)
        }
        self.entity.update_position_without_collision();
    }

    fn commit(&mut self, offset: Vector2D<Number>) {
        self.entity.commit_with_fudge(offset, (0, 0).into());
    }
}

enum BossActiveState {
    Damaged(u8),
    MovingToTarget,
    WaitingUntilExplosion(u8),
    WaitingUntilDamaged(u16),
    WaitUntilKilled,
}

struct Boss<'a> {
    entity: Entity<'a>,
    health: u8,
    target_location: u8,
    state: BossActiveState,
    timer: u32,
    screen_coords: Vector2D<Number>,
    shake_magnitude: Number,
}

enum BossInstruction {
    None,
    Dead,
}

impl<'a> Boss<'a> {
    fn new(object_controller: &'a ObjectControl, screen_coords: Vector2D<Number>) -> Self {
        let mut entity = Entity::new(
            object_controller,
            Rect::new((0_u16, 0_u16).into(), (28_u16, 28_u16).into()),
        );
        entity
            .sprite
            .set_sprite_size(agb::display::object::Size::S32x32);
        entity.sprite.set_palette(1);
        entity.position = screen_coords + (144, 136).into();
        Self {
            entity,
            health: 5,
            target_location: get_random().rem_euclid(5) as u8,
            state: BossActiveState::Damaged(60),
            timer: 0,
            screen_coords,
            shake_magnitude: 0.into(),
        }
    }
    fn update(
        &mut self,
        enemies: &mut Arena<Enemy<'a>>,
        object_controller: &'a ObjectControl,
        player: &Player,
        sfx: &mut sfx::Sfx,
    ) -> BossInstruction {
        let mut instruction = BossInstruction::None;
        match &mut self.state {
            BossActiveState::Damaged(time) => {
                *time -= 1;
                if *time == 0 {
                    self.target_location = self.get_next_target_location();
                    self.state = BossActiveState::MovingToTarget;
                    sfx.boss_move();
                }
            }
            BossActiveState::MovingToTarget => {
                let target = self.get_target_location() + self.screen_coords;
                let difference = target - self.entity.position;
                if difference.manhattan_distance() < 1.into() {
                    self.entity.velocity = (0, 0).into();
                    self.state = BossActiveState::WaitingUntilExplosion(60);
                } else {
                    self.entity.velocity = difference / 16;
                }
            }
            BossActiveState::WaitingUntilExplosion(time) => {
                *time -= 1;
                if *time == 0 {
                    if self.health == 0 {
                        enemies.clear();
                        instruction = BossInstruction::Dead;
                        self.state = BossActiveState::WaitUntilKilled;
                    } else {
                        sfx.burning();
                        self.explode(enemies, object_controller);
                        self.state = BossActiveState::WaitingUntilDamaged(60 * 5);
                    }
                }
            }
            BossActiveState::WaitingUntilDamaged(time) => {
                *time -= 1;
                if *time == 0 {
                    sfx.burning();
                    self.explode(enemies, object_controller);
                    self.state = BossActiveState::WaitingUntilDamaged(60 * 5);
                }
                if let Some(hurt) = &player.hurtbox {
                    if hurt.touches(self.entity.collider()) {
                        self.health -= 1;
                        self.state = BossActiveState::Damaged(30);
                    }
                }
            }
            BossActiveState::WaitUntilKilled => {}
        }
        let animation_rate = match self.state {
            BossActiveState::Damaged(_) => 6,
            BossActiveState::MovingToTarget => 4,
            BossActiveState::WaitingUntilExplosion(_) => 3,
            BossActiveState::WaitingUntilDamaged(_) => 8,
            BossActiveState::WaitUntilKilled => 12,
        };

        self.shake_magnitude = match self.state {
            BossActiveState::Damaged(_) => 1.into(),
            BossActiveState::MovingToTarget => 0.into(),
            BossActiveState::WaitingUntilExplosion(_) => 5.into(),
            BossActiveState::WaitingUntilDamaged(time) => {
                if time < 60 {
                    5.into()
                } else {
                    0.into()
                }
            }
            BossActiveState::WaitUntilKilled => 3.into(),
        };
        self.timer += 1;
        let frame = (self.timer / animation_rate) % 12;
        self.entity.sprite.set_tile_id(784 + (frame as u16) * 16);

        self.entity.update_position_without_collision();
        instruction
    }
    fn commit(&mut self, offset: Vector2D<Number>) {
        let shake = if self.shake_magnitude != 0.into() {
            (
                Number::from_raw(get_random()).rem_euclid(self.shake_magnitude)
                    - self.shake_magnitude / 2,
                Number::from_raw(get_random()).rem_euclid(self.shake_magnitude)
                    - self.shake_magnitude / 2,
            )
                .into()
        } else {
            (0, 0).into()
        };

        self.entity
            .commit_with_size(offset + shake, (32, 32).into());
    }
    fn explode(&self, enemies: &mut Arena<Enemy<'a>>, object_controller: &'a ObjectControl) {
        for _ in 0..(6 - self.health) {
            let x_offset: Number = Number::from_raw(get_random()).rem_euclid(2.into()) - 1;
            let y_offset: Number = Number::from_raw(get_random()).rem_euclid(2.into()) - 1;
            let mut flame = Enemy::new(
                object_controller,
                EnemyData::MiniFlame(MiniFlameData::new()),
            );
            flame.entity.position = self.entity.position;
            flame.entity.velocity = (x_offset, y_offset).into();
            enemies.insert(flame);
        }
    }

    fn get_next_target_location(&self) -> u8 {
        loop {
            let a = get_random().rem_euclid(5) as u8;
            if a != self.target_location {
                break a;
            }
        }
    }
    fn get_target_location(&self) -> Vector2D<Number> {
        match self.target_location {
            0 => (240 / 4, 160 / 4).into(),
            1 => (3 * 240 / 4, 160 / 4).into(),
            2 => (240 / 4, 3 * 160 / 4).into(),
            3 => (3 * 240 / 4, 3 * 160 / 4).into(),
            4 => (240 / 2, 160 / 2).into(),
            _ => unreachable!(),
        }
    }
}

struct Game<'a> {
    player: Player<'a>,
    input: ButtonController,
    frame_count: u32,
    level: Level,
    offset: Vector2D<Number>,
    shake_time: u16,
    sunrise_timer: u16,

    enemies: Arena<Enemy<'a>>,
    particles: Arena<Particle<'a>>,
    slime_load: usize,
    bat_load: usize,
    emu_load: usize,
    boss: BossState<'a>,
    move_state: MoveState,
    fade_count: u16,

    background_distributor: &'a mut BackgroundDistributor,
}

enum MoveState {
    Advancing,
    PinnedAtEnd,
    FollowingPlayer,
    Ending,
}

impl<'a> Game<'a> {
    fn has_just_reached_end(&self) -> bool {
        match self.boss {
            BossState::NotSpawned => self.offset.x.floor() + 248 >= tilemap::WIDTH as i32 * 8,
            _ => false,
        }
    }

    fn advance_frame(
        &mut self,
        object_controller: &'a ObjectControl,
        sfx: &mut sfx::Sfx,
    ) -> GameStatus {
        let mut state = GameStatus::Continue;

        match self.move_state {
            MoveState::Advancing => {
                self.offset += Into::<Vector2D<Number>>::into((1, 0)) / 8;

                if self.has_just_reached_end() {
                    sfx.boss();
                    self.offset.x = (tilemap::WIDTH as i32 * 8 - 248).into();
                    self.move_state = MoveState::PinnedAtEnd;
                    self.boss = BossState::Active(Boss::new(object_controller, self.offset))
                }
            }
            MoveState::PinnedAtEnd => {
                self.offset.x = (tilemap::WIDTH as i32 * 8 - 248).into();
            }
            MoveState::FollowingPlayer => {
                Game::update_sunrise(self.background_distributor, self.sunrise_timer);
                if self.sunrise_timer < 120 {
                    self.sunrise_timer += 1;
                } else {
                    let difference = self.player.entity.position.x - (self.offset.x + WIDTH / 2);

                    self.offset.x += difference / 8;
                    if self.offset.x > (tilemap::WIDTH as i32 * 8 - 248).into() {
                        self.offset.x = (tilemap::WIDTH as i32 * 8 - 248).into();
                    } else if self.offset.x < 8.into() {
                        self.offset.x = 8.into();
                        self.move_state = MoveState::Ending;
                    }
                }
            }
            MoveState::Ending => {
                self.player.controllable = false;
                if let BossState::Following(boss) = &mut self.boss {
                    boss.to_hole = true;
                    if boss.gone {
                        self.fade_count += 1;
                        self.fade_count = self.fade_count.min(600);
                        Game::update_fade_out(self.background_distributor, self.fade_count);
                    }
                }
            }
        }

        match self
            .boss
            .update(&mut self.enemies, object_controller, &self.player, sfx)
        {
            BossInstruction::Dead => {
                let boss = match &self.boss {
                    BossState::Active(b) => b,
                    _ => unreachable!(),
                };
                let new_particle = Particle::new(
                    object_controller,
                    ParticleData::new_boss_healer(boss.entity.position),
                    self.player.entity.position,
                );
                self.particles.insert(new_particle);
                sfx.stop_music();
                self.player.sword = SwordState::Swordless;
            }
            BossInstruction::None => {}
        }

        self.load_enemies(object_controller);

        if self.player.entity.position.x < self.offset.x - 8 {
            let (alive, damaged) = self.player.damage();
            if !alive {
                state = GameStatus::Lost;
            }
            if damaged {
                sfx.player_hurt();
                self.shake_time += 20;
            }
        }

        let mut this_frame_offset = self.offset;
        if self.shake_time > 0 {
            let size = self.shake_time.min(4) as i32;
            let offset: Vector2D<Number> = (
                Number::from_raw(get_random()) % size - Number::new(size) / 2,
                Number::from_raw(get_random()) % size - Number::new(size) / 2,
            )
                .into();
            this_frame_offset += offset;
            self.shake_time -= 1;
        }

        self.input.update();
        match self.player.update(&self.input, &self.level, sfx) {
            UpdateInstruction::CreateParticle(data, position) => {
                let new_particle = Particle::new(object_controller, data, position);

                self.particles.insert(new_particle);
            }
            _ => {}
        }

        let mut remove = Vec::with_capacity(10);
        for (idx, enemy) in self.enemies.iter_mut() {
            if enemy.entity.position.x < self.offset.x - 8 {
                remove.push(idx);
                continue;
            }

            match enemy.update(&self.player, &self.level, sfx) {
                UpdateInstruction::Remove => {
                    remove.push(idx);
                }
                UpdateInstruction::HealPlayerAndRemove => {
                    self.player.heal();
                    sfx.player_heal();
                    remove.push(idx);
                }
                UpdateInstruction::HealBossAndRemove => {}
                UpdateInstruction::DamagePlayer => {
                    let (alive, damaged) = self.player.damage();
                    if !alive {
                        state = GameStatus::Lost;
                    }
                    if damaged {
                        sfx.player_hurt();
                        self.shake_time += 20;
                    }
                }
                UpdateInstruction::CreateParticle(data, position) => {
                    let new_particle = Particle::new(object_controller, data, position);
                    self.particles.insert(new_particle);
                }
                UpdateInstruction::None => {}
            }
            enemy
                .entity
                .commit_with_fudge(this_frame_offset, (0, 0).into());
        }

        self.player.commit(this_frame_offset);
        self.boss.commit(this_frame_offset);

        self.level
            .background
            .set_position(this_frame_offset.floor());
        self.level
            .foreground
            .set_position(this_frame_offset.floor());
        self.level
            .clouds
            .set_position(this_frame_offset.floor() / 4);
        self.level.background.commit();
        self.level.foreground.commit();
        self.level.clouds.commit();

        for i in remove {
            self.enemies.remove(i);
        }

        let mut remove = Vec::with_capacity(10);

        for (idx, particle) in self.particles.iter_mut() {
            match particle.update(&self.player, &self.level) {
                UpdateInstruction::Remove => remove.push(idx),
                UpdateInstruction::HealBossAndRemove => {
                    sfx.sunrise();
                    let location = match &self.boss {
                        BossState::Active(b) => b.entity.position,
                        _ => unreachable!(),
                    };
                    self.boss =
                        BossState::Following(FollowingBoss::new(object_controller, location));
                    self.move_state = MoveState::FollowingPlayer;
                    remove.push(idx);
                }
                UpdateInstruction::HealPlayerAndRemove => {
                    self.player.heal();
                    sfx.player_heal();
                    remove.push(idx);
                }
                UpdateInstruction::DamagePlayer => {
                    let (alive, damaged) = self.player.damage();
                    if !alive {
                        state = GameStatus::Lost;
                    }
                    if damaged {
                        sfx.player_hurt();
                        self.shake_time += 20;
                    }
                }
                UpdateInstruction::CreateParticle(_, _) => {}
                UpdateInstruction::None => {}
            }
            particle
                .entity
                .commit_with_fudge(this_frame_offset, (0, 0).into());
        }

        for i in remove {
            self.particles.remove(i);
        }

        self.frame_count += 1;
        if let GameStatus::Lost = state {
            match self.boss {
                BossState::Active(_) => GameStatus::RespawnAtBoss,
                _ => GameStatus::Lost,
            }
        } else {
            state
        }
    }

    fn load_enemies(&mut self, object_controller: &'a ObjectControl) {
        if self.slime_load < self.level.slime_spawns.len() {
            for (idx, slime_spawn) in self
                .level
                .slime_spawns
                .iter()
                .enumerate()
                .skip(self.slime_load)
            {
                if slime_spawn.0 as i32 > self.offset.x.floor() + 300 {
                    break;
                }
                self.slime_load = idx + 1;
                let mut slime = Enemy::new(object_controller, EnemyData::Slime(SlimeData::new()));
                slime.entity.position = (slime_spawn.0 as i32, slime_spawn.1 as i32 - 7).into();
                self.enemies.insert(slime);
            }
        }
        if self.bat_load < self.level.bat_spawns.len() {
            for (idx, bat_spawn) in self.level.bat_spawns.iter().enumerate().skip(self.bat_load) {
                if bat_spawn.0 as i32 > self.offset.x.floor() + 300 {
                    break;
                }
                self.bat_load = idx + 1;
                let mut bat = Enemy::new(object_controller, EnemyData::Bat(BatData::new()));
                bat.entity.position = (bat_spawn.0 as i32, bat_spawn.1 as i32).into();
                self.enemies.insert(bat);
            }
        }
        if self.emu_load < self.level.emu_spawns.len() {
            for (idx, emu_spawn) in self.level.emu_spawns.iter().enumerate().skip(self.emu_load) {
                if emu_spawn.0 as i32 > self.offset.x.floor() + 300 {
                    break;
                }
                self.emu_load = idx + 1;
                let mut emu = Enemy::new(object_controller, EnemyData::Emu(EmuData::new()));
                emu.entity.position = (emu_spawn.0 as i32, emu_spawn.1 as i32 - 7).into();
                self.enemies.insert(emu);
            }
        }
    }

    fn update_sunrise(background_distributor: &'a mut BackgroundDistributor, time: u16) {
        let mut modified_palette = background::background.palettes[0].clone();

        let a = modified_palette.get_colour(0);
        let b = modified_palette.get_colour(1);

        modified_palette.update_colour(0, interpolate_colour(a, 17982, time, 120));
        modified_palette.update_colour(1, interpolate_colour(b, 22427, time, 120));

        let modified_palettes = [modified_palette];

        background_distributor.set_background_palettes(&modified_palettes);
    }

    fn update_fade_out(background_distributor: &'a mut BackgroundDistributor, time: u16) {
        let mut modified_palette = background::background.palettes[0].clone();

        let c = modified_palette.get_colour(2);

        modified_palette.update_colour(0, interpolate_colour(17982, 0x7FFF, time, 600));
        modified_palette.update_colour(1, interpolate_colour(22427, 0x7FFF, time, 600));
        modified_palette.update_colour(2, interpolate_colour(c, 0x7FFF, time, 600));

        let modified_palettes = [modified_palette];

        background_distributor.set_background_palettes(&modified_palettes);
    }

    fn new(
        object: &'a ObjectControl,
        level: Level,
        background_distributor: &'a mut BackgroundDistributor,
        start_at_boss: bool,
    ) -> Self {
        let mut player = Player::new(object);
        let mut offset = (8, 8).into();
        if start_at_boss {
            player.entity.position = (133 * 8, 10 * 8).into();
            offset = (130 * 8, 8).into();
        }
        Self {
            player,
            input: ButtonController::new(),
            frame_count: 0,
            level,
            offset,
            shake_time: 0,

            enemies: Arena::with_capacity(100),
            slime_load: 0,
            bat_load: 0,
            emu_load: 0,
            particles: Arena::with_capacity(30),
            boss: BossState::NotSpawned,
            move_state: MoveState::Advancing,
            sunrise_timer: 0,
            fade_count: 0,

            background_distributor,
        }
    }
}

fn game_with_level(gba: &mut agb::Gba) {
    {
        let object = gba.display.object.get();
        object.set_sprite_palettes(&[
            objects::objects.palettes[0].clone(),
            objects::boss.palettes[0].clone(),
        ]);
        object.set_sprite_tilemap(objects::objects.tiles);
        object.set_sprite_tilemap_at_idx(8192 - objects::boss.tiles.len(), objects::boss.tiles);
    }

    let vblank = agb::interrupt::VBlank::get();
    vblank.wait_for_vblank();

    let mut timers = gba.timers.timers();
    let mut mixer = gba.mixer.mixer(&mut timers.timer0);
    mixer.enable();

    let mut sfx = sfx::Sfx::new(&mut mixer);
    sfx.purple_night();

    let mut start_at_boss = false;

    loop {
        let mut background = gba.display.video.tiled0();
        background.set_background_palettes(background::background.palettes);
        background.set_background_tilemap(0, background::background.tiles);
        let mut object = gba.display.object.get();
        object.enable();

        let mut game = Game::new(
            &object,
            Level::load_level(
                background.get_regular().unwrap(),
                background.get_regular().unwrap(),
                background.get_regular().unwrap(),
            ),
            &mut background,
            start_at_boss,
        );

        start_at_boss = loop {
            sfx.frame();
            vblank.wait_for_vblank();
            sfx.after_vblank();
            match game.advance_frame(&object, &mut sfx) {
                GameStatus::Continue => {}
                GameStatus::Lost => {
                    break false;
                }
                GameStatus::RespawnAtBoss => {
                    break true;
                }
            }

            get_random(); // advance RNG to make it less predictable between runs
        }
    }
}

mod tilemap {
    include!(concat!(env!("OUT_DIR"), "/tilemap.rs"));
}

#[agb::entry]
fn main() -> ! {
    let mut gba = agb::Gba::new();

    loop {
        game_with_level(&mut gba);
    }
}

fn ping_pong(i: u16, n: u16) -> u16 {
    let cycle = 2 * (n - 1);
    let i = i % cycle;
    if i >= n {
        cycle - i
    } else {
        i
    }
}

fn interpolate_colour(initial: u16, destination: u16, time_so_far: u16, total_time: u16) -> u16 {
    const MASK: u16 = 0b11111;
    fn to_components(c: u16) -> [u16; 3] {
        [c & MASK, (c >> 5) & MASK, (c >> 10) & MASK]
    }

    let initial_rgb = to_components(initial);
    let destination_rgb = to_components(destination);
    let mut colour = 0;

    for (i, c) in initial_rgb
        .iter()
        .zip(destination_rgb)
        .map(|(a, b)| (b - a) * time_so_far / total_time + a)
        .enumerate()
    {
        colour |= (c & MASK) << (i * 5);
    }
    colour
}
