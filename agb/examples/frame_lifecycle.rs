#![no_std]
#![no_main]

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
include_background_gfx!(mod background, beach => deduplicate "examples/gfx/beach-background.aseprite");

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
            .set_position(self.location.floor())
            .show(frame);
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Set up the background palettes as needed. These are produced by the include_background_gfx! macro call above.
    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    // Get access to the graphics struct which is used to manage the frame lifecycle
    let mut gfx = gba.graphics.get();

    let mut player = Player::new(vec2(num!(100.), num!(100.)));
    let mut button_controller = ButtonController::new();

    let mut bg_tiles = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    bg_tiles.fill_with(&background::beach);

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

        // `.commit()` on frame will ensure that everything is drawn to the screen, and also wait
        // for the frame to finish rendering before returning control.
        frame.commit();
    }
}
