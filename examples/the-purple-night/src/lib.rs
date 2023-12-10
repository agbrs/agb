#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;

mod sfx;

use core::cmp::Ordering;

use alloc::{boxed::Box, vec::Vec};

use agb::{
    display::{
        object::{Graphics, OamManaged, Object, Sprite, Tag, TagMap},
        tiled::{InfiniteScrolledMap, RegularBackgroundSize, TileFormat, VRamManager},
        Priority, HEIGHT, WIDTH,
    },
    fixnum::{num, FixedNum, Rect, Vector2D},
    input::{Button, ButtonController, Tri},
    interrupt::VBlank,
    rng,
    sound::mixer::Frequency,
};
use generational_arena::Arena;
use sfx::Sfx;

static GRAPHICS: &Graphics = agb::include_aseprite!("gfx/objects.aseprite", "gfx/boss.aseprite");
static TAG_MAP: &TagMap = GRAPHICS.tags();

static LONG_SWORD_IDLE: &Tag = TAG_MAP.get("Idle - longsword");
static LONG_SWORD_WALK: &Tag = TAG_MAP.get("Walk - longsword");
static LONG_SWORD_JUMP: &Tag = TAG_MAP.get("Jump - longsword");
static LONG_SWORD_ATTACK: &Tag = TAG_MAP.get("Attack - longsword");
static LONG_SWORD_JUMP_ATTACK: &Tag = TAG_MAP.get("Jump attack - longsword");

static SHORT_SWORD_IDLE: &Tag = TAG_MAP.get("Idle - shortsword");
static SHORT_SWORD_WALK: &Tag = TAG_MAP.get("Walk - shortsword");
static SHORT_SWORD_JUMP: &Tag = TAG_MAP.get("jump - shortsword");
static SHORT_SWORD_ATTACK: &Tag = TAG_MAP.get("attack - shortsword");
static SHORT_SWORD_JUMP_ATTACK: &Tag = TAG_MAP.get("jump attack - shortsword");

static KNIFE_IDLE: &Tag = TAG_MAP.get("idle - knife");
static KNIFE_WALK: &Tag = TAG_MAP.get("walk - knife");
static KNIFE_JUMP: &Tag = TAG_MAP.get("jump - knife");
static KNIFE_ATTACK: &Tag = TAG_MAP.get("attack - knife");
static KNIFE_JUMP_ATTACK: &Tag = TAG_MAP.get("jump attack - knife");

static SWORDLESS_IDLE: &Tag = TAG_MAP.get("idle swordless");
static SWORDLESS_WALK: &Tag = TAG_MAP.get("walk swordless");
static SWORDLESS_JUMP: &Tag = TAG_MAP.get("jump swordless");
static SWORDLESS_ATTACK: &Tag = KNIFE_ATTACK;
static SWORDLESS_JUMP_ATTACK: &Tag = KNIFE_JUMP_ATTACK;

agb::include_background_gfx!(background, "53269a", background => deduplicate "gfx/background.aseprite");

type Number = FixedNum<8>;

struct Level<'a> {
    background: InfiniteScrolledMap<'a>,
    foreground: InfiniteScrolledMap<'a>,
    clouds: InfiniteScrolledMap<'a>,

    slime_spawns: Vec<(u16, u16)>,
    bat_spawns: Vec<(u16, u16)>,
    emu_spawns: Vec<(u16, u16)>,
}

