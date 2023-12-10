#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![deny(clippy::all)]

extern crate alloc;

use core::ops::Range;

use agb::{
    display::{
        self,
        affine::AffineMatrix,
        object::{
            AffineMatrixInstance, AffineMode, Graphics, OamIterator, ObjectUnmanaged, Sprite,
            SpriteLoader, SpriteVram, Tag,
        },
        palette16::Palette16,
    },
    fixnum::{num, Num, Vector2D},
    include_aseprite,
    input::{Button, ButtonController},
    rng,
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};

type Number = Num<i32, 8>;

struct Saw {
    object: ObjectUnmanaged,
    position: Vector2D<Number>,
    angle: Number,
    rotation_speed: Number,
}

#[derive(Clone, Copy)]
enum Colour {
    Red,
    Blue,
}

struct Circle {
    colour: Colour,
    position: Vector2D<Number>,
}

#[derive(Clone)]
struct SpriteCache {
    saw: SpriteVram,
    blue: SpriteVram,
    red: SpriteVram,
    numbers: Box<[SpriteVram]>,
    bars: [Box<[SpriteVram]>; 2],
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DrawDirection {
    Left,
    Right,
}

fn draw_bar(
    position: Vector2D<i32>,
    length: usize,
    colour: Colour,
    oam: &mut OamIterator,
    sprite_cache: &SpriteCache,
) -> Option<()> {
    let length = length as i32;
    let number_of_sprites = length / 8;
    let size_of_last = length % 8;

    let sprites = match colour {
        Colour::Red => &sprite_cache.bars[0],
        Colour::Blue => &sprite_cache.bars[1],
    };

    for sprite_idx in 0..number_of_sprites {
        let mut object = ObjectUnmanaged::new(sprites[0].clone());
        object
            .show()
            .set_position(position + (sprite_idx * 8, 0).into());
        oam.next()?.set(&object);
    }

    if size_of_last != 0 {
        let mut object = ObjectUnmanaged::new(sprites[8 - size_of_last as usize].clone());
        object
            .show()
            .set_position(position + (number_of_sprites * 8, 0).into());
        oam.next()?.set(&object);
    }

    Some(())
}

fn draw_number(
    mut number: u32,
    position: Vector2D<i32>,
    oam: &mut OamIterator,
    direction: DrawDirection,
    sprite_cache: &SpriteCache,
) -> Option<()> {
    let mut digits = Vec::new();
    if number == 0 {
        digits.push(0);
    }

    while number != 0 {
        digits.push(number % 10);
        number /= 10;
    }

    let mut current_position = if direction == DrawDirection::Right {
        position + (4 * (digits.len() - 1) as i32, 0).into()
    } else {
        position
    };

    for digit in digits {
        let mut obj = ObjectUnmanaged::new(sprite_cache.numbers[digit as usize].clone());
        obj.show().set_position(current_position);

        oam.next()?.set(&obj);

        current_position -= (4, 0).into();
    }

    Some(())
}

impl SpriteCache {
    fn new(loader: &mut SpriteLoader) -> Self {
        static SPRITES: &Graphics = include_aseprite!(
            "gfx/circles.aseprite",
            "gfx/saw.aseprite",
            "gfx/numbers.aseprite",
            "gfx/bar.aseprite"
        );

        fn generate_sprites(
            tag: &'static Tag,
            range: Range<usize>,
            loader: &mut SpriteLoader,
        ) -> Box<[SpriteVram]> {
            range
                .map(|x| tag.sprite(x))
                .map(|x| loader.get_vram_sprite(x))
                .collect::<Vec<_>>()
                .into_boxed_slice()
        }

        static NUMBERS: &Tag = SPRITES.tags().get("numbers");
        static BLUE_CIRCLE: &Sprite = SPRITES.tags().get("Blue").sprite(0);
        static RED_CIRCLE: &Sprite = SPRITES.tags().get("Red").sprite(0);
        static SAW: &Sprite = SPRITES.tags().get("Saw").sprite(0);
        static BAR_RED: &Tag = SPRITES.tags().get("Red Bar");
        static BAR_BLUE: &Tag = SPRITES.tags().get("Blue Bar");

        Self {
            saw: loader.get_vram_sprite(SAW),
            blue: loader.get_vram_sprite(BLUE_CIRCLE),
            red: loader.get_vram_sprite(RED_CIRCLE),
            numbers: generate_sprites(NUMBERS, 0..10, loader),
            bars: [
                generate_sprites(BAR_RED, 0..8, loader),
                generate_sprites(BAR_BLUE, 0..8, loader),
            ],
        }
    }
}

struct Game {
    settings: FinalisedSettings,
    circles: VecDeque<Circle>,
    saws: VecDeque<Saw>,
    head_position: Vector2D<Number>,
    phase_time: Number,
    input: ButtonController,
    frame_since_last_saw: i32,
    alive_frames: u32,
    energy: Number,
}

enum GameState {
    Continue,
    Loss,
}

impl Game {
    fn from_settings(settings: Settings) -> Self {
        let finalised = settings.to_finalised_settings();

        let mut circles = VecDeque::with_capacity(finalised.number_of_circles);
        for idx in 0..finalised.number_of_circles {
            circles.push_back(Circle {
                colour: Colour::Red,
                position: Vector2D::new(
                    finalised.speed * idx as i32 - 4,
                    settings.head_start_position.y,
                ),
            })
        }

        Game {
            input: agb::input::ButtonController::new(),
            energy: finalised.max_energy,
            settings: finalised,
            circles,
            saws: VecDeque::new(),
            head_position: settings.head_start_position,
            phase_time: 0.into(),
            frame_since_last_saw: 0,
            alive_frames: 0,
        }
    }

