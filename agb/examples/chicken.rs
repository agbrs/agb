//! An example of a very simple platformer game where you can run around and
//! jump playing as a chicken character. This was the first game ever made using
//! the `agb` crate.
#![no_std]
#![no_main]

use agb::{
    display::{
        GraphicsFrame, HEIGHT, WIDTH,
        object::{Object, Sprite},
        tiled::{
            InfiniteScrolledMap, RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER,
        },
    },
    fixnum::{Num, Rect, Vector2D, num, vec2},
    include_aseprite, include_background_gfx,
    input::{Button, ButtonController},
};

// Try modifying these constants to see how the gameplay changes
const ACCELERATION: Number = Number::from_raw(1 << 4);
const GRAVITY: Number = Number::from_raw(1 << 4);
const FLAPPING_GRAVITY: Number = Number::from_raw(GRAVITY.to_raw() / 3);
const JUMP_VELOCITY: Number = Number::from_raw(1 << 9);
const TERMINAL_VELOCITY: Number = Number::from_raw(1 << 7);

// This is the number of frames of grace period you have after walking off a
// surface before jumping no longer works
const COYOTE_FRAMES: usize = 20;

#[derive(PartialEq, Eq)]
enum State {
    Ground,
    Upwards,
    Flapping,
}

type Number = Num<i32, 8>;
type Vector = Vector2D<Num<i32, 8>>;

struct Chicken {
    left_ground_frames: usize,
    state: State,
    object: Object,
    position: Vector,
    velocity: Vector,
}

const MAP_WIDTH: i32 = background::map.width as i32;
const MAP_HEIGHT: i32 = background::map.height as i32;

fn tile_is_colliding(tile: Vector2D<i32>) -> bool {
    if tile.x < 0 || tile.x > MAP_WIDTH || tile.y < 0 || tile.y > MAP_HEIGHT {
        true
    } else {
        let idx = tile.x + tile.y * MAP_WIDTH;
        let tile = background::map.tile_settings[idx as usize].tile_id();

        // I just grabbed the indexes of the 2 colliding tiles from aseprite
        const COLLIDING_TILES: &[u16] = &[
            background::map.tile_settings[1053].tile_id(),
            background::map.tile_settings[1653].tile_id(),
        ];

        COLLIDING_TILES.contains(&tile)
    }
}

#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    entry(gba);
}

fn entry(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut input = agb::input::ButtonController::new();

    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let background = RegularBackground::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut background = InfiniteScrolledMap::new(background);

    let mut chicken = Chicken::new(vec2(6, 7));

    let mut camera_position: Vector = (0, 0).into();

    let mut frame_count = 0usize;

    loop {
        frame_count = frame_count.wrapping_add(1);

        input.update();

        let target_position = chicken.position.x.floor() / WIDTH * WIDTH;
        camera_position.x += (Number::new(target_position) - camera_position.x) / 8;

        background.set_scroll_pos(
            vec2(camera_position.x.round(), camera_position.y.floor()),
            |pos| {
                let (x, y) = (pos.x, pos.y);
                let tile_idx = if !(0..MAP_WIDTH).contains(&x) || !(0..20).contains(&y) {
                    0
                } else {
                    (x + y * MAP_WIDTH) as usize
                };

                (
                    &background::map.tiles,
                    background::map.tile_settings[tile_idx],
                )
            },
        );

        let mut frame = gfx.frame();
        background.show(&mut frame);
        chicken.update(frame_count, &input, camera_position, &mut frame);

        frame.commit();
    }
}

impl Chicken {
    fn new(tile: Vector2D<i32>) -> Self {
        Self {
            left_ground_frames: 0,
            state: State::Ground,
            object: Object::new(IDLE),
            position: (tile * 8 - (0, 4).into()).change_base(),
            velocity: (0, 0).into(),
        }
    }

