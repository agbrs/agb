#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    display::{
        background::{
            InfiniteScrolledMap, PartialUpdateStatus, TileFormat, TileSet, TileSetting, VRamManager,
        },
        object::{ObjectControl, ObjectStandard, Size},
        Priority, HEIGHT, WIDTH,
    },
    fixnum::{FixedNum, Vector2D},
    input::{self, Button, ButtonController},
};
use alloc::boxed::Box;

mod enemies;
mod level_display;
mod sfx;
mod splash_screen;

pub struct Level {
    background: &'static [u16],
    foreground: &'static [u16],
    dimensions: Vector2D<u32>,
    collision: &'static [u32],

    slimes: &'static [(i32, i32)],
    snails: &'static [(i32, i32)],
    enemy_stops: &'static [(i32, i32)],
    start_pos: (i32, i32),
}

mod object_tiles {
    pub const WIZARD_TILE_START: u16 = 0;
    pub const WIZARD_JUMP: u16 = 4 * 4;
    pub const WIZARD_FALL_START: u16 = 5 * 4;

    pub const HAT_TILE_START: u16 = 9 * 4;
    pub const HAT_TILE_START_SECOND: u16 = 28 * 4;
    pub const HAT_TILE_START_THIRD: u16 = 38 * 4;

    pub const SLIME_IDLE_START: u16 = 19 * 4;
    pub const SLIME_JUMP_START: u16 = 20 * 4;
    pub const SLIME_SPLAT_START: u16 = 24 * 4;

    pub const SNAIL_IDLE_START: u16 = 48 * 4;
    pub const SNAIL_EMERGE_START: u16 = 49 * 4;
    pub const SNAIL_MOVE: u16 = 54 * 4;
    pub const SNAIL_DEATH_START: u16 = 56 * 4;
}

agb::include_gfx!("gfx/object_sheet.toml");

mod map_tiles {

    use super::Level;
    pub const LEVELS: &[Level] = &[
        l1_1::get_level(),
        l1_2::get_level(),
        l1_3::get_level(),
        l1_4::get_level(),
        l1_5::get_level(),
        l1_7::get_level(), // these are intentionally this way round
        l1_6::get_level(),
        l1_8::get_level(),
        l2_3::get_level(), // goes 2-3, 2-1 then 2-2
        l2_1::get_level(),
        l2_2::get_level(),
        l2_4::get_level(),
    ];

    pub mod l1_1 {
        include!(concat!(env!("OUT_DIR"), "/1-1.json.rs"));
    }
    pub mod l1_2 {
        include!(concat!(env!("OUT_DIR"), "/1-2.json.rs"));
    }
    pub mod l1_3 {
        include!(concat!(env!("OUT_DIR"), "/1-3.json.rs"));
    }
    pub mod l1_4 {
        include!(concat!(env!("OUT_DIR"), "/1-4.json.rs"));
    }
    pub mod l1_5 {
        include!(concat!(env!("OUT_DIR"), "/1-5.json.rs"));
    }
    pub mod l1_6 {
        include!(concat!(env!("OUT_DIR"), "/1-6.json.rs"));
    }
    pub mod l1_7 {
        include!(concat!(env!("OUT_DIR"), "/1-7.json.rs"));
    }
    pub mod l2_1 {
        include!(concat!(env!("OUT_DIR"), "/2-1.json.rs"));
    }

    pub mod l1_8 {
        include!(concat!(env!("OUT_DIR"), "/1-8.json.rs"));
    }
    pub mod l2_2 {
        include!(concat!(env!("OUT_DIR"), "/2-2.json.rs"));
    }
    pub mod l2_3 {
        include!(concat!(env!("OUT_DIR"), "/2-3.json.rs"));
    }

    pub mod l2_4 {
        include!(concat!(env!("OUT_DIR"), "/2-4.json.rs"));
    }

    pub mod tilemap {
        include!(concat!(env!("OUT_DIR"), "/tilemap.rs"));
    }
}

agb::include_gfx!("gfx/tile_sheet.toml");

type FixedNumberType = FixedNum<10>;

