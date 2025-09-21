//! Async display example
//!
//! Creates a simple color-cycling rectangle that changes colors periodically during
//! VBlank phases that are awaited asynchronously (there is a VBlank interrupt).
//!
//! VBlank (Vertical Blank) prevents screen tearing by ensuring video memory
//! is only modified when the display hardware isn't reading it. This creates
//! smooth, professional-looking animations at 60Hz.
//!
//! ```text
//! |------ FRAME DRAWING ------|___ VBlank ___|------ FRAME DRAWING ------|___ VBlank ___|...
//!```

#![no_std]
#![no_main]

use embassy_agb::{
    agb::display::{
        tiled::{
            DynamicTile16, RegularBackground, RegularBackgroundSize, TileEffect, TileFormat,
            VRAM_MANAGER,
        },
        Palette16, Priority, Rgb15,
    },
    Spawner,
};

#[embassy_agb::main]
async fn main(_spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());
    let mut display = gba.display();

    // Create a palette with distinct colors for the animation
    let palette = Palette16::new([
        Rgb15::new(0x0000), // 0: Black
        Rgb15::new(0x001F), // 1: Blue
        Rgb15::new(0x03E0), // 2: Green
        Rgb15::new(0x7C00), // 3: Red
        Rgb15::new(0x7FFF), // 4: White
        Rgb15::new(0x0000), // 5-15: Black (unused)
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
        Rgb15::new(0x0000),
    ]);

    VRAM_MANAGER.set_background_palettes(&[palette]);

    // Set up the background layer
    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // Create a tile filled with a single color
    let mut color_tile = DynamicTile16::new();
    for y in 0..8 {
        for x in 0..8 {
            color_tile.set_pixel(x, y, 1); // Start with blue
        }
    }

    // Create a centered rectangle using the color tile
    // GBA screen: 240x160 pixels = 30x20 tiles
    // Rectangle: 8x6 tiles, so center at (11, 7) to (18, 12)
    const RECT_WIDTH: i32 = 8;
    const RECT_HEIGHT: i32 = 6;
    const SCREEN_TILES_X: i32 = 30;
    const SCREEN_TILES_Y: i32 = 20;

    let start_x = (SCREEN_TILES_X - RECT_WIDTH) / 2;
    let start_y = (SCREEN_TILES_Y - RECT_HEIGHT) / 2;

    for tile_y in start_y..(start_y + RECT_HEIGHT) {
        for tile_x in start_x..(start_x + RECT_WIDTH) {
            bg.set_tile_dynamic16((tile_x, tile_y), &color_tile, TileEffect::default());
        }
    }

    // Animation state
    let colors = [1u8, 2, 3, 4]; // Blue, Green, Red, White
    let mut color_index = 0;
    let mut frame_count = 0u32;
    const FRAMES_PER_COLOR_CHANGE: u32 = 60; // Change color every second at 60fps

    loop {
        // Wait for VBlank: prevents screen tearing by ensuring we only modify
        // video memory when the display isn't reading it (~60Hz timing)
        display.wait_for_vblank().await;

        // Get frame: acquire rendering context (no additional VBlank wait needed)
        let mut frame = display.frame_no_wait();

        // Render: show background and commit changes to display
        bg.show(&mut frame);
        frame.commit();

        // Animation: change color every 60 frames (1 second at 60fps)
        if frame_count % FRAMES_PER_COLOR_CHANGE == 0 && frame_count > 0 {
            color_index = (color_index + 1) % colors.len();
            let new_color = colors[color_index];

            // Update tile data with new color
            for y in 0..8 {
                for x in 0..8 {
                    color_tile.set_pixel(x, y, new_color);
                }
            }

            // Apply updated tile to the centered rectangle
            for tile_y in start_y..(start_y + RECT_HEIGHT) {
                for tile_x in start_x..(start_x + RECT_WIDTH) {
                    bg.set_tile_dynamic16((tile_x, tile_y), &color_tile, TileEffect::default());
                }
            }
        }

        frame_count += 1;
    }
}