    fn frame(&mut self, sprite_cache: &SpriteCache) -> GameState {
        self.input.update();

        let (height, colour) = if self.input.is_pressed(Button::A) && self.energy > 0.into() {
            self.energy -= self.settings.energy_use_speed;
            (self.settings.wave_height_ability, Colour::Blue)
        } else {
            if self.input.is_released(Button::A) {
                self.energy += self.settings.energy_recover_speed;
                self.energy = self.energy.min(self.settings.max_energy);
            }
            (self.settings.wave_height_normal, Colour::Red)
        };

        let next_phase_time = self.phase_time + self.settings.phase_speed;

        let this_frame_y_delta = next_phase_time.cos() - self.phase_time.cos();
        self.phase_time = next_phase_time % num!(1.);
        let this_frame_y_delta = this_frame_y_delta * height;
        self.head_position.y += this_frame_y_delta;

        // update circles
        for circle in self.circles.iter_mut() {
            circle.position.x -= self.settings.speed;
        }

        self.circles.pop_front();

        // generate circle
        let circle = Circle {
            colour,
            position: self.head_position,
        };

        self.circles.push_back(circle);

        // update saws + check for death
        let mut saw_has_hit_head = false;
        let mut number_of_saws_to_pop = 0;
        for (idx, saw) in self.saws.iter_mut().enumerate() {
            saw.position.x -= self.settings.speed;
            if saw.position.x < (-32).into() {
                number_of_saws_to_pop = idx + 1;
            }
            saw.angle += saw.rotation_speed;

            let angle_affine_matrix = AffineMatrix::from_rotation(saw.angle);

            saw.object.set_affine_matrix(AffineMatrixInstance::new(
                angle_affine_matrix.to_object_wrapping(),
            ));
            saw.object.show_affine(AffineMode::Affine);

            saw.object
                .set_position(saw.position.floor() - (16, 16).into());

            if (saw.position - self.head_position).magnitude_squared()
                < ((16 + 4) * (16 + 4)).into()
            {
                saw_has_hit_head = true;
            }
        }

        // destroy saws
        for _ in 0..number_of_saws_to_pop {
            self.saws.pop_front();
        }

        // create saw
        self.frame_since_last_saw -= 1;
        if self.frame_since_last_saw <= 0 {
            self.frame_since_last_saw = self.settings.frames_between_saws;
            let mut rotation_direction = rng::gen().signum();
            if rotation_direction == 0 {
                rotation_direction = 1;
            }

            let rotation_magnitude =
                Number::from_raw(rng::gen().abs() % (1 << 8)) % num!(0.02) + num!(0.005);

            let rotation_speed = rotation_magnitude * rotation_direction;
            let saw = Saw {
                object: ObjectUnmanaged::new(sprite_cache.saw.clone()),
                position: (300, rng::gen().rem_euclid(display::HEIGHT)).into(),
                angle: 0.into(),
                rotation_speed,
            };

            self.saws.push_back(saw);
        }

        self.alive_frames += 1;

        let out_of_bounds_death = self.head_position.y.floor() < -4
            || (self.head_position.y + 1).floor() > display::HEIGHT + 4;

        if saw_has_hit_head || out_of_bounds_death {
            GameState::Loss
        } else {
            GameState::Continue
        }
    }

