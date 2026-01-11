use agb::{
    display::{tiled::DynamicTile16, utils::blit_16_colour},
    fixnum::{Num, Vector2D, vec2},
    hash_map::{HashMap, HashSet},
};
use alloc::{rc::Rc, vec::Vec};
use core::{array, hash::Hash, ops::Deref};

use crate::tiles;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Quadrant {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TileType {
    Dirt = 0,
    Water = 1,
    Air = 2,
}

#[derive(Default)]
pub struct TileCache {
    cache: HashMap<TileSpec, [TileHolder; 2]>,
    tiles: HashSet<TileHolder>,
}

#[derive(Clone)]
pub struct TileHolder(Rc<DynamicTile16>);

impl Hash for TileHolder {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.data().hash(state);
    }
}

impl PartialEq for TileHolder {
    fn eq(&self, other: &Self) -> bool {
        self.0.data() == other.0.data()
    }
}

impl Eq for TileHolder {}

impl Deref for TileHolder {
    type Target = DynamicTile16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct TileSpec {
    quadrant: Quadrant,
    me: TileType,
    them: TileType,
    neighbours: NeighbourTileContext,
}

// All these refer orthographically to the current tile
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct NeighbourTileContext {
    left: TileType,
    up_left: TileType,
    up: TileType,
    up_right: TileType,
}

impl TileCache {
    // expects gba_tile_pos.x to be even
    pub fn get_tiles(&mut self, map: &Map, gba_tile_pos: Vector2D<i32>) -> &[TileHolder; 2] {
        let tile_spec = map.get_from_gba_tile(gba_tile_pos.x, gba_tile_pos.y);

        self.cache.entry(tile_spec).or_insert_with(|| {
            let genned_tiles = build_combined_tile(tile_spec);

            genned_tiles.map(|genned_tile| {
                let tile_holder = TileHolder(Rc::new(genned_tile));
                if let Some(existing_tile) = self.tiles.get(&tile_holder) {
                    existing_tile.clone()
                } else {
                    self.tiles.insert(tile_holder.clone());
                    tile_holder
                }
            })
        })
    }

    pub fn cache_size(&self) -> usize {
        self.tiles.len()
    }
}

fn build_combined_tile(tile_spec: TileSpec) -> [DynamicTile16; 2] {
    let TileSpec {
        quadrant,
        me,
        them,
        neighbours,
    } = tile_spec;

    // Wall offset in the tileset (walls are 2 rows below floors)
    const WALL_OFFSET: u16 = (tiles::ISOMETRIC.width * 2) as u16;

    fn get_tile_id(offset: u16, tile_type: TileType, is_wall: bool) -> u16 {
        offset + tile_type as u16 * 5 + if is_wall { WALL_OFFSET } else { 0 }
    }

    // - Upper wall: from a tile above this quadrant on screen (extends down into this quadrant)
    // - Local wall: from either `me` or `them` (the two tiles this quadrant sits between)
    let (upper_wall, local_wall, gap_fill) = match quadrant {
        Quadrant::TopLeft => {
            // Upper wall: bottom-left wall of up tile
            // Local wall: top-right wall of left ghost tile
            let upper = get_tile_id(Quadrant::BottomLeft.offset(), neighbours.up, true);
            let local = get_tile_id(Quadrant::TopRight.offset(), them, true);
            (upper, local, None)
        }
        Quadrant::TopRight => {
            // Upper wall: bottom-right wall of up tile
            // Local wall: top-left wall of up ghost tile
            // Gap fill: 1px right edge from up_left tile
            let upper = get_tile_id(Quadrant::BottomRight.offset(), neighbours.up, true);
            let local = get_tile_id(Quadrant::TopLeft.offset(), them, true);
            let gap = get_tile_id(Quadrant::TopRight.offset() + 2, neighbours.up_left, true);
            (upper, local, Some(gap))
        }
        Quadrant::BottomLeft => {
            // Upper wall: bottom-right wall of up_left tile
            // Local wall: top-left wall of central tile
            // Gap fill: 1px right edge from left tile
            let upper = get_tile_id(Quadrant::BottomRight.offset(), neighbours.up_left, true);
            let local = get_tile_id(Quadrant::TopLeft.offset(), me, true);
            let gap = get_tile_id(Quadrant::TopRight.offset() + 2, neighbours.left, true);
            (upper, local, Some(gap))
        }
        Quadrant::BottomRight => {
            // Upper wall: bottom-left wall of up_right tile
            // Local wall: top-right wall of central tile
            let upper = get_tile_id(Quadrant::BottomLeft.offset(), neighbours.up_right, true);
            let local = get_tile_id(Quadrant::TopRight.offset(), me, true);
            (upper, local, None)
        }
    };

    array::from_fn(|i| {
        let i = i as u16;

        fn get_tile(offset: u16) -> &'static [u32] {
            tiles::ISOMETRIC.tiles.get_tile_data(offset)
        }

        let mut tile = DynamicTile16::new().fill_with(0);

        let me_tile = get_tile(get_tile_id(quadrant.offset(), me, false) + i);
        let them_tile = get_tile(get_tile_id(quadrant.reverse().offset(), them, false) + i);

        // Order floors so higher-priority tile types render on top
        let (first, second) = if me > them {
            (me_tile, them_tile)
        } else {
            (them_tile, me_tile)
        };

        if let Some(gap_fill) = gap_fill
            && i == 0
        {
            blit_16_colour(tile.data_mut(), get_tile(gap_fill));
        }

        blit_16_colour(tile.data_mut(), get_tile(upper_wall + i));
        blit_16_colour(tile.data_mut(), get_tile(local_wall + i));

        blit_16_colour(tile.data_mut(), first);
        blit_16_colour(tile.data_mut(), second);

        tile
    })
}

