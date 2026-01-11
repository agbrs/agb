#![no_main]
#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
//! Coordinate system
//! =================
//!
//! There are 3 coordinate systems which are used here:
//!
//! 1. World coordinates
//!    A simple (x, y) grid where the game logic lives. Each cell is one logical tile.
//! 2. Macro tile coordinates
//!    A macro tile is 4x2 GBA tiles (32x16 pixels) and is centred around a single world
//!    coordinate tile.
//!
//!    To convert between world coordinates to macro coordinates, use
//!    ```text   
//!    macro_x = world_x - world_y
//!    macro_y = world_x + world_y
//!
//!    world_x = (macro_x + macro_y) / 2
//!    world_y = (macro_y - macro_x) / 2
//!    ```
//!
//!    Half of world tiles are found in the corners of macro tiles. These are called 'ghost tiles'
//!    since they are only rendered as a side-effect of rendering the central tile of the macro
//!    tile.
//!
//! 3. GBA tile coordinates
//!    One macro tile = 4×2 GBA
//!
//! Quadrants
//! =========
//!
//! Each macro tile is divided into 4 quadrants, each 2x1 GBA tiles:
//!
//! ```text
//! ┌──┬──┬──┬──┐
//! │ TL  │ TR  │
//! ├──┼──┼──┼──┤
//! │ BL  │ BR  │
//! └──┴──┴──┴──┘
//! ```
//!
//! Each quadrant sits on the boundary between the central tile and one ghost tile:
//!
//! ```text
//!   TL                 TR
//!       ┌─────┬─────┐
//!       │   /    \  │
//!       │  /  me  \ │
//!       ├     ┼     ┤
//!       │  \      / │
//!       │   \    /  │
//!       └─────┴─────┘
//!   BL                 BR
//! ```
//!
//! This means each quadrant can be rendered using only local information:
//! - `me`: the central tile of this macro tile
//! - `them`: the ghost tile this quadrant borders
//!
//! The `neighbours` context provides additional tiles needed for wall rendering
//! and fixing 1px seams at tile edges.

use agb::{
    Gba,
    display::{
        Priority, Rgb15,
        tiled::{RegularBackground, RegularBackgroundSize, TileEffect, TileFormat, VRAM_MANAGER},
    },
    dma::HBlankDma,
    fixnum::{num, vec2},
    include_aseprite, include_background_gfx, include_colours,
    input::ButtonController,
};

use alloc::vec;

use isometric_render::TileType;

use crate::{
    character::Character,
    isometric_render::{Map, TileCache},
};

extern crate alloc;

mod character;
mod isometric_render;

include_background_gfx!(mod tiles, "333333",
    ISOMETRIC => "gfx/isometric_tiles.aseprite"
);

include_aseprite!(mod sprites, "gfx/kaiju.aseprite");

static SKY_GRADIENT: [Rgb15; 160] = include_colours!("gfx/sky-background-gradient.aseprite");

#[agb::entry]
fn entry(gba: Gba) -> ! {
    main(gba);
}

fn main(mut gba: Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let mut floor_bg = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    let mut wall_bg = RegularBackground::new(
        Priority::P2,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    wall_bg.set_scroll_pos((0, 7));

    let mut gfx = gba.graphics.get();

    let mut tile_cache = TileCache::default();

    let (lower_layer, upper_layer) = {
        use TileType::Air as A;
        use TileType::Dirt as D;
        use TileType::Water as W;

        #[rustfmt::skip]
        let upper_layer = vec![
            A, A, A, A, A, A, A, A, A, A, A, A, A,
            A, D, A, A, A, D, A, A, A, D, D, A, A,
            A, D, A, A, D, D, A, A, A, D, D, A, A,
            A, D, A, A, D, A, A, A, A, D, D, A, A,
            A, D, A, A, A, A, A, A, A, D, D, A, A,
            A, A, A, A, A, A, A, A, A, A, A, A, A,
        ];

        #[rustfmt::skip]
        let lower_layer = vec![
            D, D, D, D, D, D, D, D, D, D, D, W, W,
            D, D, D, D, D, D, D, D, D, D, D, W, W,
            D, D, D, D, D, D, D, A, D, D, D, D, D,
            D, D, D, D, D, D, D, A, D, D, D, D, D,
            D, D, D, D, W, W, D, D, D, D, D, D, D,
            D, D, D, D, W, W, D, D, D, D, D, D, D,
        ];

        (lower_layer, upper_layer)
    };

    let floor_map = Map::new(13, 6, lower_layer);
    let wall_map = Map::new(13, 6, upper_layer);

    for y in 0..32 {
        for x in 0..16 {
            let pos = vec2(x * 2, y);

            for (i, tile) in tile_cache.get_tiles(&floor_map, pos).iter().enumerate() {
                floor_bg.set_tile_dynamic16((x * 2 + i as i32, y), tile, TileEffect::default());
            }

            for (i, tile) in tile_cache.get_tiles(&wall_map, pos).iter().enumerate() {
                wall_bg.set_tile_dynamic16((x * 2 + i as i32, y), tile, TileEffect::default());
            }
        }
    }

    let initial_position = vec2(num!(6), num!(3));
    let mut character = Character::new(&sprites::KAIJU, initial_position);

    let mut input = ButtonController::new();

    agb::println!("Cache size: {}", tile_cache.cache_size());

    loop {
        input.update();
        character.update(&input, &wall_map, &floor_map);

        let mut frame = gfx.frame();

        let floor_id = floor_bg.show(&mut frame);
        wall_bg.show(&mut frame);

        HBlankDma::new(
            VRAM_MANAGER.background_palette_colour_dma(0, 0),
            &SKY_GRADIENT,
        )
        .show(&mut frame);

        character.show(&mut frame, &wall_map);

        frame
            .blend()
            .object_transparency(num!(0.5), num!(0.5))
            .enable_background(floor_id);

        frame.commit();
    }
}
