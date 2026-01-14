use super::*;
use crate::{
    Gba,
    display::{Palette16, Rgb, Rgb15},
    include_background_gfx,
    interrupt::VBlank,
    test_runner::assert_image_output,
};

include_background_gfx!(mod agb_logo, test_logo => deduplicate "gfx/test_logo.aseprite");

const WIZARD_FACE_TILE: usize = 19 + 4 * 30;

#[test_case]
fn test_commit_in_basic_case(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();
    VRAM_MANAGER.set_background_palettes(agb_logo::PALETTES);

    let mut bg_data = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    bg_data.set_tile(
        (0, 0),
        &agb_logo::test_logo.tiles,
        agb_logo::test_logo.tile_settings[WIZARD_FACE_TILE],
    );

    let mut frame = graphics.frame();
    bg_data.show(&mut frame);

    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output("gfx/test_output/regular_background/test_commit_in_basic_case.png");
}

#[test_case]
fn test_commit_when_background_tiles_are_modified_after_show(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();
    VRAM_MANAGER.set_background_palettes(agb_logo::PALETTES);

    let mut bg_data = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    bg_data.set_tile(
        (5, 5),
        &agb_logo::test_logo.tiles,
        agb_logo::test_logo.tile_settings[WIZARD_FACE_TILE],
    );

    let mut frame = graphics.frame();
    bg_data.show(&mut frame);

    bg_data.set_tile(
        (5, 6),
        &agb_logo::test_logo.tiles,
        agb_logo::test_logo.tile_settings[WIZARD_FACE_TILE],
    );

    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/regular_background/test_commit_when_background_tiles_are_modified_after_show.png",
    );
}

#[test_case]
fn test_commit_when_background_tiles_are_dropped_after_show(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();
    VRAM_MANAGER.set_background_palettes(agb_logo::PALETTES);

    let mut bg_data = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    bg_data.set_tile(
        (5, 5),
        &agb_logo::test_logo.tiles,
        agb_logo::test_logo.tile_settings[WIZARD_FACE_TILE],
    );

    let mut frame = graphics.frame();
    bg_data.show(&mut frame);

    drop(bg_data);

    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/regular_background/test_commit_when_background_tiles_are_dropped_after_show.png",
    );
}

#[test_case]
fn test_commit_when_background_tiles_rendered_twice(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();
    VRAM_MANAGER.set_background_palettes(agb_logo::PALETTES);

    let mut bg_data = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    bg_data.set_tile(
        (5, 5),
        &agb_logo::test_logo.tiles,
        agb_logo::test_logo.tile_settings[WIZARD_FACE_TILE],
    );

    let mut frame = graphics.frame();
    bg_data.show(&mut frame);

    bg_data.set_scroll_pos((16, 16));
    bg_data.show(&mut frame);

    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/regular_background/test_commit_when_background_tiles_rendered_twice.png",
    );
}

#[test_case]
fn can_create_100_backgrounds_one_at_a_time(gba: &mut Gba) {
    let mut gfx = gba.graphics.get();

    for _ in 0..100 {
        let bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background64x64,
            TileFormat::FourBpp,
        );
        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();
    }
}

