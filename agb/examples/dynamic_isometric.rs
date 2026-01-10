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

fn main(mut gba: Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut gfx = gba.graphics.get();

    let mut tile_cache: HashMap<TilePosition, [DynamicTile16; 2]> = HashMap::new();
    for position in TilePosition::iter() {
        tile_cache.insert(position, build_combined_tile(position, 0, 0));
    }

    for y in 0..16 {
        for x in 0..8 {
            for (index, tile) in TilePosition::iter()
                .flat_map(|pos| tile_cache.get(&pos).unwrap())
                .enumerate()
            {
                bg.set_tile_dynamic16(
                    (x * 4 + (index % 4) as u16, y * 2 + (index / 4) as u16),
                    tile,
                    TileEffect::default(),
                );
            }
        }
    }

    loop {
        let mut frame = gfx.frame();

        bg.show(&mut frame);

        frame.commit();
    }
}

fn build_combined_tile(position: TilePosition, tile_a: u16, tile_b: u16) -> [DynamicTile16; 2] {
    let mut result = [DynamicTile16::new(), DynamicTile16::new()];

    for (i, tile) in result.iter_mut().enumerate() {
        let i = i as u16;
        tile.data().copy_from_slice(
            tiles::ISOMETRIC
                .tiles
                .get_tile_data(i + tile_a * 4 + position.offset()),
        );
        blit_4(
            tile.data(),
            tiles::ISOMETRIC
                .tiles
                .get_tile_data(i + tile_b * 4 + position.reverse().offset()),
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
