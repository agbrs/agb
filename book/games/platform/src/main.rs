// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        GraphicsFrame, Priority,
        object::{Object, SpriteVram},
        tiled::{
            InfiniteScrolledMap, RegularBackground, RegularBackgroundSize, TileFormat, TileSetting,
            VRAM_MANAGER,
        },
    },
    fixnum::{Num, Rect, Vector2D, num, vec2},
    include_aseprite, include_background_gfx,
    input::{Button, ButtonController},
};

extern crate alloc;

impl Level {
    fn bounds(&self) -> Rect<i32> {
        Rect::new(
            vec2(0, 0),
            vec2(self.width as i32 - 1, self.height as i32 - 1),
        )
    }
}

impl Level {
    fn collides(&self, tile: Vector2D<i32>) -> bool {
        if !self.bounds().contains_point(tile) {
            return false;
        }

        let idx = (tile.x + tile.y * self.width as i32) as usize;

        self.collision_map[idx / 8] & (1 << (idx % 8)) != 0
    }

    fn wins(&self, tile: Vector2D<i32>) -> bool {
        if !self.bounds().contains_point(tile) {
            return false;
        }

        let idx = (tile.x + tile.y * self.width as i32) as usize;

        self.winning_map[idx / 8] & (1 << (idx % 8)) != 0
    }
}

include_background_gfx!(mod tiles, "2ce8f4", TILES => "gfx/tilesheet.png");
include_aseprite!(mod sprites, "gfx/sprites.aseprite");

struct Level {
    width: u32,
    height: u32,
    background: &'static [TileSetting],
    collision_map: &'static [u8],
    winning_map: &'static [u8],
    player_start: (i32, i32),
}

mod levels {
    use super::Level;
    use agb::display::tiled::TileSetting;
    static TILES: &[TileSetting] = super::tiles::TILES.tile_settings;

    include!(concat!(env!("OUT_DIR"), "/levels.rs"));
}

// define a common set of number and vector type to use throughout
type Number = Num<i32, 8>;
type Vector = Vector2D<Number>;

struct Player {
    position: Vector,
    velocity: Vector,
    frame: usize,
    sprite: SpriteVram,
    flipped: bool,
}

impl Player {
    fn new(start: Vector2D<i32>) -> Self {
        Player {
            position: start.change_base(),
            velocity: (0, 0).into(),
            frame: 0,
            sprite: sprites::STANDING.sprite(0).into(),
            flipped: false,
        }
    }

    fn is_on_ground(&self, level: &Level) -> bool {
        level.collides(vec2(self.position.x, self.position.y + 8 + Number::from_raw(1)).floor() / 8)
    }

    fn handle_horizontal_input(&mut self, x_tri: i32, on_ground: bool) {
        let mut x = x_tri;

        if x_tri.signum() != self.velocity.x.to_raw().signum() {
            x *= 2;
        }

        if on_ground {
            x *= 2;
        }

        self.velocity.x += Number::new(x) / 16;
    }

    fn handle_jump(&mut self) {
        self.velocity.y = Number::new(-2);
    }

    fn handle_collision_component(
        velocity: &mut Number,
        position: &mut Number,
        half_width: i32,
        colliding: &dyn Fn(i32) -> bool,
    ) {
        let potential = *position + *velocity;
        let potential_external = potential + velocity.to_raw().signum() * half_width;

        let target_tile = potential_external.floor() / 8;

        if !colliding(target_tile) {
            *position = potential;
        } else {
            let center_of_target_tile = target_tile * 8 + 4;
            let player_position =
                center_of_target_tile - velocity.to_raw().signum() * (4 + half_width);
            *position = player_position.into();
            *velocity = 0.into();
        }
    }

    fn handle_collision(&mut self, level: &Level) {
        Self::handle_collision_component(&mut self.velocity.x, &mut self.position.x, 4, &|x| {
            level.collides(vec2(x, self.position.y.floor() / 8))
        });
        Self::handle_collision_component(&mut self.velocity.y, &mut self.position.y, 8, &|y| {
            level.collides(vec2(self.position.x.floor() / 8, y))
        });
    }

    fn update_sprite(&mut self) {
        self.frame += 1;

        if self.velocity.x > num!(0.1) {
            self.flipped = false;
        }
        if self.velocity.x < num!(-0.1) {
            self.flipped = true;
        }

        self.sprite = if self.velocity.y < num!(-0.1) {
            sprites::JUMPING.animation_frame(&mut self.frame, 2)
        } else if self.velocity.y > num!(0.1) {
            sprites::FALLING.animation_frame(&mut self.frame, 2)
        } else if self.velocity.x.abs() > num!(0.05) {
            sprites::WALKING.animation_frame(&mut self.frame, 2)
        } else {
            sprites::STANDING.animation_frame(&mut self.frame, 2)
        }
        .into()
    }

    fn update(&mut self, input: &ButtonController, level: &Level) {
        let on_ground = self.is_on_ground(level);

        self.handle_horizontal_input(input.x_tri() as i32, on_ground);

        if input.is_just_pressed(Button::A) && on_ground {
            self.handle_jump();
        }

        self.velocity.y += num!(0.05);
        self.velocity.x *= 15;
        self.velocity.x /= 16;
        self.handle_collision(level);

        self.update_sprite();
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(self.sprite.clone())
            .set_hflip(self.flipped)
            .set_pos(self.position.round() - vec2(8, 8))
            .show(frame);
    }

    fn has_won(&self, level: &Level) -> bool {
        level.wins(self.position.floor() / 8)
    }
}

struct World {
    level: &'static Level,
    player: Player,
    bg: InfiniteScrolledMap,
}

impl World {
    fn new(level: &'static Level) -> Self {
        let bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let bg = InfiniteScrolledMap::new(bg);

        World {
            level,
            bg,
            player: Player::new(level.player_start.into()),
        }
    }

    fn set_pos(&mut self, pos: Vector2D<i32>) {
        self.bg.set_scroll_pos(pos, |pos| {
            let tile = if self.level.bounds().contains_point(pos) {
                let idx = pos.x + pos.y * self.level.width as i32;
                self.level.background[idx as usize]
            } else {
                TileSetting::BLANK
            };

            (&tiles::TILES.tiles, tile)
        });
    }

    fn update(&mut self, input: &ButtonController) {
        self.set_pos(vec2(0, 0));

        self.player.update(input, self.level);
    }

    fn has_won(&self) -> bool {
        self.player.has_won(self.level)
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
        self.player.show(frame);
    }
}

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let mut level = 0;
    let mut world = World::new(levels::LEVELS[level]);
    let mut input = ButtonController::new();

    loop {
        input.update();
        world.update(&input);

        let mut frame = gfx.frame();

        world.show(&mut frame);

        frame.commit();

        if world.has_won() {
            level += 1;
            level %= levels::LEVELS.len();
            world = World::new(levels::LEVELS[level]);
        }
    }
}

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}
