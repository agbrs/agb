#![no_std]
#![no_main]

use agb::{
    display::{
        GraphicsFrame, HEIGHT, WIDTH,
        object::{Object, Sprite},
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    fixnum::{Num, Rect, Vector2D, num, vec2},
    include_aseprite, include_background_gfx,
    input::{Button, ButtonController},
};

const ACCELERATION: Number = Number::from_raw(1 << 4);
const GRAVITY: Number = Number::from_raw(1 << 4);
const FLAPPING_GRAVITY: Number = Number::from_raw(GRAVITY.to_raw() / 3);
const JUMP_VELOCITY: Number = Number::from_raw(1 << 9);
const TERMINAL_VELOCITY: Number = Number::from_raw(1 << 7);

const CAYOTE_FRAMES: usize = 20;

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

fn tile_is_collidable(tile: Vector2D<i32>) -> bool {
    if tile.x < 0 || tile.x > 32 || tile.y < 0 || tile.y > 32 {
        true
    } else {
        let idx = tile.x + tile.y * 32;
        let tile = background::map.tile_settings[idx as usize].tile_id();

        // I just grabbed the indexes of the 2 collidable tiles from aseprite
        const COLLIDABLE_TILES: &[u16] = &[
            background::map.tile_settings[227].tile_id(),
            background::map.tile_settings[355].tile_id(),
        ];

        COLLIDABLE_TILES.contains(&tile)
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut input = agb::input::ButtonController::new();

    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut background = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for (i, &tile) in background::map.tile_settings.iter().enumerate() {
        let i = i as u16;
        background.set_tile((i % 32, i / 32), &background::map.tiles, tile);
    }

    let mut chicken = Chicken::new(vec2(6, 7));

    let mut frame_count = 0usize;

    loop {
        frame_count = frame_count.wrapping_add(1);

        input.update();

        let mut frame = gfx.frame();
        background.show(&mut frame);
        chicken.update(frame_count, &input, &mut frame);

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

    fn update(&mut self, frame_count: usize, input: &ButtonController, frame: &mut GraphicsFrame) {
        if self.state != State::Ground {
            self.left_ground_frames += 1;
        } else {
            self.left_ground_frames = 0;
        }

        self.velocity.x += ACCELERATION * (input.x_tri() as i32);
        self.velocity.x = (self.velocity.x * 61) / 64;

        if self.left_ground_frames < CAYOTE_FRAMES && input.is_just_pressed(Button::A) {
            self.left_ground_frames = 200;
            self.velocity.y = -JUMP_VELOCITY;
        }

        self.restrict_to_screen();
        self.update_collision();
        self.update_sprite(frame_count);
        self.object.show(frame);
    }

    fn update_sprite(&mut self, frame: usize) {
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
            .set_position((self.position + vec2(num!(0.5), num!(0.5))).floor() - vec2(4, 4));
    }

    fn restrict_to_screen(&mut self) {
        let bounding_rect = Rect::new(
            vec2(num!(4), num!(4)),
            vec2(num!(WIDTH - 8), num!(HEIGHT - 8)),
        );

        self.position = bounding_rect.clamp_point(self.position);
    }

    fn handle_collision_component(
        velocity: &mut Number,
        position: &mut Number,
        collidable: &dyn Fn(i32) -> bool,
    ) {
        let potential = *position + *velocity;
        let potential_external = potential + velocity.to_raw().signum() * 4;

        let target_tile = potential_external.floor() / 8;

        if !collidable(target_tile) {
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
            tile_is_collidable(vec2(x, self.position.y.floor() / 8))
        });
        Self::handle_collision_component(&mut self.velocity.y, &mut self.position.y, &|y| {
            tile_is_collidable(vec2(self.position.x.floor() / 8, y))
        });

        self.state = State::Ground;

        if !tile_is_collidable(
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

include_background_gfx!(background, map => deduplicate "examples/gfx/chicken-map.aseprite");