#[test_case]
fn test_dynamic_tile_16_checkerboard(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();

    // Set up a simple palette with distinct colours
    const RED: Rgb15 = Rgb::new(255, 0, 0).to_rgb15();
    const GREEN: Rgb15 = Rgb::new(0, 255, 0).to_rgb15();
    const BLUE: Rgb15 = Rgb::new(0, 0, 255).to_rgb15();
    const CYAN: Rgb15 = Rgb::new(0, 255, 255).to_rgb15();
    const MAGENTA: Rgb15 = Rgb::new(255, 0, 255).to_rgb15();
    const YELLOW: Rgb15 = Rgb::new(255, 255, 0).to_rgb15();
    const GREY: Rgb15 = Rgb::new(128, 128, 128).to_rgb15();

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new([
            RED,
            GREEN,
            BLUE,
            Rgb15::WHITE,
            CYAN,
            MAGENTA,
            YELLOW,
            GREY,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
        ]),
    );

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // Create a checkerboard pattern tile
    let mut tile = DynamicTile16::new().fill_with(0);
    for y in 0..8 {
        for x in 0..8 {
            let colour = if (x + y) % 2 == 0 { 1 } else { 2 }; // Green and Blue
            tile.set_pixel(x, y, colour);
        }
    }

    // Place the tile in a 4x4 grid
    for y in 0..4 {
        for x in 0..4 {
            bg.set_tile_dynamic16((x as u16, y as u16), &tile, TileEffect::default());
        }
    }

    let mut frame = graphics.frame();
    bg.show(&mut frame);
    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output("gfx/test_output/regular_background/test_dynamic_tile_16_checkerboard.png");
}

#[test_case]
fn test_dynamic_tile_256_gradient(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();

    // Set up a 256-colour gradient palette
    for i in 0..256 {
        let red = ((i % 32) * 8) as u8;
        let green = (((i / 8) % 32) * 8) as u8;
        let blue = (((i / 4) % 32) * 8) as u8;
        VRAM_MANAGER.set_background_palette_colour_256(i, Rgb::new(red, green, blue).to_rgb15());
    }

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::EightBpp,
    );

    // Create tiles with different gradient patterns
    for tile_y in 0..4 {
        for tile_x in 0..4 {
            let mut tile = DynamicTile256::new();
            for y in 0..8 {
                for x in 0..8 {
                    // Create a gradient based on position
                    let colour = ((tile_x * 8 + x) + (tile_y * 8 + y) * 4) as u8;
                    tile.set_pixel(x, y, colour);
                }
            }
            bg.set_tile_dynamic256((tile_x as u16, tile_y as u16), &tile, TileEffect::default());
        }
    }

    let mut frame = graphics.frame();
    bg.show(&mut frame);
    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output("gfx/test_output/regular_background/test_dynamic_tile_256_gradient.png");
}

#[test_case]
fn test_dynamic_tile_16_multiple_tiles(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();

    const RED: Rgb15 = Rgb::new(255, 0, 0).to_rgb15();
    const GREEN: Rgb15 = Rgb::new(0, 255, 0).to_rgb15();
    const BLUE: Rgb15 = Rgb::new(0, 0, 255).to_rgb15();
    const YELLOW: Rgb15 = Rgb::new(255, 255, 0).to_rgb15();
    const CYAN: Rgb15 = Rgb::new(0, 255, 255).to_rgb15();
    const MAGENTA: Rgb15 = Rgb::new(255, 0, 255).to_rgb15();

    VRAM_MANAGER.set_background_palette(
        0,
        &Palette16::new([
            Rgb15::BLACK, // 0 - transparent/black
            RED,          // 1
            GREEN,        // 2
            BLUE,         // 3
            Rgb15::WHITE, // 4
            YELLOW,       // 5
            CYAN,         // 6
            MAGENTA,      // 7
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
            Rgb15::BLACK,
        ]),
    );

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    // Create different patterned tiles
    for tile_idx in 0..8 {
        let mut tile = DynamicTile16::new().fill_with(0);

        // Fill with a solid colour based on tile index
        let colour = (tile_idx % 8) as u8;
        for y in 0..8 {
            for x in 0..8 {
                // Create a border effect
                if x == 0 || x == 7 || y == 0 || y == 7 {
                    tile.set_pixel(x, y, 4); // White border
                } else {
                    tile.set_pixel(x, y, colour);
                }
            }
        }

        let x_pos = tile_idx % 4;
        let y_pos = tile_idx / 4;
        bg.set_tile_dynamic16((x_pos as u16, y_pos as u16), &tile, TileEffect::default());
    }

    let mut frame = graphics.frame();
    bg.show(&mut frame);
    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/regular_background/test_dynamic_tile_16_multiple_tiles.png",
    );
}
