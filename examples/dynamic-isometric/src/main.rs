#![no_main]
#![no_std]

use agb::{
    Gba,
    display::{
        GraphicsFrame, Priority, Rgb15,
        object::{GraphicsMode, Object, Tag},
        tiled::{
            DynamicTile16, RegularBackground, RegularBackgroundSize, TileEffect, TileFormat,
            VRAM_MANAGER,
        },
        utils::blit_16_colour,
    },
    dma::HBlankDma,
    fixnum::{Num, Vector2D, num, vec2},
    hash_map::{HashMap, HashSet},
    include_aseprite, include_background_gfx, include_colours,
    input::ButtonController,
};

use alloc::{rc::Rc, vec, vec::Vec};

use core::hash::Hash;

extern crate alloc;

include_background_gfx!(mod tiles, "333333",
    ISOMETRIC => "gfx/isometric_tiles.aseprite"
);

include_aseprite!(mod sprites, "gfx/kaiju.aseprite");

static SKY_GRADIENT: [Rgb15; 160] = include_colours!("gfx/sky-background-gradient.aseprite");

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
                floor_bg.set_tile_dynamic16((x * 2 + i as i32, y), &tile.0, TileEffect::default());
            }

            let cache_key = wall_map.get_from_gba_tile(x * 2, y);

            for (i, tile) in tile_cache.get_tiles(cache_key).iter().enumerate() {
                wall_bg.set_tile_dynamic16((x * 2 + i as i32, y), &tile.0, TileEffect::default());
            }
        }
    }

    let initial_position = vec2(num!(6), num!(3));
    let mut character = Character::new(&sprites::KAIJU, initial_position);

    let mut input = ButtonController::new();

    agb::println!("Cache size: {}", tile_cache.tiles.len());

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

#[derive(Default)]
struct TileCache {
    cache: HashMap<CacheKey, [TileHolder; 2]>,
    tiles: HashSet<TileHolder>,
}