impl Quadrant {
    fn offset(self) -> u16 {
        match self {
            Quadrant::TopLeft => 0,
            Quadrant::TopRight => 2,
            Quadrant::BottomLeft => tiles::ISOMETRIC.width as u16,
            Quadrant::BottomRight => tiles::ISOMETRIC.width as u16 + 2,
        }
    }

    fn reverse(self) -> Self {
        match self {
            Quadrant::TopLeft => Quadrant::BottomRight,
            Quadrant::TopRight => Quadrant::BottomLeft,
            Quadrant::BottomLeft => Quadrant::TopRight,
            Quadrant::BottomRight => Quadrant::TopLeft,
        }
    }

    /// Returns the position in the direction of this quadrant
    fn neighbour(self, pos: Vector2D<i32>) -> Vector2D<i32> {
        let neighbour_quadrant = match self {
            Quadrant::TopLeft => (-1, 0),
            Quadrant::TopRight => (0, -1),
            Quadrant::BottomLeft => (0, 1),
            Quadrant::BottomRight => (1, 0),
        };

        pos + vec2(neighbour_quadrant.0, neighbour_quadrant.1)
    }

    fn from_gba_tile(tile: Vector2D<i32>) -> Self {
        match (
            (div_floor(tile.x, TILE_WIDTH / 2)).rem_euclid(TILE_WIDTH / 2),
            tile.y.rem_euclid(TILE_HEIGHT),
        ) {
            (0, 0) => Quadrant::TopLeft,
            (1, 0) => Quadrant::TopRight,
            (0, 1) => Quadrant::BottomLeft,
            (1, 1) => Quadrant::BottomRight,
            _ => unreachable!(),
        }
    }
}

pub struct Map {
    map_data: Vec<TileType>,
    width: usize,
    height: usize,
}

const TILE_WIDTH: i32 = 4;
const TILE_HEIGHT: i32 = 2;

impl Map {
    pub fn new(width: usize, height: usize, map_data: Vec<TileType>) -> Self {
        assert_eq!(map_data.len(), width * height);

        Self {
            map_data,
            width,
            height,
        }
    }

    pub fn get_tile(&self, pos: Vector2D<i32>) -> TileType {
        let Vector2D { x, y } = pos;
        if x < 0 || x as usize >= self.width || y < 0 || y as usize >= self.height {
            return TileType::Air;
        }

        self.map_data[x as usize + y as usize * self.width]
    }

    fn get_from_gba_tile(&self, x: i32, y: i32) -> TileSpec {
        let quadrant = Quadrant::from_gba_tile(vec2(x, y));

        let macro_tile_x = div_floor(x, TILE_WIDTH);
        let macro_tile_y = div_floor(y, TILE_HEIGHT);

        let tile = vec2(macro_tile_x + macro_tile_y, macro_tile_y - macro_tile_x);

        let me = self.get_tile(tile);
        let neighbour = self.get_tile(quadrant.neighbour(tile));

        TileSpec {
            quadrant,
            me,
            them: neighbour,
            neighbours: NeighbourTileContext {
                left: self.get_tile(tile + vec2(-1, 1)),
                up_left: self.get_tile(tile + vec2(-1, 0)),
                up: self.get_tile(tile + vec2(-1, -1)),
                up_right: self.get_tile(tile + vec2(0, -1)),
            },
        }
    }
}

/// same as the div_floor in nightly but isn't stable yet so here until that's stabilised
fn div_floor(numerator: i32, divisor: i32) -> i32 {
    let d = numerator / divisor;
    let r = numerator % divisor;
    let correction = (numerator ^ divisor) >> (i32::BITS - 1);
    if r != 0 { d + correction } else { d }
}

pub fn world_to_gba_tile_smooth(world: Vector2D<Num<i32, 12>>) -> Vector2D<Num<i32, 12>> {
    let macro_pos = world_to_macro_smooth(world);
    vec2(macro_pos.x * TILE_WIDTH, macro_pos.y * TILE_HEIGHT)
}

fn world_to_macro_smooth(world: Vector2D<Num<i32, 12>>) -> Vector2D<Num<i32, 12>> {
    vec2(world.x - world.y + 1, world.x + world.y + 1) / 2
}
