use super::*;
use crate::{Gba, include_background_gfx, interrupt::VBlank, test_runner::assert_image_output};

include_background_gfx!(crate, agb_logo, test_logo => deduplicate "gfx/test_logo.png");

const WIZARD_FACE_TILE: usize = 19 + 4 * 30;

#[test_case]
fn test_commit_in_basic_case(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();
    VRAM_MANAGER.set_background_palettes(agb_logo::PALETTES);

    let mut bg_data = RegularBackgroundTiles::new(
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

    let mut bg_data = RegularBackgroundTiles::new(
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

    let mut bg_data = RegularBackgroundTiles::new(
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

    let mut bg_data = RegularBackgroundTiles::new(
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