pub struct Entity<'a> {
    sprite: ObjectStandard<'a>,
    position: Vector2D<FixedNumberType>,
    velocity: Vector2D<FixedNumberType>,
    collision_mask: Vector2D<u16>,
}

impl<'a> Entity<'a> {
    pub fn new(object: &'a ObjectControl, collision_mask: Vector2D<u16>) -> Self {
        let mut sprite = object.get_object_standard();
        sprite.set_priority(Priority::P1);
        Entity {
            sprite,
            collision_mask,
            position: (0, 0).into(),
            velocity: (0, 0).into(),
        }
    }

    fn something_at_point<T: Fn(i32, i32) -> bool>(
        &self,
        position: Vector2D<FixedNumberType>,
        something_fn: T,
    ) -> bool {
        let left = (position.x - self.collision_mask.x as i32 / 2).floor() / 8;
        let right = (position.x + self.collision_mask.x as i32 / 2 - 1).floor() / 8;
        let top = (position.y - self.collision_mask.y as i32 / 2).floor() / 8;
        let bottom = (position.y + self.collision_mask.y as i32 / 2 - 1).floor() / 8;

        for x in left..=right {
            for y in top..=bottom {
                if something_fn(x, y) {
                    return true;
                }
            }
        }
        false
    }

    fn collision_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        self.something_at_point(position, |x, y| level.collides(x, y))
    }

    fn killision_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        self.something_at_point(position, |x, y| level.kills(x, y))
    }

    fn completion_at_point(&self, level: &Level, position: Vector2D<FixedNumberType>) -> bool {
        self.something_at_point(position, |x, y| level.wins(x, y))
    }

    fn enemy_collision_at_point(
        &self,
        enemies: &[enemies::Enemy],
        position: Vector2D<FixedNumberType>,
    ) -> bool {
        for enemy in enemies {
            if enemy.collides_with_hat(position) {
                return true;
            }
        }
        false
    }

    // returns the distance actually moved
    fn update_position(&mut self, level: &Level) -> Vector2D<FixedNumberType> {
        let old_position = self.position;
        let x_velocity = (self.velocity.x, 0.into()).into();
        if !self.collision_at_point(level, self.position + x_velocity) {
            self.position += x_velocity;
        } else {
            self.position += self.binary_search_collision(level, (1, 0).into(), self.velocity.x);
        }

        let y_velocity = (0.into(), self.velocity.y).into();
        if !self.collision_at_point(level, self.position + y_velocity) {
            self.position += y_velocity;
        } else {
            self.position += self.binary_search_collision(level, (0, 1).into(), self.velocity.y);
        }

        self.position - old_position
    }

    fn update_position_with_enemy(
        &mut self,
        level: &Level,
        enemies: &[enemies::Enemy],
    ) -> (Vector2D<FixedNumberType>, bool) {
        let mut was_enemy_collision = false;
        let old_position = self.position;
        let x_velocity = (self.velocity.x, 0.into()).into();

        if !(self.collision_at_point(level, self.position + x_velocity)
            || self.enemy_collision_at_point(enemies, self.position + x_velocity))
        {
            self.position += x_velocity;
        } else if self.enemy_collision_at_point(enemies, self.position + x_velocity) {
            self.position -= x_velocity;
            was_enemy_collision = true;
        }

        let y_velocity = (0.into(), self.velocity.y).into();
        if !(self.collision_at_point(level, self.position + y_velocity)
            || self.enemy_collision_at_point(enemies, self.position + y_velocity))
        {
            self.position += y_velocity;
        } else if self.enemy_collision_at_point(enemies, self.position + y_velocity) {
            self.position -= y_velocity;
            was_enemy_collision = true;
        }

        (self.position - old_position, was_enemy_collision)
    }

    fn binary_search_collision(
        &self,
        level: &Level,
        unit_vector: Vector2D<FixedNumberType>,
        initial: FixedNumberType,
    ) -> Vector2D<FixedNumberType> {
        let mut low: FixedNumberType = 0.into();
        let mut high = initial;

        let one: FixedNumberType = 1.into();
        while (high - low).abs() > one / 8 {
            let mid = (low + high) / 2;
            let new_vel: Vector2D<FixedNumberType> = unit_vector * mid;

            if self.collision_at_point(level, self.position + new_vel) {
                high = mid;
            } else {
                low = mid;
            }
        }

        unit_vector * low
    }

    fn commit_position(&mut self, offset: Vector2D<FixedNumberType>) {
        let position = (self.position - offset).floor();
        self.sprite.set_position(position - (8, 8).into());
        if position.x < -8 || position.x > WIDTH + 8 || position.y < -8 || position.y > HEIGHT + 8 {
            self.sprite.hide();
        } else {
            self.sprite.show();
        }
        self.sprite.commit();
    }
}

