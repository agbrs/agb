//! This example uses blend to make a thunder effect fill the screen, but not change
//! the hud or make the character sprites change.
//!
//! This is an extension on the `HUD` example.
#![no_main]
#![no_std]

use core::ops::Range;

use agb::{
    display::{
        GraphicsFrame, Priority,
        object::{Object, SpriteVram},
        tiled::{
            RegularBackground, RegularBackgroundId, RegularBackgroundSize, TileFormat, VRAM_MANAGER,
        },
    },
    fixnum::{Num, Vector2D, num, vec2},
    include_aseprite, include_background_gfx, include_wav,
    input::ButtonController,
    rng,
    sound::mixer::{Frequency, Mixer, SoundChannel, SoundData},
};

include_aseprite!(mod sprites, "examples/gfx/crab.aseprite");
include_background_gfx!(mod backgrounds,
    BEACH => deduplicate "examples/gfx/beach-background-rain.aseprite",
    HUD => deduplicate "examples/gfx/hud.aseprite",
);

const HEALTH_TEXT: Range<usize> = 0..4;
const HEART_EMPTY: usize = 4;
const HEART_FULL: usize = 5;

// Just 1s of generated static
static RAIN: SoundData = include_wav!("examples/sfx/rain.wav");
// From here: https://opengameart.org/content/100-cc0-sfx-2 (CC0)
// For a real game, you might want a few different options for thunder effects, because just one
// sounds quite repetitive.
static THUNDER: SoundData = include_wav!("examples/sfx/thunder.wav");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Set up the background palettes as needed. These are produced by the include_background_gfx! macro call above.
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    // Get access to the graphics struct which is used to manage the frame lifecycle
    let mut gfx = gba.graphics.get();

    let mut player = Player::new(vec2(num!(100.), num!(100.)));
    let mut button_controller = ButtonController::new();

    let mut bg_tiles = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    bg_tiles.fill_with(&backgrounds::BEACH);

    let mut hud_tiles = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    mixer.enable();
    let mut rain_channel = SoundChannel::new(RAIN);
    rain_channel.should_loop().volume(num!(0.15));
    mixer.play_sound(rain_channel);

    // offset the hud by half a tile
    hud_tiles.set_scroll_pos((-4, -4));

    populate_hud(&mut hud_tiles);

    let mut thunder = ThunderEffect::new();

    loop {
        button_controller.update();
        thunder.update(&mut mixer);

        // Update all entities in the game. In this case it is just the player, but in
        // larger games there could be more things to update.
        player.update(&button_controller);

        // Create the GraphicsFrame
        let mut frame = gfx.frame();

        // Call `.show()` on everything we want to show in this frame. If you don't call `.show()`
        // on something, it won't be visible for this frame.
        player.show(&mut frame);
        let bg_id = bg_tiles.show(&mut frame);
        hud_tiles.show(&mut frame);
        thunder.show(&mut frame, bg_id);

        // `.commit()` on frame will ensure that everything is drawn to the screen, and also wait
        // for the frame to finish rendering before returning control.
        mixer.frame();
        frame.commit();
    }
}

#[derive(Clone, Copy, Default)]
enum ThunderStatus {
    Flash(Num<u8, 4>),
    Wait(i16),
    Rumble,
    #[default]
    None,
}

impl ThunderStatus {
    fn start(&mut self) {
        *self = ThunderStatus::Flash(num!(1));
    }

    fn update(&mut self, mixer: &mut Mixer) {
        match self {
            ThunderStatus::Flash(num) => {
                *num -= Num::from_raw(1); // the smallest positive number
                if *num == num!(0) {
                    *self = ThunderStatus::Wait(((rng::next_i32() as u16) % 128) as i16);
                }
            }
            ThunderStatus::Wait(i) => {
                *i -= 1;
                if *i == 0 {
                    *self = ThunderStatus::Rumble;
                }
            }
            ThunderStatus::Rumble => {
                mixer.play_sound(SoundChannel::new(THUNDER));
                *self = ThunderStatus::None;
            }
            ThunderStatus::None => {}
        }
    }

    fn show(&self, frame: &mut GraphicsFrame, bg_id: RegularBackgroundId) {
        if let ThunderStatus::Flash(value) = self {
            frame
                .blend()
                .brighten(*value)
                .layer()
                .enable_background(bg_id);
        }
    }
}

struct ThunderEffect {
    next_thunder: usize,
    status: ThunderStatus,
}

impl ThunderEffect {
    fn new() -> Self {
        Self {
            next_thunder: 60 * 5, // start the next one within 5 seconds
            status: Default::default(),
        }
    }

    fn update(&mut self, mixer: &mut Mixer) {
        self.status.update(mixer);

        if matches!(self.status, ThunderStatus::None) {
            self.next_thunder -= 1;
            if self.next_thunder <= (rng::next_i32() as usize) % 256 {
                self.next_thunder = 60 * 10;
                self.status.start();
            }
        }
    }

    fn show(&self, frame: &mut GraphicsFrame, bg_id: RegularBackgroundId) {
        self.status.show(frame, bg_id);
    }
}

struct Player {
    sprite: SpriteVram,
    location: Vector2D<Num<i32, 4>>,
}

impl Player {
    pub fn new(initial_location: Vector2D<Num<i32, 4>>) -> Self {
        let sprite = sprites::IDLE.sprite(0).into();

        Self {
            sprite,

            location: initial_location,
        }
    }

    pub fn update(&mut self, button_controller: &ButtonController) {
        self.location += button_controller.vector::<Num<i32, 4>>() * num!(0.5);
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(self.sprite.clone())
            .set_pos(self.location.floor())
            .set_priority(Priority::P3) // draw below the HUD
            .show(frame);
    }
}

fn populate_hud(hud_tiles: &mut RegularBackground) {
    // write out 'HEALTH:'

    for i in HEALTH_TEXT {
        hud_tiles.set_tile(
            (i as i32, 0),
            &backgrounds::HUD.tiles,
            backgrounds::HUD.tile_settings[i],
        );
    }

    let life_amount = 3;
    let total_hearts = 5;

    // start drawing the hearts at position 6
    for i in 0..total_hearts {
        let tile_index = if i < life_amount {
            HEART_FULL
        } else {
            HEART_EMPTY
        };

        hud_tiles.set_tile(
            (6 + i, 0),
            &backgrounds::HUD.tiles,
            backgrounds::HUD.tile_settings[tile_index],
        );
    }
}
