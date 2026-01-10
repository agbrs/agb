#![no_main]
#![no_std]

use agb::{
    Gba,
    display::{
        Priority,
        tiled::{
            DynamicTile16, RegularBackground, RegularBackgroundSize, TileEffect, TileFormat,
            VRAM_MANAGER,
        },
        utils::blit_4,
    },
    hash_map::HashMap,
    include_background_gfx,
};

extern crate alloc;

include_background_gfx!(mod tiles, "000000",
    ISOMETRIC => "examples/gfx/isometric_tiles.aseprite"
);

#[agb::entry]
fn entry(gba: Gba) -> ! {
    main(gba);
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TilePosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum TileType {
    Dirt = 0,
    Water = 1,
}

fn main(mut gba: Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut gfx = gba.graphics.get();

    let mut tile_cache = TileCache::default();

    for y in 0..16 {
        for x in 0..8 {
            let mut index = 0;
            for pos in TilePosition::iter() {
                for tile in tile_cache.get_tiles(pos, TileType::Dirt, TileType::Water) {
                    bg.set_tile_dynamic16(
                        (x * 4 + (index % 4) as u16, y * 2 + (index / 4) as u16),
                        tile,
                        TileEffect::default(),
                    );

                    index += 1;
                }
            }
        }
    }

    loop {
        let mut frame = gfx.frame();

        bg.show(&mut frame);

        frame.commit();
    }
}

#[derive(Default)]
struct TileCache {
    // Direction, Centre, Outer
    cache: HashMap<(TilePosition, TileType, TileType), [DynamicTile16; 2]>,
}

impl TileCache {
    fn get_tiles(
        &mut self,
        position: TilePosition,
        tile_a: TileType,
        tile_b: TileType,
    ) -> &[DynamicTile16; 2] {
        self.cache
            .entry((position, tile_a, tile_b))
            .or_insert_with(|| build_combined_tile(position, tile_a, tile_b))
    }
}

fn build_combined_tile(
    position: TilePosition,
    tile_a: TileType,
    tile_b: TileType,
) -> [DynamicTile16; 2] {
    let mut result = [DynamicTile16::new(), DynamicTile16::new()];

    for (i, tile) in result.iter_mut().enumerate() {
        let i = i as u16;
        tile.data().copy_from_slice(
            tiles::ISOMETRIC
                .tiles
                .get_tile_data(i + tile_a as u16 * 4 + position.offset()),
        );
        blit_4(
            tile.data(),
            tiles::ISOMETRIC
                .tiles
                .get_tile_data(i + tile_b as u16 * 4 + position.reverse().offset()),
        );
    }

    result
}

impl TilePosition {
    fn offset(self) -> u16 {
        match self {
            TilePosition::TopLeft => 0,
            TilePosition::TopRight => 2,
            TilePosition::BottomLeft => tiles::ISOMETRIC.width as u16,
            TilePosition::BottomRight => tiles::ISOMETRIC.width as u16 + 2,
        }
    }

    fn reverse(self) -> Self {
        match self {
            TilePosition::TopLeft => TilePosition::BottomRight,
            TilePosition::TopRight => TilePosition::BottomLeft,
            TilePosition::BottomLeft => TilePosition::TopRight,
            TilePosition::BottomRight => TilePosition::TopLeft,
        }
    }

    fn iter() -> impl Iterator<Item = Self> {
        [
            TilePosition::TopLeft,
            TilePosition::TopRight,
            TilePosition::BottomLeft,
            TilePosition::BottomRight,
        ]
        .into_iter()
    }
}
