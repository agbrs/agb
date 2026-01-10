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
use alloc::{vec, vec::Vec};

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
    Air = 2,
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

    let map = Map::new(4, 4);

    for y in 0..32 {
        for x in 0..16 {
            let (position, me, them) = map.get_from_gba_tile(x * 2, y);

            for (i, tile) in tile_cache.get_tiles(position, me, them).iter().enumerate() {
                bg.set_tile_dynamic16((x * 2 + i as i32, y), tile, TileEffect::default());
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
        let me = tiles::ISOMETRIC
            .tiles
            .get_tile_data(i + tile_a as u16 * 4 + position.offset());
        let them = tiles::ISOMETRIC
            .tiles
            .get_tile_data(i + tile_b as u16 * 4 + position.reverse().offset());

        let (first, second) = if tile_a <= tile_b {
            (me, them)
        } else {
            (them, me)
        };

        tile.data().copy_from_slice(first);
        blit_4(tile.data(), second);
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

struct Map {
    map_data: Vec<TileType>,
    width: usize,
    height: usize,
}

const TILE_WIDTH: i32 = 4;
const TILE_HEIGHT: i32 = 2;

impl Map {
    fn new(width: usize, height: usize) -> Self {
        let mut map_data = vec![TileType::Dirt; width * height];
        map_data[8] = TileType::Water;
        map_data[9] = TileType::Water;
        map_data[10] = TileType::Water;
        map_data[5] = TileType::Air;

        Self {
            map_data,
            width,
            height,
        }
    }

    fn get_tile(&self, x: i32, y: i32) -> TileType {
        if x < 0 || x as usize >= self.width || y < 0 || y as usize >= self.height {
            return TileType::Air;
        }

        self.map_data[x as usize + y as usize * self.width]
    }

    fn get_from_gba_tile(&self, x: i32, y: i32) -> (TilePosition, TileType, TileType) {
        let x = x - 16;
        let y = y - 4;

        let tile_position = match ((div_floor(x, 2)).rem_euclid(2), y.rem_euclid(2)) {
            (0, 0) => TilePosition::TopLeft,
            (1, 0) => TilePosition::TopRight,
            (0, 1) => TilePosition::BottomLeft,
            (1, 1) => TilePosition::BottomRight,
            _ => unreachable!(),
        };

        let macro_tile_x = div_floor(x, 4);
        let macro_tile_y = div_floor(y, 2);

        let (tile_x, tile_y) = (macro_tile_x + macro_tile_y, macro_tile_y - macro_tile_x);

        let neighbour_pos = match tile_position {
            TilePosition::TopLeft => (-1, 0),
            TilePosition::TopRight => (0, -1),
            TilePosition::BottomLeft => (0, 1),
            TilePosition::BottomRight => (1, 0),
        };

        let me = self.get_tile(tile_x, tile_y);
        let neighbour = self.get_tile(tile_x + neighbour_pos.0, tile_y + neighbour_pos.1);

        (tile_position, me, neighbour)
    }
}

fn div_floor(numerator: i32, divisor: i32) -> i32 {
    let d = numerator / divisor;
    let r = numerator % divisor;
    let correction = (numerator ^ divisor) >> (i32::BITS - 1);
    if r != 0 { d + correction } else { d }
}