struct Map<'a, 'b> {
    background: &'a mut InfiniteScrolledMap<'b>,
    foreground: &'a mut InfiniteScrolledMap<'b>,
    position: Vector2D<FixedNumberType>,
    level: &'a Level,
}

impl<'a, 'b> Map<'a, 'b> {
    pub fn commit_position(&mut self, vram: &mut VRamManager) {
        self.background.set_pos(vram, self.position.floor());
        self.foreground.set_pos(vram, self.position.floor());

        self.background.commit();
        self.foreground.commit();
    }

    pub fn init_background(&mut self, vram: &mut VRamManager) -> PartialUpdateStatus {
        self.background.init_partial(vram, self.position.floor())
    }

    pub fn init_foreground(&mut self, vram: &mut VRamManager) -> PartialUpdateStatus {
        self.foreground.init_partial(vram, self.position.floor())
    }
}

impl Level {
    fn collides(&self, x: i32, y: i32) -> bool {
        self.at_point(x, y, map_tiles::tilemap::COLLISION_TILE as u32)
    }

    fn kills(&self, x: i32, y: i32) -> bool {
        self.at_point(x, y, map_tiles::tilemap::KILL_TILE as u32)
    }

    fn at_point(&self, x: i32, y: i32, tile: u32) -> bool {
        if (x < 0 || x >= self.dimensions.x as i32) || (y < 0 || y >= self.dimensions.y as i32) {
            return true;
        }
        let pos = (self.dimensions.x as i32 * y + x) as usize;
        let tile_foreground = self.foreground[pos];
        let tile_background = self.background[pos];
        let foreground_tile_property = self.collision[tile_foreground as usize];
        let background_tile_property = self.collision[tile_background as usize];
        foreground_tile_property == tile || background_tile_property == tile
    }

    fn wins(&self, x: i32, y: i32) -> bool {
        self.at_point(x, y, map_tiles::tilemap::WIN_TILE as u32)
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum HatState {
    OnHead,
    Thrown,
    WizardTowards,
}

struct Player<'a> {
    wizard: Entity<'a>,
    hat: Entity<'a>,
    hat_state: HatState,
    hat_left_range: bool,
    hat_slow_counter: i32,
    wizard_frame: u8,
    num_recalls: i8,
    is_on_ground: bool,
    facing: input::Tri,
}

fn ping_pong(i: i32, n: i32) -> i32 {
    let cycle = 2 * (n - 1);
    let i = i % cycle;
    if i >= n {
        cycle - i
    } else {
        i
    }
}

impl<'a> Player<'a> {
    fn new(controller: &'a ObjectControl, start_position: Vector2D<FixedNumberType>) -> Self {
        let mut hat = Entity::new(controller, (6_u16, 6_u16).into());
        let mut wizard = Entity::new(controller, (6_u16, 14_u16).into());

        wizard.sprite.set_tile_id(object_tiles::WIZARD_TILE_START);
        hat.sprite.set_tile_id(object_tiles::HAT_TILE_START);
        wizard.sprite.set_sprite_size(Size::S16x16);
        hat.sprite.set_sprite_size(Size::S16x16);
        wizard.sprite.show();
        hat.sprite.show();

        wizard.sprite.commit();
        hat.sprite.commit();

        wizard.position = start_position;
        hat.position = start_position - (0, 10).into();

        Player {
            wizard,
            hat,
            hat_slow_counter: 0,
            hat_state: HatState::OnHead,
            hat_left_range: false,
            wizard_frame: 0,
            num_recalls: 0,
            is_on_ground: true,
            facing: input::Tri::Zero,
        }
    }