#[derive(Clone)]
struct TileHolder(Rc<DynamicTile16>);

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

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct CacheKey {
    direction: TilePosition,
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
    fn get_tiles(&mut self, cache_key: CacheKey) -> &[TileHolder; 2] {
        self.cache.entry(cache_key).or_insert_with(|| {
            let genned_tiles = build_combined_tile(cache_key);

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
}

fn build_combined_tile(cache_key: CacheKey) -> [DynamicTile16; 2] {
    let mut result = [
        DynamicTile16::new().fill_with(0),
        DynamicTile16::new().fill_with(0),
    ];

    let CacheKey {
        direction: position,
        me: tile_a,
        them: tile_b,
        neighbours,
    } = cache_key;

    for (i, tile) in result.iter_mut().enumerate() {
        let i = i as u16;

        fn get_tile(offset: u16, tile_type: TileType) -> &'static [u32] {
            tiles::ISOMETRIC
                .tiles
                .get_tile_data(offset + tile_type as u16 * 5)
        }

        let me = get_tile(i + position.offset(), tile_a);
        let them = get_tile(i + position.reverse().offset(), tile_b);

        const WALL_OFFSET: u16 = (tiles::ISOMETRIC.width * 2) as u16;

        let (first_wall, second_wall, gap_fill) = match position {
            TilePosition::TopLeft => {
                // upper bottom left wall, their top right wall, their floor, my floor
                let ublw = get_tile(
                    TilePosition::BottomLeft.offset() + i + WALL_OFFSET,
                    neighbours.up,
                );
                let ttrw = get_tile(TilePosition::TopRight.offset() + i + WALL_OFFSET, tile_b);

                (ublw, ttrw, None)
            }
            TilePosition::TopRight => {
                // upper bottom right wall, their top left wall, their floor, my floor
                let ubrw = get_tile(
                    TilePosition::BottomRight.offset() + i + WALL_OFFSET,
                    neighbours.up,
                );
                let ttlw = get_tile(TilePosition::TopLeft.offset() + i + WALL_OFFSET, tile_b);

                // the RHS of the wall to fill the 1px gap
                if i == 0 {
                    let wall_rhs = get_tile(
                        TilePosition::TopRight.offset() + 2 + WALL_OFFSET,
                        neighbours.up_left,
                    );
                    (ubrw, ttlw, Some(wall_rhs))
                } else {
                    (ubrw, ttlw, None)
                }
            }
            TilePosition::BottomLeft => {
                // (upper.0) bottom right wall, my top left wall, their floor, my floor
                let ubrw = get_tile(
                    TilePosition::BottomRight.offset() + i + WALL_OFFSET,
                    neighbours.up_left,
                );
                let mtlw = get_tile(TilePosition::TopLeft.offset() + i + WALL_OFFSET, tile_a);

                // the RHS of the wall to fill the 1px gap
                if i == 0 {
                    let wall_rhs = get_tile(
                        TilePosition::TopRight.offset() + 2 + WALL_OFFSET,
                        neighbours.left,
                    );
                    (ubrw, mtlw, Some(wall_rhs))
                } else {
                    (ubrw, mtlw, None)
                }
            }
            TilePosition::BottomRight => {
                // (upper.2) bottom left wall, my top right wall, their floor, my floor
                let ubrw = get_tile(
                    TilePosition::BottomLeft.offset() + i + WALL_OFFSET,
                    neighbours.up_right,
                );
                let mtlw = get_tile(TilePosition::TopRight.offset() + i + WALL_OFFSET, tile_a);

                (ubrw, mtlw, None)
            }
        };

        if let Some(gap_fill) = gap_fill {
            blit_16_colour(tile.data_mut(), gap_fill);
        }

        blit_16_colour(tile.data_mut(), first_wall);
        blit_16_colour(tile.data_mut(), second_wall);

        let (first, second) = if tile_a > tile_b {
            (me, them)
        } else {
            (them, me)
        };

        blit_16_colour(tile.data_mut(), first);
        blit_16_colour(tile.data_mut(), second);
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
}

struct Map {
    map_data: Vec<TileType>,
    width: usize,
    height: usize,
}

const TILE_WIDTH: i32 = 4;
const TILE_HEIGHT: i32 = 2;

impl Map {
    fn new(width: usize, height: usize, map_data: Vec<TileType>) -> Self {
        assert_eq!(map_data.len(), width * height);

        Self {
            map_data,
            width,
            height,
        }
    }

    fn get_tile(&self, pos: Vector2D<i32>) -> TileType {
        let Vector2D { x, y } = pos;
        if x < 0 || x as usize >= self.width || y < 0 || y as usize >= self.height {
            return TileType::Air;
        }

        self.map_data[x as usize + y as usize * self.width]
    }

    fn get_from_gba_tile(&self, x: i32, y: i32) -> CacheKey {
        let tile_position = match (
            (div_floor(x, TILE_WIDTH / 2)).rem_euclid(TILE_WIDTH / 2),
            y.rem_euclid(TILE_HEIGHT),
        ) {
            (0, 0) => TilePosition::TopLeft,
            (1, 0) => TilePosition::TopRight,
            (0, 1) => TilePosition::BottomLeft,
            (1, 1) => TilePosition::BottomRight,
            _ => unreachable!(),
        };

        let macro_tile_x = div_floor(x, TILE_WIDTH);
        let macro_tile_y = div_floor(y, TILE_HEIGHT);

        let (tile_x, tile_y) = (macro_tile_x + macro_tile_y, macro_tile_y - macro_tile_x);

        let neighbour_pos = match tile_position {
            TilePosition::TopLeft => (-1, 0),
            TilePosition::TopRight => (0, -1),
            TilePosition::BottomLeft => (0, 1),
            TilePosition::BottomRight => (1, 0),
        };

        let me = self.get_tile(vec2(tile_x, tile_y));
        let neighbour = self.get_tile(vec2(tile_x + neighbour_pos.0, tile_y + neighbour_pos.1));

        CacheKey {
            direction: tile_position,
            me,
            them: neighbour,
            neighbours: NeighbourTileContext {
                left: self.get_tile(vec2(tile_x - 1, tile_y + 1)),
                up_left: self.get_tile(vec2(tile_x - 1, tile_y)),
                up: self.get_tile(vec2(tile_x - 1, tile_y - 1)),
                up_right: self.get_tile(vec2(tile_x, tile_y - 1)),
            },
        }
    }
}

fn div_floor(numerator: i32, divisor: i32) -> i32 {
    let d = numerator / divisor;
    let r = numerator % divisor;
    let correction = (numerator ^ divisor) >> (i32::BITS - 1);
    if r != 0 { d + correction } else { d }
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

        let macro_space = vec2(
            self.position.x - self.position.y + 1,
            self.position.x + self.position.y + 1,
        ) / 2;
        let real_tile_space = vec2(macro_space.x * TILE_WIDTH, macro_space.y * TILE_HEIGHT);
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