impl<'a> Level<'a> {
    fn load_level(
        mut backdrop: InfiniteScrolledMap<'a>,
        mut foreground: InfiniteScrolledMap<'a>,
        mut clouds: InfiniteScrolledMap<'a>,
        start_pos: Vector2D<i32>,
        vram: &mut VRamManager,
        sfx: &mut Sfx,
    ) -> Self {
        let vblank = VBlank::get();

        let mut between_updates = || {
            sfx.frame();
            vblank.wait_for_vblank();
        };

        backdrop.init(vram, start_pos, &mut between_updates);
        foreground.init(vram, start_pos, &mut between_updates);
        clouds.init(vram, start_pos / 4, &mut between_updates);

        backdrop.commit(vram);
        foreground.commit(vram);
        clouds.commit(vram);

        backdrop.set_visible(true);
        foreground.set_visible(true);
        clouds.set_visible(true);

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

        if !(0..=tilemap::WIDTH).contains(&x) || !(0..=tilemap::HEIGHT).contains(&y) {
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

    fn clear(&mut self, vram: &mut VRamManager) {
        self.background.clear(vram);
        self.foreground.clear(vram);
        self.clouds.clear(vram);
    }
}

struct Entity<'a> {
    sprite: Object<'a>,
    position: Vector2D<Number>,
    velocity: Vector2D<Number>,
    collision_mask: Rect<Number>,
    visible: bool,
}

impl<'a> Entity<'a> {
    fn new(object_controller: &'a OamManaged, collision_mask: Rect<Number>) -> Self {
        let mut sprite = object_controller.object_sprite(LONG_SWORD_IDLE.sprite(0));
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
        let mut number_collision = self.collision_mask;
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

    fn commit_with_fudge(&mut self, offset: Vector2D<Number>, fudge: Vector2D<Number>) {
        if !self.visible {
            self.sprite.hide();
        } else {
            let position =
                (self.position - offset + fudge + Vector2D::new(num!(0.5), num!(0.5))).floor();
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
    fn idle_animation(self, counter: u16) -> &'static Sprite {
        let counter = counter as usize;
        match self {
            SwordState::LongSword => LONG_SWORD_IDLE.animation_sprite(counter / 8),
            SwordState::ShortSword => SHORT_SWORD_IDLE.animation_sprite(counter / 8),
            SwordState::Dagger => KNIFE_IDLE.animation_sprite(counter / 8),
            SwordState::Swordless => SWORDLESS_IDLE.animation_sprite(counter / 8),
        }
    }
    fn jump_tag(self) -> &'static Tag {
        match self {
            SwordState::LongSword => LONG_SWORD_JUMP,
            SwordState::ShortSword => SHORT_SWORD_JUMP,
            SwordState::Dagger => KNIFE_JUMP,
            SwordState::Swordless => SWORDLESS_JUMP,
        }
    }
    fn walk_animation(self, counter: u16) -> &'static Sprite {
        let counter = counter as usize;
        match self {
            SwordState::LongSword => LONG_SWORD_WALK.animation_sprite(counter / 4),
            SwordState::ShortSword => SHORT_SWORD_WALK.animation_sprite(counter / 4),
            SwordState::Dagger => KNIFE_WALK.animation_sprite(counter / 4),
            SwordState::Swordless => SWORDLESS_WALK.animation_sprite(counter / 4),
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
            SwordState::LongSword => (self.attack_duration().saturating_sub(timer)) / 8,
            SwordState::ShortSword => (self.attack_duration().saturating_sub(timer)) / 8,
            SwordState::Dagger => (self.attack_duration().saturating_sub(timer)) / 8,
            SwordState::Swordless => (self.attack_duration().saturating_sub(timer)) / 8,
        }
    }
    fn jump_attack_tag(self) -> &'static Tag {
        match self {
            SwordState::LongSword => LONG_SWORD_JUMP_ATTACK,
            SwordState::ShortSword => SHORT_SWORD_JUMP_ATTACK,
            SwordState::Dagger => KNIFE_JUMP_ATTACK,
            SwordState::Swordless => SWORDLESS_JUMP_ATTACK,
        }
    }
    fn jump_attack_frame(self, timer: u16) -> u16 {
        (self.jump_attack_duration().saturating_sub(timer)) / 8
    }
    fn hold_frame(self) -> u16 {
        7
    }

