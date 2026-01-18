use crate::{
    Gba,
    display::{
        Priority, Rgb, Rgb15,
        tiled::{
            AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour, DynamicTile256,
            VRAM_MANAGER,
        },
    },
    interrupt::VBlank,
    test_runner::assert_image_output,
};

#[test_case]
fn can_create_100_affine_backgrounds_one_at_a_time(gba: &mut Gba) {
    let mut gfx = gba.graphics.get();

    for _ in 0..100 {
        let bg = AffineBackground::new(
            Priority::P0,
            AffineBackgroundSize::Background64x64,
            AffineBackgroundWrapBehaviour::NoWrap,
        );
        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();
    }
}

#[test_case]
fn test_affine_dynamic_tile_256_checkerboard(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();

    const RED: Rgb15 = Rgb::new(255, 0, 0).to_rgb15();
    const GREEN: Rgb15 = Rgb::new(0, 255, 0).to_rgb15();
    const BLUE: Rgb15 = Rgb::new(0, 0, 255).to_rgb15();

    // Set up a 256-colour palette
    for i in 0..256 {
        let colour = match i {
            0 => Rgb15::BLACK,
            1 => RED,
            2 => GREEN,
            3 => BLUE,
            4 => Rgb15::WHITE,
            _ => {
                let r = ((i % 32) * 8) as u8;
                let g = (((i / 8) % 32) * 8) as u8;
                let b = (((i / 4) % 32) * 8) as u8;
                Rgb::new(r, g, b).to_rgb15()
            }
        };
        VRAM_MANAGER.set_background_palette_colour_256(i, colour);
    }

    let mut bg = AffineBackground::new(
        Priority::P0,
        AffineBackgroundSize::Background16x16,
        AffineBackgroundWrapBehaviour::NoWrap,
    );

    // Create a checkerboard pattern using dynamic tiles
    let mut tile_white = DynamicTile256::new_affine().fill_with(4); // White
    let mut tile_black = DynamicTile256::new_affine().fill_with(0); // Black

    // Add a small pattern to make tiles distinguishable
    for i in 0..4 {
        tile_white.set_pixel(i, i, 1); // Red diagonal on white
        tile_black.set_pixel(7 - i, i, 2); // Green diagonal on black
    }

    // Create a checkerboard pattern
    for y in 0..8 {
        for x in 0..8 {
            if (x + y) % 2 == 0 {
                bg.set_tile_dynamic256((x, y), &tile_white);
            } else {
                bg.set_tile_dynamic256((x, y), &tile_black);
            }
        }
    }

    let mut frame = graphics.frame();
    bg.show(&mut frame);
    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/affine_background/test_affine_dynamic_tile_256_checkerboard.png",
    );
}

#[test_case]
fn test_affine_dynamic_tile_256_gradient(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();

    // Set up a gradient palette
    for i in 0..256 {
        let r = ((i % 32) * 8) as u8;
        let g = (((i / 4) % 32) * 8) as u8;
        let b = (((i / 2) % 32) * 8) as u8;
        VRAM_MANAGER.set_background_palette_colour_256(i, Rgb::new(r, g, b).to_rgb15());
    }

    let mut bg = AffineBackground::new(
        Priority::P0,
        AffineBackgroundSize::Background16x16,
        AffineBackgroundWrapBehaviour::NoWrap,
    );

    // Create tiles with gradient patterns
    for tile_y in 0..4 {
        for tile_x in 0..4 {
            let mut tile = DynamicTile256::new_affine();
            for y in 0..8 {
                for x in 0..8 {
                    // Create a gradient based on global position
                    let global_x = tile_x * 8 + x;
                    let global_y = tile_y * 8 + y;
                    let colour = ((global_x + global_y * 8) % 256) as u8;
                    tile.set_pixel(x, y, colour);
                }
            }
            bg.set_tile_dynamic256((tile_x as i32, tile_y as i32), &tile);
        }
    }

    let mut frame = graphics.frame();
    bg.show(&mut frame);
    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/affine_background/test_affine_dynamic_tile_256_gradient.png",
    );
}

#[test_case]
fn test_affine_dynamic_tile_256_border_pattern(gba: &mut Gba) {
    let vblank = VBlank::get();
    vblank.wait_for_vblank();

    let mut graphics = gba.graphics.get();

    const RED: Rgb15 = Rgb::new(255, 0, 0).to_rgb15();
    const GREEN: Rgb15 = Rgb::new(0, 255, 0).to_rgb15();
    const BLUE: Rgb15 = Rgb::new(0, 0, 255).to_rgb15();
    const YELLOW: Rgb15 = Rgb::new(255, 255, 0).to_rgb15();
    const CYAN: Rgb15 = Rgb::new(0, 255, 255).to_rgb15();
    const MAGENTA: Rgb15 = Rgb::new(255, 0, 255).to_rgb15();

    // Set up palette with distinct colours
    VRAM_MANAGER.set_background_palette_colour_256(0, Rgb15::BLACK);
    VRAM_MANAGER.set_background_palette_colour_256(1, RED);
    VRAM_MANAGER.set_background_palette_colour_256(2, GREEN);
    VRAM_MANAGER.set_background_palette_colour_256(3, BLUE);
    VRAM_MANAGER.set_background_palette_colour_256(4, Rgb15::WHITE);
    VRAM_MANAGER.set_background_palette_colour_256(5, YELLOW);
    VRAM_MANAGER.set_background_palette_colour_256(6, CYAN);
    VRAM_MANAGER.set_background_palette_colour_256(7, MAGENTA);

    let mut bg = AffineBackground::new(
        Priority::P0,
        AffineBackgroundSize::Background16x16,
        AffineBackgroundWrapBehaviour::NoWrap,
    );

    // Create tiles with border patterns and different fill colours
    for tile_idx in 0..16 {
        let mut tile = DynamicTile256::new_affine();
        let fill_colour = (tile_idx % 8) as u8;
        let border_colour = 4u8; // White border

        for y in 0..8 {
            for x in 0..8 {
                if x == 0 || x == 7 || y == 0 || y == 7 {
                    tile.set_pixel(x, y, border_colour);
                } else {
                    tile.set_pixel(x, y, fill_colour);
                }
            }
        }

        let x_pos = tile_idx % 4;
        let y_pos = tile_idx / 4;
        bg.set_tile_dynamic256((x_pos, y_pos), &tile);
    }

    let mut frame = graphics.frame();
    bg.show(&mut frame);
    frame.commit();
    vblank.wait_for_vblank();

    assert_image_output(
        "gfx/test_output/affine_background/test_affine_dynamic_tile_256_border_pattern.png",
    );
}
