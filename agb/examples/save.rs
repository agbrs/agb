//! Shows how to use the save subsystem of agb. Saves the current location of the crab
//! at the end of each frame, so when you reload the game, it'll be right back where you left it.
#![no_std]
#![no_main]

use agb::{
    display::{HEIGHT, Palette16, Rgb15, WIDTH, object::Object, tiled::VRAM_MANAGER},
    fixnum::{Num, Vector2D, vec2},
    include_aseprite,
    input::ButtonController,
    save::SaveSlotManager,
};
use serde::{Deserialize, Serialize};

extern crate alloc;

include_aseprite!(
    mod sprites,
    "examples/gfx/crab.aseprite"
);

/// Metadata shown in save slot selection (not used in this simple example)
#[derive(Clone, Serialize, Deserialize)]
struct SaveMetadata;

/// The actual save data - stores the crab's position
#[derive(Clone, Serialize, Deserialize)]
struct SaveData {
    x: i32,
    y: i32,
}

impl SaveData {
    fn position(&self) -> Vector2D<Num<i32, 8>> {
        vec2(Num::from_raw(self.x), Num::from_raw(self.y))
    }

    fn from_position(position: Vector2D<Num<i32, 8>>) -> Self {
        SaveData {
            x: position.x.to_raw(),
            y: position.y.to_raw(),
        }
    }
}

const SAVE_MAGIC: [u8; 32] = *b"agb-example-save________________";

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut button = ButtonController::new();

    // Initialize the save system with 1 slot
    let mut save_manager: SaveSlotManager<SaveMetadata> = gba
        .save
        .init_sram(1, SAVE_MAGIC)
        .expect("Failed to initialize save");

    // Try to load existing save, or start at center
    let mut position: Vector2D<Num<i32, 8>> = save_manager
        .read::<SaveData>(0)
        .ok()
        .map(|data| data.position())
        .unwrap_or_else(|| vec2(WIDTH / 2, HEIGHT / 2).change_base());

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new([0xffff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].map(Rgb15::new)),
    );

    loop {
        let mut frame = gfx.frame();
        button.update();

        position.x += button.x_tri() as i32;
        position.y += button.y_tri() as i32;

        position.x = position.x.clamp(0.into(), (WIDTH - 32).into());
        position.y = position.y.clamp(0.into(), (HEIGHT - 32).into());

        // Save the current position
        let save_data = SaveData::from_position(position);
        save_manager
            .write(0, &save_data, &SaveMetadata)
            .expect("Failed to save");

        Object::new(sprites::IDLE.sprite(0))
            .set_pos(position.floor())
            .show(&mut frame);

        frame.commit();
    }
}