    fn cooldown_time(self) -> u16 {
        match self {
            SwordState::LongSword => 20,
            SwordState::ShortSword => 10,
            SwordState::Dagger => 1,
            SwordState::Swordless => 0,
        }
    }
    fn attack_tag(self) -> &'static Tag {
        match self {
            SwordState::LongSword => LONG_SWORD_ATTACK,
            SwordState::ShortSword => SHORT_SWORD_ATTACK,
            SwordState::Dagger => KNIFE_ATTACK,
            SwordState::Swordless => SWORDLESS_ATTACK,
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
    fudge_factor: Vector2D<Number>,
    hurtbox: Option<Rect<Number>>,
    controllable: bool,
}

impl<'a> Player<'a> {
    fn new(object_controller: &'a OamManaged<'_>) -> Player<'a> {
        let mut entity = Entity::new(object_controller, Rect::new((0, 1).into(), (5, 10).into()));
        let s = object_controller.sprite(LONG_SWORD_IDLE.sprite(0));
        entity.sprite.set_sprite(s);
        entity.position = (144, 0).into();

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
        controller: &'a OamManaged,
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
                self.entity.velocity.x = self.entity.velocity.x * 40 / 64;

                match &mut self.attack_timer {
                    AttackTimer::Idle => {
                        if x != Tri::Zero {
                            self.facing = x;
                        }
                        self.entity.sprite.set_hflip(self.facing == Tri::Negative);
                        self.entity.velocity.x += self.sword.ground_walk_force() * x as i32;
                        if self.entity.velocity.x.abs() > Number::new(1) / 10 {
                            let sprite =
                                controller.sprite(self.sword.walk_animation(self.sprite_offset));
                            self.entity.sprite.set_sprite(sprite);
                        } else {
                            let sprite =
                                controller.sprite(self.sword.idle_animation(self.sprite_offset));
                            self.entity.sprite.set_sprite(sprite);
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
                        self.fudge_factor.x = (self.sword.fudge(frame) * self.facing as i32).into();
                        let tag = self.sword.attack_tag();
                        let sprite = controller.sprite(tag.animation_sprite(frame as usize));
                        self.entity.sprite.set_sprite(sprite);

                        hurtbox = self.sword.ground_attack_hurtbox(frame);

                        if *a == 0 {
                            self.attack_timer = AttackTimer::Cooldown(self.sword.cooldown_time());
                        }
                    }
                    AttackTimer::Cooldown(a) => {
                        *a -= 1;
                        let frame = self.sword.hold_frame();
                        self.fudge_factor.x = (self.sword.fudge(frame) * self.facing as i32).into();
                        let tag = self.sword.attack_tag();
                        let sprite = controller.sprite(tag.animation_sprite(frame as usize));
                        self.entity.sprite.set_sprite(sprite);
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
                        let frame = if self.sprite_offset < 3 * 4 {
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
                        let tag = self.sword.jump_tag();
                        let sprite = controller.sprite(tag.animation_sprite(frame as usize));
                        self.entity.sprite.set_sprite(sprite);

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
                        let tag = self.sword.jump_attack_tag();
                        let sprite = controller.sprite(tag.animation_sprite(frame as usize));
                        self.entity.sprite.set_sprite(sprite);

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

        self.fudge_factor.x -= num!(1.5) * (self.facing as i32);

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

    // returns true if the player is alive and false otherwise
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

    fn update<'a>(
        &mut self,
        controller: &'a OamManaged,
        entity: &mut Entity<'a>,
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

        static BAT_IDLE: &Tag = TAG_MAP.get("bat");

        match &mut self.bat_state {
            BatState::Idle => {
                self.sprite_offset += 1;
                if self.sprite_offset >= 9 * 8 {
                    self.sprite_offset = 0;
                }

                if self.sprite_offset == 8 * 5 {
                    sfx.bat_flap();
                }

                let sprite = BAT_IDLE.sprite(self.sprite_offset as usize / 8);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

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

                let sprite = BAT_IDLE.sprite(self.sprite_offset as usize / 2);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

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
                static BAT_DEAD: &Tag = TAG_MAP.get("bat dead");
                let sprite = BAT_DEAD.sprite(0);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

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

    fn update<'a>(
        &mut self,
        controller: &'a OamManaged,
        entity: &mut Entity<'a>,
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

                static IDLE: &Tag = TAG_MAP.get("slime idle");

                let sprite = IDLE.sprite(self.sprite_offset as usize / 16);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

                if (player.entity.position - entity.position).manhattan_distance() < 40.into() {
                    let direction = match player.entity.position.x.cmp(&entity.position.x) {
                        Ordering::Equal => Tri::Zero,
                        Ordering::Greater => Tri::Positive,
                        Ordering::Less => Tri::Negative,
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

                    static CHASE: &Tag = TAG_MAP.get("Slime jump");

                    let sprite = CHASE.sprite(frame as usize);
                    let sprite = controller.sprite(sprite);

                    entity.sprite.set_sprite(sprite);

                    entity.velocity.x = match frame {
                        2..=4 => (Number::new(1) / 5) * Number::new(*direction as i32),
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
                    static DEATH: &Tag = TAG_MAP.get("Slime death");
                    let sprite = DEATH.sprite(*count as usize / 4);
                    let sprite = controller.sprite(sprite);

                    entity.sprite.set_sprite(sprite);
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

    fn update<'a>(
        &mut self,
        controller: &'a OamManaged,
        entity: &mut Entity<'a>,
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

        static ANGRY: &Tag = TAG_MAP.get("angry boss");

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
                    let sprite = ANGRY.animation_sprite(self.sprite_offset as usize / 8);
                    let sprite = controller.sprite(sprite);
                    entity.sprite.set_sprite(sprite);

                    entity.velocity = (0.into(), Number::new(-1) / Number::new(4)).into();
                }

                if should_die {
                    self.sprite_offset = 0;
                    self.state = MiniFlameState::Dead;

                    if rng::gen() % 4 == 0 {
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

                    if rng::gen() % 4 == 0 {
                        instruction = UpdateInstruction::CreateParticle(
                            ParticleData::new_health(),
                            entity.position,
                        );
                    }
                } else if should_damage {
                    instruction = UpdateInstruction::DamagePlayer;
                }

                if entity.velocity.manhattan_distance() < Number::new(1) / Number::new(4) {
                    self.state = MiniFlameState::Idle(90);
                }

                let sprite = ANGRY.animation_sprite(self.sprite_offset as usize / 2);
                let sprite = controller.sprite(sprite);
                entity.sprite.set_sprite(sprite);
            }
            MiniFlameState::Dead => {
                entity.velocity = (0, 0).into();
                if self.sprite_offset >= 6 * 12 {
                    instruction = UpdateInstruction::Remove;
                }

                static DEATH: &Tag = TAG_MAP.get("angry boss dead");

                let sprite = DEATH.animation_sprite(self.sprite_offset as usize / 12);
                let sprite = controller.sprite(sprite);
                entity.sprite.set_sprite(sprite);

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

    fn update<'a>(
        &mut self,
        controller: &'a OamManaged,
        entity: &mut Entity<'a>,
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

                static IDLE: &Tag = TAG_MAP.get("emu - idle");

                let sprite = IDLE.sprite(self.sprite_offset as usize / 16);
                let sprite = controller.sprite(sprite);
                entity.sprite.set_sprite(sprite);

                if (entity.position.y - player.entity.position.y).abs() < 10.into() {
                    let velocity = Number::new(1)
                        * (player.entity.position.x - entity.position.x)
                            .to_raw()
                            .signum();
                    entity.velocity.x = velocity;

                    match velocity.cmp(&0.into()) {
                        Ordering::Greater => {
                            entity.sprite.set_hflip(true);
                            self.state = EmuState::Charging(Tri::Positive);
                        }
                        Ordering::Less => {
                            self.state = EmuState::Charging(Tri::Negative);
                            entity.sprite.set_hflip(false);
                        }
                        Ordering::Equal => {
                            self.state = EmuState::Idle;
                        }
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

                static WALK: &Tag = TAG_MAP.get("emu-walk");

                let sprite = WALK.sprite(self.sprite_offset as usize / 2);
                let sprite = controller.sprite(sprite);
                entity.sprite.set_sprite(sprite);

                let gravity: Number = 1.into();
                let gravity = gravity / 16;
                entity.velocity.y += gravity;

                let distance_traveled = entity.update_position(level);

                if distance_traveled.x == 0.into() {
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

                static DEATH: &Tag = TAG_MAP.get("emu - die");

                let sprite = DEATH.animation_sprite(self.sprite_offset as usize / 4);
                let sprite = controller.sprite(sprite);
                entity.sprite.set_sprite(sprite);

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
    fn collision_mask(&self) -> Rect<Number> {
        match self {
            EnemyData::Slime(_) => Rect::new((0.into(), num!(1.5)).into(), (4, 11).into()),
            EnemyData::Bat(_) => Rect::new((0, 0).into(), (12, 4).into()),
            EnemyData::MiniFlame(_) => Rect::new((0, 0).into(), (12, 12).into()),
            EnemyData::Emu(_) => Rect::new((0, 0).into(), (7, 11).into()),
        }
    }

    fn sprite(&self) -> &'static Sprite {
        static SLIME: &Tag = TAG_MAP.get("slime idle");
        static BAT: &Tag = TAG_MAP.get("bat");
        static MINI_FLAME: &Tag = TAG_MAP.get("angry boss");
        static EMU: &Tag = TAG_MAP.get("emu - idle");
        match self {
            EnemyData::Slime(_) => SLIME.sprite(0),
            EnemyData::Bat(_) => BAT.sprite(0),
            EnemyData::MiniFlame(_) => MINI_FLAME.sprite(0),
            EnemyData::Emu(_) => EMU.sprite(0),
        }
    }

    fn update<'a>(
        &mut self,
        controller: &'a OamManaged,
        entity: &mut Entity<'a>,
        player: &Player,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        match self {
            EnemyData::Slime(data) => data.update(controller, entity, player, level, sfx),
            EnemyData::Bat(data) => data.update(controller, entity, player, level, sfx),
            EnemyData::MiniFlame(data) => data.update(controller, entity, player, level, sfx),
            EnemyData::Emu(data) => data.update(controller, entity, player, level, sfx),
        }
    }
}

struct Enemy<'a> {
    entity: Entity<'a>,
    enemy_data: EnemyData,
}

impl<'a> Enemy<'a> {
    fn new(object_controller: &'a OamManaged, enemy_data: EnemyData) -> Self {
        let mut entity = Entity::new(object_controller, enemy_data.collision_mask());

        let sprite = enemy_data.sprite();
        let sprite = object_controller.sprite(sprite);

        entity.sprite.set_sprite(sprite);
        entity.sprite.show();

        Self { entity, enemy_data }
    }

    fn update(
        &mut self,
        controller: &'a OamManaged,
        player: &Player,
        level: &Level,
        sfx: &mut sfx::Sfx,
    ) -> UpdateInstruction {
        self.enemy_data
            .update(controller, &mut self.entity, player, level, sfx)
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

    fn update<'a>(
        &mut self,
        controller: &'a OamManaged,
        entity: &mut Entity<'a>,
        player: &Player,
        _level: &Level,
    ) -> UpdateInstruction {
        match self {
            ParticleData::Dust(frame) => {
                if *frame == 8 * 3 {
                    return UpdateInstruction::Remove;
                }

                static DUST: &Tag = TAG_MAP.get("dust");
                let sprite = DUST.sprite(*frame as usize / 3);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

                *frame += 1;
                UpdateInstruction::None
            }
            ParticleData::Health(frame) => {
                if *frame > 8 * 3 * 6 {
                    return UpdateInstruction::Remove; // have played the animation 6 times
                }

                static HEALTH: &Tag = TAG_MAP.get("Heath");
                let sprite = HEALTH.animation_sprite(*frame as usize / 3);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

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
                static HEALTH: &Tag = TAG_MAP.get("Heath");
                let sprite = HEALTH.animation_sprite(*frame as usize / 3);
                let sprite = controller.sprite(sprite);

                entity.sprite.set_sprite(sprite);

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
        object_controller: &'a OamManaged,
        particle_data: ParticleData,
        position: Vector2D<Number>,
    ) -> Self {
        let mut entity = Entity::new(object_controller, Rect::new((0, 0).into(), (0, 0).into()));

        entity.position = position;

        Self {
            entity,
            particle_data,
        }
    }

    fn update(
        &mut self,
        controller: &'a OamManaged,
        player: &Player,
        level: &Level,
    ) -> UpdateInstruction {
        self.entity.sprite.show();
        self.particle_data
            .update(controller, &mut self.entity, player, level)
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
        object_controller: &'a OamManaged,
        player: &Player,
        sfx: &mut sfx::Sfx,
    ) -> BossInstruction {
        match self {
            BossState::Active(boss) => boss.update(enemies, object_controller, player, sfx),
            BossState::Following(boss) => {
                boss.update(object_controller, player);
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
    fn new(object_controller: &'a OamManaged, position: Vector2D<Number>) -> Self {
        let mut entity = Entity::new(object_controller, Rect::new((0, 0).into(), (0, 0).into()));
        entity.position = position;

        Self {
            entity,
            following: true,
            timer: 0,
            to_hole: false,
            gone: false,
        }
    }
    fn update(&mut self, controller: &'a OamManaged, player: &Player) {
        let difference = player.entity.position - self.entity.position;
        self.timer += 1;

        let frame = if self.to_hole {
            let target: Vector2D<Number> = (17 * 8, -3 * 8).into();
            let difference = target - self.entity.position;
            if difference.manhattan_distance() < 1.into() {
                self.gone = true;
            } else {
                self.entity.velocity = difference.normalise() * 2;
            }

            self.timer / 8
        } else if self.timer < 120 {
            self.timer / 20
        } else if self.following {
            self.entity.velocity = difference / 16;
            if difference.manhattan_distance() < 20.into() {
                self.following = false;
            }
            self.timer / 8
        } else {
            self.entity.velocity = (0, 0).into();
            if difference.manhattan_distance() > 60.into() {
                self.following = true;
            }
            self.timer / 16
        };

        static BOSS: &Tag = TAG_MAP.get("happy boss");

        let sprite = BOSS.animation_sprite(frame as usize);
        let sprite = controller.sprite(sprite);

        self.entity.sprite.set_sprite(sprite);

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
    fn new(object_controller: &'a OamManaged, screen_coords: Vector2D<Number>) -> Self {
        let mut entity = Entity::new(object_controller, Rect::new((0, 0).into(), (28, 28).into()));
        entity.position = screen_coords + (144, 136).into();
        Self {
            entity,
            health: 5,
            target_location: rng::gen().rem_euclid(5) as u8,
            state: BossActiveState::Damaged(60),
            timer: 0,
            screen_coords,
            shake_magnitude: 0.into(),
        }
    }
    fn update(
        &mut self,
        enemies: &mut Arena<Enemy<'a>>,
        object_controller: &'a OamManaged,
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
        let frame = self.timer / animation_rate;

        static BOSS: &Tag = TAG_MAP.get("Boss");

        let sprite = BOSS.animation_sprite(frame as usize);
        let sprite = object_controller.sprite(sprite);

        self.entity.sprite.set_sprite(sprite);

        self.entity.update_position_without_collision();
        instruction
    }
    fn commit(&mut self, offset: Vector2D<Number>) {
        let shake = if self.shake_magnitude != 0.into() {
            (
                Number::from_raw(rng::gen()).rem_euclid(self.shake_magnitude)
                    - self.shake_magnitude / 2,
                Number::from_raw(rng::gen()).rem_euclid(self.shake_magnitude)
                    - self.shake_magnitude / 2,
            )
                .into()
        } else {
            (0, 0).into()
        };

        self.entity
            .commit_with_size(offset + shake, (32, 32).into());
    }
    fn explode(&self, enemies: &mut Arena<Enemy<'a>>, object_controller: &'a OamManaged) {
        for _ in 0..(6 - self.health) {
            let x_offset: Number = Number::from_raw(rng::gen()).rem_euclid(2.into()) - 1;
            let y_offset: Number = Number::from_raw(rng::gen()).rem_euclid(2.into()) - 1;
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
            let a = rng::gen().rem_euclid(5) as u8;
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
    level: Level<'a>,
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
            BossState::NotSpawned => self.offset.x.floor() + 248 >= tilemap::WIDTH * 8,
            _ => false,
        }
    }

    fn clear(&mut self, vram: &mut VRamManager) {
        self.level.clear(vram);
    }

    fn advance_frame(
        &mut self,
        object_controller: &'a OamManaged,
        vram: &mut VRamManager,
        sfx: &mut sfx::Sfx,
    ) -> GameStatus {
        let mut state = GameStatus::Continue;

        match self.move_state {
            MoveState::Advancing => {
                let difference = self.player.entity.position.x - (self.offset.x + WIDTH / 2);

                self.offset.x = self.offset.x.max(self.offset.x + difference / 16);

                if self.has_just_reached_end() {
                    sfx.boss();
                    self.offset.x = (tilemap::WIDTH * 8 - 248).into();
                    self.move_state = MoveState::PinnedAtEnd;
                    self.boss = BossState::Active(Boss::new(object_controller, self.offset))
                }
            }
            MoveState::PinnedAtEnd => {
                self.offset.x = (tilemap::WIDTH * 8 - 248).into();
            }
            MoveState::FollowingPlayer => {
                Game::update_sunrise(vram, self.sunrise_timer);
                if self.sunrise_timer < 120 {
                    self.sunrise_timer += 1;
                } else {
                    let difference = self.player.entity.position.x - (self.offset.x + WIDTH / 2);

                    self.offset.x += difference / 8;
                    if self.offset.x > (tilemap::WIDTH * 8 - 248).into() {
                        self.offset.x = (tilemap::WIDTH * 8 - 248).into();
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
                        Game::update_fade_out(vram, self.fade_count);
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
                Number::from_raw(rng::gen()) % size - Number::new(size) / 2,
                Number::from_raw(rng::gen()) % size - Number::new(size) / 2,
            )
                .into();
            this_frame_offset += offset;
            self.shake_time -= 1;
        }

        let this_frame_offset = this_frame_offset.floor().into();

        self.input.update();
        if let UpdateInstruction::CreateParticle(data, position) =
            self.player
                .update(object_controller, &self.input, &self.level, sfx)
        {
            let new_particle = Particle::new(object_controller, data, position);

            self.particles.insert(new_particle);
        }

        let mut remove = Vec::new();
        for (idx, enemy) in self.enemies.iter_mut() {
            if enemy.entity.position.x < self.offset.x - 8 {
                remove.push(idx);
                continue;
            }

            match enemy.update(object_controller, &self.player, &self.level, sfx) {
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

        let background_offset = (this_frame_offset.floor().x, 8).into();

        self.level.background.set_pos(vram, background_offset);
        self.level.foreground.set_pos(vram, background_offset);
        self.level.clouds.set_pos(vram, background_offset / 4);

        for i in remove {
            self.enemies.remove(i);
        }

        let mut remove = Vec::with_capacity(10);

        for (idx, particle) in self.particles.iter_mut() {
            match particle.update(object_controller, &self.player, &self.level) {
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

    fn load_enemies(&mut self, object_controller: &'a OamManaged) {
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

    fn update_sunrise(vram: &mut VRamManager, time: u16) {
        let mut modified_palette = background::PALETTES[0].clone();

        let a = modified_palette.colour(0);
        let b = modified_palette.colour(1);

        modified_palette.update_colour(0, interpolate_colour(a, 17982, time, 120));
        modified_palette.update_colour(1, interpolate_colour(b, 22427, time, 120));

        let modified_palettes = [modified_palette];

        vram.set_background_palettes(&modified_palettes);
    }

    fn update_fade_out(vram: &mut VRamManager, time: u16) {
        let mut modified_palette = background::PALETTES[0].clone();

        let c = modified_palette.colour(2);

        modified_palette.update_colour(0, interpolate_colour(17982, 0x7FFF, time, 600));
        modified_palette.update_colour(1, interpolate_colour(22427, 0x7FFF, time, 600));
        modified_palette.update_colour(2, interpolate_colour(c, 0x7FFF, time, 600));

        let modified_palettes = [modified_palette];

        vram.set_background_palettes(&modified_palettes);
    }

    fn new(object: &'a OamManaged<'a>, level: Level<'a>, start_at_boss: bool) -> Self {
        let mut player = Player::new(object);
        let mut offset = (144 - WIDTH / 2, 8).into();
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
        }
    }
}

fn game_with_level(gba: &mut agb::Gba) {
    let vblank = agb::interrupt::VBlank::get();
    vblank.wait_for_vblank();

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    mixer.enable();

    let mut sfx = sfx::Sfx::new(&mut mixer);
    sfx.purple_night();

    let mut start_at_boss = false;

    let (background, mut vram) = gba.display.video.tiled0();
    vram.set_background_palettes(background::PALETTES);
    let tileset = &background::background.tiles;
    let object = gba.display.object.get_managed();

    loop {
        let backdrop = InfiniteScrolledMap::new(
            background.background(
                Priority::P2,
                RegularBackgroundSize::Background32x32,
                TileFormat::FourBpp,
            ),
            Box::new(|pos| {
                (
                    tileset,
                    background::background.tile_settings[*tilemap::BACKGROUND_MAP
                        .get((pos.x + tilemap::WIDTH * pos.y) as usize)
                        .unwrap_or(&0)
                        as usize],
                )
            }),
        );

        let foreground = InfiniteScrolledMap::new(
            background.background(
                Priority::P0,
                RegularBackgroundSize::Background32x32,
                TileFormat::FourBpp,
            ),
            Box::new(|pos| {
                (
                    tileset,
                    background::background.tile_settings[*tilemap::FOREGROUND_MAP
                        .get((pos.x + tilemap::WIDTH * pos.y) as usize)
                        .unwrap_or(&0)
                        as usize],
                )
            }),
        );

        let clouds = InfiniteScrolledMap::new(
            background.background(
                Priority::P3,
                RegularBackgroundSize::Background32x32,
                TileFormat::FourBpp,
            ),
            Box::new(|pos| {
                (
                    tileset,
                    background::background.tile_settings[*tilemap::CLOUD_MAP
                        .get((pos.x + tilemap::WIDTH * pos.y) as usize)
                        .unwrap_or(&0)
                        as usize],
                )
            }),
        );

        let start_pos = if start_at_boss {
            (130 * 8, 8).into()
        } else {
            (144 - WIDTH / 2, 8).into()
        };

        let mut game = Game::new(
            &object,
            Level::load_level(backdrop, foreground, clouds, start_pos, &mut vram, &mut sfx),
            start_at_boss,
        );

        start_at_boss = loop {
            sfx.frame();
            vblank.wait_for_vblank();
            game.level.background.commit(&mut vram);
            game.level.foreground.commit(&mut vram);
            game.level.clouds.commit(&mut vram);
            object.commit();
            match game.advance_frame(&object, &mut vram, &mut sfx) {
                GameStatus::Continue => {}
                GameStatus::Lost => {
                    break false;
                }
                GameStatus::RespawnAtBoss => {
                    break true;
                }
            }

            let _ = rng::gen(); // advance RNG to make it less predictable between runs
        };

        game.clear(&mut vram);
    }
}

mod tilemap {
    include!(concat!(env!("OUT_DIR"), "/tilemap.rs"));
}

pub fn main(mut gba: agb::Gba) -> ! {
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
