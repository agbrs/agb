//! Shows how to use multiple backgrounds to create a background for the character to walk around
//! and a heads-up-display (HUD) which always shows above the character.
#![no_std]
#![no_main]

use core::ops::Range;

use agb::{
    display::{
        GraphicsFrame, Priority,
        object::{Object, SpriteVram},
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    fixnum::{Num, Vector2D, num, vec2},
    include_aseprite, include_background_gfx,
    input::ButtonController,
};

include_aseprite!(mod sprites, "examples/gfx/crab.aseprite");
include_background_gfx!(mod backgrounds,
    BEACH => deduplicate "examples/gfx/beach-background.aseprite",
    HUD => deduplicate "examples/gfx/hud.aseprite",
);

const HEALTH_TEXT: Range<usize> = 0..4;
const HEART_EMPTY: usize = 4;
const HEART_FULL: usize = 5;

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

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Set up the background palettes as needed. These are produced by the include_background_gfx! macro call above.
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    // Get access to the graphics struct which is used to manage the frame lifecycle
    let mut gfx = gba.graphics.get();

    let mut player = Player::new(vec2(num!(100.), num!(100.)));
    let mut button_controller = ButtonController::new();

    let mut bg_tiles = RegularBackgroundTiles::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    bg_tiles.fill_with(&backgrounds::BEACH);

    let mut hud_tiles = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // offset the hud by half a tile
    hud_tiles.set_scroll_pos((-4, -4));

    populate_hud(&mut hud_tiles);

    loop {
        button_controller.update();

        // Update all entities in the game. In this case it is just the player, but in
        // larger games there could be more things to update.
        player.update(&button_controller);

        // Create the GraphicsFrame
        let mut frame = gfx.frame();

        // Call `.show()` on everything we want to show in this frame. If you don't call `.show()`
        // on something, it won't be visible for this frame.
        player.show(&mut frame);
        bg_tiles.show(&mut frame);
        hud_tiles.show(&mut frame);

        // `.commit()` on frame will ensure that everything is drawn to the screen, and also wait
        // for the frame to finish rendering before returning control.
        frame.commit();
    }
}

fn populate_hud(hud_tiles: &mut RegularBackgroundTiles) {
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