    fn update(
        &mut self,
        frame_count: usize,
        input: &ButtonController,
        camera_position: Vector,
        frame: &mut GraphicsFrame,
    ) {
        if self.state != State::Ground {
            self.left_ground_frames += 1;
        } else {
            self.left_ground_frames = 0;
        }

        self.velocity.x += ACCELERATION * (input.x_tri() as i32);
        self.velocity.x = (self.velocity.x * 61) / 64;

        if self.left_ground_frames < COYOTE_FRAMES && input.is_just_pressed(Button::A) {
            self.left_ground_frames = 200;
            self.velocity.y = -JUMP_VELOCITY;
        }

        self.restrict_to_screen();
        self.update_collision();
        self.update_sprite(frame_count, camera_position);
        self.object.show(frame);
    }

    fn update_sprite(&mut self, frame: usize, camera_position: Vector) {
        if self.velocity.x.to_raw() > 1 {
            self.object.set_hflip(false);
        } else if self.velocity.x.to_raw() < -1 {
            self.object.set_hflip(true);
        }

        match self.state {
            State::Ground => {
                if self.velocity.x.abs() > ACCELERATION {
                    self.object.set_sprite(WALK.animation_sprite(frame / 10));
                } else {
                    self.object.set_sprite(IDLE);
                }
            }
            State::Upwards => {}
            State::Flapping => {
                self.object.set_sprite(JUMP.animation_sprite(frame / 5));
            }
        }

        self.object
            .set_pos((self.position - camera_position).round() - vec2(4, 4));
    }

    fn restrict_to_screen(&mut self) {
        let bounding_rect = Rect::new(
            vec2(num!(4), num!(4)),
            vec2(num!(MAP_WIDTH * 8 - 8), num!(HEIGHT - 8)),
        );

        let previous = self.position;
        self.position = bounding_rect.clamp_point(self.position);
        if previous.x != self.position.x {
            self.velocity.x = num!(0);
        }
        if previous.y != self.position.y {
            self.velocity.y = num!(0);
        }
    }

    fn handle_collision_component(
        velocity: &mut Number,
        position: &mut Number,
        colliding: &dyn Fn(i32) -> bool,
    ) {
        let potential = *position + *velocity;
        let potential_external = potential + velocity.to_raw().signum() * 4;

        let target_tile = potential_external.floor() / 8;

        if !colliding(target_tile) {
            *position = potential;
        } else {
            let center_of_target_tile = target_tile * 8 + 4;
            let center_of_tile_with_chicken =
                center_of_target_tile - velocity.to_raw().signum() * 8;
            *position = center_of_tile_with_chicken.into();
            *velocity = 0.into();
        }
    }

    fn update_collision(&mut self) {
        Self::handle_collision_component(&mut self.velocity.x, &mut self.position.x, &|x| {
            tile_is_colliding(vec2(x, self.position.y.floor() / 8))
        });
        Self::handle_collision_component(&mut self.velocity.y, &mut self.position.y, &|y| {
            tile_is_colliding(vec2(self.position.x.floor() / 8, y))
        });

        self.state = State::Ground;

        if !tile_is_colliding(
            (self.position + vec2(0.into(), Number::new(4) + Number::from_raw(1))).floor() / 8,
        ) {
            if self.velocity.y < 0.into() {
                self.state = State::Upwards;
                self.velocity.y += GRAVITY;
            } else {
                self.state = State::Flapping;
                self.velocity.y += FLAPPING_GRAVITY;
                if self.velocity.y > TERMINAL_VELOCITY {
                    self.velocity.y = TERMINAL_VELOCITY;
                }
            }
        }
    }
}

include_aseprite!(mod sprites, "examples/gfx/chicken.aseprite");
use sprites::{JUMP, WALK};
static IDLE: &Sprite = sprites::IDLE.sprite(0);

include_background_gfx!(mod background, "7e8dd2", map => deduplicate "examples/gfx/chicken-map.aseprite");
