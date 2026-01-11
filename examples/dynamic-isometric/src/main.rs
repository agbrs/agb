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
        GraphicsFrame, Priority, Rgb15,
        object::{GraphicsMode, Object, Tag},
        tiled::{RegularBackground, RegularBackgroundSize, TileEffect, TileFormat, VRAM_MANAGER},
    },
    dma::HBlankDma,
    fixnum::{Num, Vector2D, num, vec2},
    include_aseprite, include_background_gfx, include_colours,
    input::ButtonController,
};

use alloc::vec;

use isometric_render::TileType;

use crate::isometric_render::{Map, TileCache, world_to_gba_tile_smooth};

extern crate alloc;

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
            let cache_key = floor_map.get_from_gba_tile(x * 2, y);

            for (i, tile) in tile_cache.get_tiles(cache_key).iter().enumerate() {
                floor_bg.set_tile_dynamic16((x * 2 + i as i32, y), tile, TileEffect::default());
            }

            let cache_key = wall_map.get_from_gba_tile(x * 2, y);

            for (i, tile) in tile_cache.get_tiles(cache_key).iter().enumerate() {
                wall_bg.set_tile_dynamic16((x * 2 + i as i32, y), tile, TileEffect::default());
            }
        }
    }

    let initial_position = vec2(num!(6), num!(3));
    let mut character = Character::new(&sprites::KAIJU, initial_position);

    let mut input = ButtonController::new();

    agb::println!("Cache size: {}", tile_cache.cache_size());

    let mut character_target_position = initial_position;

    loop {
        input.update();
        let mut frame = gfx.frame();

        let floor_id = floor_bg.show(&mut frame);
        wall_bg.show(&mut frame);

        HBlankDma::new(
            VRAM_MANAGER.background_palette_colour_dma(0, 0),
            &SKY_GRADIENT,
        )
        .show(&mut frame);

        let just_pressed = input.just_pressed_vector::<Num<i32, 12>>();
        if just_pressed != vec2(num!(0), num!(0)) {
            if character_target_position != character.position {
                character.position = character_target_position;
            }

            character.flipped = just_pressed.x > num!(0) || just_pressed.y < num!(0);

            let new_location = character_target_position + just_pressed;
            if wall_map.get_tile(new_location.floor()) == TileType::Air
                && floor_map.get_tile(new_location.floor()) != TileType::Air
            {
                character_target_position = new_location;
            }
        }

        character.position = (character.position + character_target_position) / 2;

        character.show(&mut frame, &wall_map);

        frame
            .blend()
            .object_transparency(num!(0.5), num!(0.5))
            .enable_background(floor_id);

        frame.commit();
    }
}

struct Character {
    tag: &'static Tag,
    // position is the current foot location in world space
    position: Vector2D<Num<i32, 12>>,
    foot_offset: Vector2D<i32>,
    flipped: bool,
}

impl Character {
    fn new(tag: &'static Tag, position: Vector2D<Num<i32, 12>>) -> Self {
        Self {
            tag,
            position,
            foot_offset: vec2(16, 30),
            flipped: false,
        }
    }

    fn show(&self, frame: &mut GraphicsFrame, wall_map: &Map) {
        // which priority do we need for the bottom sprites?
        let tile_pos = self.position.round();
        let priority = if wall_map.get_tile(tile_pos + vec2(1, 0)) != TileType::Air
            || wall_map.get_tile(tile_pos + vec2(1, 1)) != TileType::Air
            || wall_map.get_tile(tile_pos + vec2(0, 1)) != TileType::Air
        {
            Priority::P3
        } else {
            Priority::P1
        };

        let real_tile_space = world_to_gba_tile_smooth(self.position);
        let real_pixel_space = (real_tile_space * 8).round();

        Object::new(self.tag.sprite(0))
            .set_pos(real_pixel_space - self.foot_offset)
            .set_priority(Priority::P1)
            .set_hflip(self.flipped)
            .show(frame);
        Object::new(self.tag.sprite(1))
            .set_pos(real_pixel_space - self.foot_offset + vec2(0, 16))
            .set_priority(priority)
            .set_hflip(self.flipped)
            .show(frame);

        // drop shadow
        Object::new(sprites::DROP_SHADOW.sprite(0))
            .set_pos(real_pixel_space - vec2(16, 8))
            .set_priority(priority)
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(frame);
    }
}