    fn update_frame(
        &mut self,
        input: &ButtonController,
        timer: i32,
        level: &Level,
        enemies: &[enemies::Enemy],
        sfx_player: &mut sfx::SfxPlayer,
    ) {
        // throw or recall
        if input.is_just_pressed(Button::A) {
            if self.hat_state == HatState::OnHead {
                let direction: Vector2D<FixedNumberType> = {
                    let up_down = input.y_tri() as i32;
                    let left_right = if up_down == 0 {
                        self.facing as i32
                    } else {
                        input.x_tri() as i32
                    };
                    (left_right, up_down).into()
                };

                if direction != (0, 0).into() {
                    let mut velocity = direction.normalise() * 5;
                    if velocity.y > 0.into() {
                        velocity.y *= FixedNumberType::new(4) / 3;
                    }
                    self.hat.velocity = velocity;
                    self.hat_state = HatState::Thrown;

                    sfx_player.throw();
                }
            } else if self.hat_state == HatState::Thrown {
                self.num_recalls += 1;
                if self.num_recalls < 3 {
                    self.hat.velocity = (0, 0).into();
                    self.wizard.velocity = (0, 0).into();
                    self.hat_state = HatState::WizardTowards;
                }
            } else if self.hat_state == HatState::WizardTowards {
                self.hat_state = HatState::Thrown;
                self.wizard.velocity /= 8;
            }
        }

        let was_on_ground = self.is_on_ground;
        let is_on_ground = self
            .wizard
            .collision_at_point(level, self.wizard.position + (0, 1).into());

        if is_on_ground && !was_on_ground && self.wizard.velocity.y > 1.into() {
            sfx_player.land();
        }
        self.is_on_ground = is_on_ground;

        if self.hat_state != HatState::WizardTowards {
            if is_on_ground {
                self.num_recalls = 0;
            }

            if is_on_ground {
                self.wizard.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 16;
                self.wizard.velocity = self.wizard.velocity * 54 / 64;
                if input.is_just_pressed(Button::B) {
                    self.wizard.velocity.y = -FixedNumberType::new(3) / 2;
                    sfx_player.jump();
                }
            } else {
                self.wizard.velocity.x += FixedNumberType::new(input.x_tri() as i32) / 64;
                self.wizard.velocity = self.wizard.velocity * 63 / 64;
                let gravity: Vector2D<FixedNumberType> = (0, 1).into();
                let gravity = gravity / 16;
                self.wizard.velocity += gravity;
            }

            self.wizard.velocity = self.wizard.update_position(level);

            if self.wizard.velocity.x.abs() > 0.into() {
                let offset = (ping_pong(timer / 16, 4)) as u16;
                self.wizard_frame = offset as u8;

                self.wizard
                    .sprite
                    .set_tile_id(object_tiles::WIZARD_TILE_START + offset * 4);
            }

            if self.wizard.velocity.y < -FixedNumberType::new(1) / 16 {
                // going up
                self.wizard_frame = 5;

                self.wizard.sprite.set_tile_id(object_tiles::WIZARD_JUMP);
            } else if self.wizard.velocity.y > FixedNumberType::new(1) / 16 {
                // going down
                let offset = if self.wizard.velocity.y * 2 > 3.into() {
                    ((timer / 4) % 4) as u16
                } else {
                    // Don't flap beard unless going quickly
                    0
                };

                self.wizard_frame = 0;

                self.wizard
                    .sprite
                    .set_tile_id(object_tiles::WIZARD_FALL_START + offset * 4);
            }

            if input.x_tri() != agb::input::Tri::Zero {
                self.facing = input.x_tri();
            }
        }

        let hat_base_tile = match self.num_recalls {
            0 => object_tiles::HAT_TILE_START,
            1 => object_tiles::HAT_TILE_START_SECOND,
            _ => object_tiles::HAT_TILE_START_THIRD,
        };

        match self.facing {
            agb::input::Tri::Negative => {
                self.wizard.sprite.set_hflip(true);
                self.hat.sprite.set_tile_id(hat_base_tile + 4 * 5);
            }
            agb::input::Tri::Positive => {
                self.wizard.sprite.set_hflip(false);
                self.hat.sprite.set_tile_id(hat_base_tile);
            }
            _ => {}
        }

        let hat_resting_position = match self.wizard_frame {
            1 | 2 => (0, 9).into(),
            5 => (0, 10).into(),
            _ => (0, 8).into(),
        };

        match self.hat_state {
            HatState::Thrown => {
                // hat is thrown, make hat move towards wizard
                let distance_vector =
                    self.wizard.position - self.hat.position - hat_resting_position;
                let distance = distance_vector.magnitude();
                let direction = if distance == 0.into() {
                    (0, 0).into()
                } else {
                    distance_vector / distance
                };

                let hat_sprite_divider = match self.num_recalls {
                    0 => 1,
                    1 => 2,
                    _ => 4,
                };

                let hat_sprite_offset = timer / hat_sprite_divider % 10;

                self.hat
                    .sprite
                    .set_tile_id(hat_base_tile + (hat_sprite_offset * 4) as u16);

                if self.hat_slow_counter < 30 && self.hat.velocity.magnitude() < 2.into() {
                    self.hat.velocity = (0, 0).into();
                    self.hat_slow_counter += 1;
                } else {
                    self.hat.velocity += direction / 4;
                }
                let (new_velocity, enemy_collision) =
                    self.hat.update_position_with_enemy(level, enemies);
                self.hat.velocity = new_velocity;

                if enemy_collision {
                    sfx_player.snail_hat_bounce();
                }

                if distance > 16.into() {
                    self.hat_left_range = true;
                }
                if self.hat_left_range && distance < 16.into() {
                    sfx_player.catch();
                    self.hat_state = HatState::OnHead;
                }
            }
            HatState::OnHead => {
                // hat is on head, place hat on head
                self.hat_slow_counter = 0;
                self.hat_left_range = false;
                self.hat.position = self.wizard.position - hat_resting_position;
            }
            HatState::WizardTowards => {
                self.hat
                    .sprite
                    .set_tile_id(hat_base_tile + 4 * (timer / 2 % 10) as u16);
                let distance_vector =
                    self.hat.position - self.wizard.position + hat_resting_position;
                let distance = distance_vector.magnitude();
                if distance != 0.into() {
                    let v = self.wizard.velocity.magnitude() + 1;
                    self.wizard.velocity = distance_vector / distance * v;
                }
                self.wizard.velocity = self.wizard.update_position(level);
                if distance < 16.into() {
                    self.wizard.velocity /= 8;
                    self.hat_state = HatState::OnHead;
                    sfx_player.catch();
                }
            }
        }
    }
}

struct PlayingLevel<'a, 'b> {
    timer: i32,
    background: Map<'a, 'b>,
    input: ButtonController,
    player: Player<'a>,