    fn render(&self, oam: &mut OamIterator, sprite_cache: &SpriteCache) -> Option<()> {
        for saw in self.saws.iter() {
            oam.next()?.set(&saw.object);
        }

        for circle in self.circles.iter() {
            let mut object = ObjectUnmanaged::new(match circle.colour {
                Colour::Red => sprite_cache.red.clone(),
                Colour::Blue => sprite_cache.blue.clone(),
            });

            object
                .show()
                .set_position(circle.position.floor() - (4, 4).into());

            oam.next()?.set(&object);
        }

        Some(())
    }
}

struct Settings {
    phase_speed: Number,
    frames_between_saws: i32,
    speed: Number,
    head_start_position: Vector2D<Number>,
    wave_height_normal: Number,
    wave_height_ability: Number,
    max_energy: Number,
    energy_use_speed: Number,
    energy_recover_speed: Number,
}

impl Settings {
    fn to_finalised_settings(&self) -> FinalisedSettings {
        FinalisedSettings {
            number_of_circles: ((self.head_start_position.x + 4) / self.speed + 1)
                .floor()
                .try_into()
                .expect("number should be positive"),
            speed: self.speed,
            phase_speed: self.phase_speed,
            frames_between_saws: self.frames_between_saws,
            wave_height_ability: self.wave_height_ability,
            wave_height_normal: self.wave_height_normal,
            max_energy: self.max_energy,
            energy_recover_speed: self.energy_recover_speed,
            energy_use_speed: self.energy_use_speed,
        }
    }
}

struct FinalisedSettings {
    wave_height_normal: Number,
    wave_height_ability: Number,
    phase_speed: Number,
    frames_between_saws: i32,
    speed: Number,
    number_of_circles: usize,
    max_energy: Number,
    energy_use_speed: Number,
    energy_recover_speed: Number,
}

pub fn main(mut gba: agb::Gba) -> ! {
    let (mut unmanaged, mut sprites) = gba.display.object.get_unmanaged();
    let sprite_cache = SpriteCache::new(&mut sprites);

    let (_background, mut vram) = gba.display.video.tiled0();

    vram.set_background_palettes(&[Palette16::new([u16::MAX; 16])]);

    let vblank = agb::interrupt::VBlank::get();

    let mut max_score = 0;

    loop {
        let mut game = Game::from_settings(Settings {
            phase_speed: num!(0.02),
            frames_between_saws: 60,
            speed: num!(1.),
            head_start_position: (40, 100).into(),
            wave_height_normal: 20.into(),
            wave_height_ability: 5.into(),
            max_energy: 128.into(),
            energy_use_speed: num!(0.5),
            energy_recover_speed: 0.into(),
        });
        loop {
            let state = game.frame(&sprite_cache);
            if game.alive_frames > max_score {
                max_score = game.alive_frames;
            }
            let max_bar_width = display::WIDTH - 2;
            let bar_width_pixels = (game.energy * max_bar_width) / game.settings.max_energy;
            let bar_width_pixels = (bar_width_pixels + num!(0.5)).floor().max(0) as usize;
            vblank.wait_for_vblank();
            let oam_frame = &mut unmanaged.iter();
            draw_number(
                max_score,
                (display::WIDTH - 5, 2).into(),
                oam_frame,
                DrawDirection::Left,
                &sprite_cache,
            );
            draw_number(
                game.alive_frames,
                (2, 2).into(),
                oam_frame,
                DrawDirection::Right,
                &sprite_cache,
            );
            draw_bar(
                (1, 1).into(),
                bar_width_pixels,
                game.circles.back().unwrap().colour,
                oam_frame,
                &sprite_cache,
            );

            game.render(oam_frame, &sprite_cache);

            if matches!(state, GameState::Loss) {
                for _ in 0..30 {
                    vblank.wait_for_vblank();
                }
                break;
            }
        }
    }
}