    enemies: [enemies::Enemy<'a>; 16],
}

enum UpdateState {
    Normal,
    Dead,
    Complete,
}

impl<'a, 'b, 'c> PlayingLevel<'a, 'b> {
    fn open_level(
        level: &'a Level,
        object_control: &'a ObjectControl,
        background: &'a mut InfiniteScrolledMap<'b>,
        foreground: &'a mut InfiniteScrolledMap<'b>,
        input: ButtonController,
    ) -> Self {
        let mut e: [enemies::Enemy<'a>; 16] = Default::default();
        let mut enemy_count = 0;
        for &slime in level.slimes {
            e[enemy_count] = enemies::Enemy::new_slime(object_control, slime.into());
            enemy_count += 1;
        }

        for &snail in level.snails {
            e[enemy_count] = enemies::Enemy::new_snail(object_control, snail.into());
            enemy_count += 1;
        }

        let start_pos: Vector2D<FixedNumberType> = level.start_pos.into();

        let background_position = (
            (start_pos.x - WIDTH / 2)
                .clamp(0.into(), ((level.dimensions.x * 8) as i32 - WIDTH).into()),
            (start_pos.y - HEIGHT / 2)
                .clamp(0.into(), ((level.dimensions.y * 8) as i32 - HEIGHT).into()),
        )
            .into();

        PlayingLevel {
            timer: 0,
            background: Map {
                background,
                foreground,
                level,
                position: background_position,
            },
            player: Player::new(object_control, start_pos),
            input,
            enemies: e,
        }
    }

    fn show_backgrounds(&mut self) {
        self.background.background.show();
        self.background.foreground.show();
    }

    fn hide_backgrounds(&mut self) {
        self.background.background.hide();
        self.background.background.hide();
    }

    fn dead_start(&mut self) {
        self.player.wizard.velocity = (0, -1).into();
        self.player.wizard.sprite.set_priority(Priority::P0);
    }

    fn dead_update(&mut self) -> bool {
        self.timer += 1;

        self.player.wizard.velocity += (0.into(), FixedNumberType::new(1) / 32).into();
        self.player.wizard.position += self.player.wizard.velocity;
        self.player
            .wizard
            .sprite
            .set_tile_id((self.timer / 8 % 2 * 4 + 63 * 4) as u16);

        self.player.wizard.commit_position(self.background.position);

        self.player.wizard.position.y - self.background.position.y < (HEIGHT + 8).into()
    }

    fn update_frame(
        &mut self,
        sfx_player: &mut sfx::SfxPlayer,
        vram: &mut VRamManager,
    ) -> UpdateState {
        self.timer += 1;
        self.input.update();

        let mut player_dead = false;

        self.player.update_frame(
            &self.input,
            self.timer,
            self.background.level,
            &self.enemies,
            sfx_player,
        );

        for enemy in self.enemies.iter_mut() {
            match enemy.update(
                self.background.level,
                self.player.wizard.position,
                self.player.hat_state,
                self.timer,
                sfx_player,
            ) {
                enemies::EnemyUpdateState::KillPlayer => player_dead = true,
                enemies::EnemyUpdateState::None => {}
            }
        }

        self.background.position = self.get_next_map_position();
        self.background.commit_position(vram);

        self.player.wizard.commit_position(self.background.position);
        self.player.hat.commit_position(self.background.position);

        for enemy in self.enemies.iter_mut() {
            enemy.commit(self.background.position);
        }

        player_dead |= self
            .player
            .wizard
            .killision_at_point(self.background.level, self.player.wizard.position);
        if player_dead {
            UpdateState::Dead
        } else if self
            .player
            .wizard
            .completion_at_point(self.background.level, self.player.wizard.position)
        {
            UpdateState::Complete
        } else {
            UpdateState::Normal
        }
    }

    fn get_next_map_position(&self) -> Vector2D<FixedNumberType> {
        // want to ensure the player and the hat are visible if possible, so try to position the map
        // so the centre is at the average position. But give the player some extra priority
        let hat_pos = self.player.hat.position.floor();
        let player_pos = self.player.wizard.position.floor();

        let new_target_position = (hat_pos + player_pos * 3) / 4;

        let screen: Vector2D<i32> = (WIDTH, HEIGHT).into();
        let half_screen = screen / 2;
        let current_centre = self.background.position.floor() + half_screen;

        let mut target_position = ((current_centre * 3 + new_target_position) / 4) - half_screen;

        target_position.x = target_position.x.clamp(
            0,
            (self.background.level.dimensions.x * 8 - (WIDTH as u32)) as i32,
        );
        target_position.y = target_position.y.clamp(
            0,
            (self.background.level.dimensions.y * 8 - (HEIGHT as u32)) as i32,
        );

        target_position.into()
    }
}

#[agb::entry]
fn main(mut agb: agb::Gba) -> ! {
    let (tiled, mut vram) = agb.display.video.tiled0();
    vram.set_background_palettes(tile_sheet::background.palettes);
    let mut world_display = tiled.background(Priority::P0);

    let tile_set_ref = vram.add_tileset(TileSet::new(
        tile_sheet::background.tiles,
        TileFormat::FourBpp,
    ));

    splash_screen::show_splash_screen(
        splash_screen::SplashScreen::Start,
        None,
        None,
        &mut world_display,
        &mut vram,
    );

    loop {
        let mut object = agb.display.object.get();
        let mut timer_controller = agb.timers.timers();
        let mut mixer = agb.mixer.mixer(&mut timer_controller.timer0);

        object.set_sprite_palettes(object_sheet::object_sheet.palettes);
        object.set_sprite_tilemap(object_sheet::object_sheet.tiles);

        for y in 0..32u16 {
            for x in 0..32u16 {
                world_display.set_tile(
                    &mut vram,
                    (x, y).into(),
                    tile_set_ref,
                    TileSetting::from_raw(level_display::BLANK),
                );
            }
        }

        world_display.commit();
        world_display.show();

        object.enable();

        mixer.enable();
        let mut music_box = sfx::MusicBox::new();

        let vblank = agb::interrupt::VBlank::get();
        let mut current_level = 10;

        loop {
            if current_level == map_tiles::LEVELS.len() as u32 {
                break;
            }

            music_box.before_frame(&mut mixer);
            mixer.frame();
            vblank.wait_for_vblank();
            mixer.after_vblank();

            level_display::write_level(
                &mut world_display,
                current_level / 8 + 1,
                current_level % 8 + 1,
                tile_set_ref,
                &mut vram,
            );

            world_display.commit();
            world_display.show();

            music_box.before_frame(&mut mixer);
            mixer.frame();
            vblank.wait_for_vblank();
            mixer.after_vblank();

            vram.set_background_palettes(tile_sheet::background.palettes);

            let mut background = InfiniteScrolledMap::new(
                tiled.background(Priority::P2),
                Box::new(move |pos: Vector2D<i32>| {
                    let level = &map_tiles::LEVELS[current_level as usize];
                    (
                        tile_set_ref,
                        TileSetting::from_raw(
                            *level
                                .background
                                .get((pos.y * level.dimensions.x as i32 + pos.x) as usize)
                                .unwrap_or(&0),
                        ),
                    )
                }),
            );
            let mut foreground = InfiniteScrolledMap::new(
                tiled.background(Priority::P0),
                Box::new(move |pos: Vector2D<i32>| {
                    let level = &map_tiles::LEVELS[current_level as usize];
                    (
                        tile_set_ref,
                        TileSetting::from_raw(
                            *level
                                .foreground
                                .get((pos.y * level.dimensions.x as i32 + pos.x) as usize)
                                .unwrap_or(&0),
                        ),
                    )
                }),
            );

            let mut level = PlayingLevel::open_level(
                &map_tiles::LEVELS[current_level as usize],
                &object,
                &mut background,
                &mut foreground,
                agb::input::ButtonController::new(),
            );

            while level.background.init_background(&mut vram) != PartialUpdateStatus::Done {
                music_box.before_frame(&mut mixer);
                mixer.frame();
                vblank.wait_for_vblank();
                mixer.after_vblank();
            }

            while level.background.init_foreground(&mut vram) != PartialUpdateStatus::Done {
                music_box.before_frame(&mut mixer);
                mixer.frame();
                vblank.wait_for_vblank();
                mixer.after_vblank();
            }

            for _ in 0..20 {
                music_box.before_frame(&mut mixer);
                mixer.frame();
                vblank.wait_for_vblank();
                mixer.after_vblank();
            }

            level.show_backgrounds();

            world_display.hide();

            loop {
                match level
                    .update_frame(&mut sfx::SfxPlayer::new(&mut mixer, &music_box), &mut vram)
                {
                    UpdateState::Normal => {}
                    UpdateState::Dead => {
                        level.dead_start();
                        while level.dead_update() {
                            music_box.before_frame(&mut mixer);
                            mixer.frame();
                            vblank.wait_for_vblank();
                            mixer.after_vblank();
                        }
                        break;
                    }
                    UpdateState::Complete => {
                        current_level += 1;
                        break;
                    }
                }

                music_box.before_frame(&mut mixer);
                mixer.frame();
                vblank.wait_for_vblank();
                mixer.after_vblank();
            }

            level.hide_backgrounds();
        }

        splash_screen::show_splash_screen(
            splash_screen::SplashScreen::End,
            Some(&mut mixer),
            Some(&mut music_box),
            &mut world_display,
            &mut vram,
        );
    }
}
