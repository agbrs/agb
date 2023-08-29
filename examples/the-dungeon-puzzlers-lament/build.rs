use quote::{quote, TokenStreamExt};
use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    str::FromStr,
};

use proc_macro2::TokenStream;

const LEVEL_NAMES: &[&str] = &[
    "level1",
    "level2",
    "level3",
    "level4",
    "level5",
    "level6",
    "level_switch",
    "level_spikes",
    "level_spikes2",
    "level_squid_force_button",
    "level_squid_intro",
    "level_squid2",
    "level_squid1",
    "level_squid_item",
    "level_squid_button",
    "level_squid_drop",
    "level_spikes3",
    "level_around",
    "level_squidprogramming",
    "a_familiar_sight",
    "block_push_1",
    "just_rocks",
    "squid_rock",
    "ice_ice",
    "block_push_2",
    "glove_key",
    "block_push_3",
    "teleporter_1",
    "squid_teleport",
    "teleporter_2",
    "slime_teleporter",
    "another_ice",
    "another_ice_2",
];

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable must be specified");

    let mut tile_loader = tiled::Loader::new();

    let ui_map = load_tmx(&mut tile_loader, "maps/UI.tmx");
    let ui_tiles = export_ui_tiles(&ui_map, quote!(ui));

    const DPL_LEVELS_ENVIRONMENT_VARIABLE: &str = "DPL_LEVELS";

    println!(
        "cargo:rerun-if-env-changed={}",
        DPL_LEVELS_ENVIRONMENT_VARIABLE
    );

    let levels: Vec<String> = env::var(DPL_LEVELS_ENVIRONMENT_VARIABLE)
        .map(|x| x.split(',').map(|x| x.trim().to_string()).collect())
        .unwrap_or(LEVEL_NAMES.iter().map(|x| x.to_string()).collect());

    let levels = levels
        .iter()
        .map(|level| load_level(&mut tile_loader, &format!("maps/levels/{level}.tmx")))
        .collect::<Vec<_>>();
    let levels_tiles = levels.iter().map(|level| &level.0);
    let levels_data = levels.iter().map(|level| &level.1);

    let tilemaps_output = quote! {
        use agb::display::tiled::TileSetting;

        pub const UI_BACKGROUND_MAP: &[TileSetting] = #ui_tiles;
        pub const LEVELS_MAP: &[&[TileSetting]] = &[#(#levels_tiles),*];
    };

    let levels_output = quote! {
        pub const LEVELS: &[Level] = &[#(#levels_data),*];
    };

    {
        let tilemaps_output_file = File::create(format!("{out_dir}/tilemaps.rs"))
            .expect("Failed to open tilemaps.rs for writing");
        let mut tilemaps_writer = BufWriter::new(tilemaps_output_file);
        write!(&mut tilemaps_writer, "{tilemaps_output}").unwrap();
    }

    {
        let levels_output_file = File::create(format!("{out_dir}/levels.rs"))
            .expect("Failed to open levels.rs for writing");
        let mut levels_output_writer = BufWriter::new(levels_output_file);

        write!(&mut levels_output_writer, "{levels_output}").unwrap();
    }
}

fn load_level(loader: &mut tiled::Loader, filename: &str) -> (TokenStream, Level) {
    let level_map = load_tmx(loader, filename);
    let tiles = export_tiles(&level_map, quote!(level));
    let data = export_level(&level_map);

    (tiles, data)
}

fn load_tmx(loader: &mut tiled::Loader, filename: &str) -> tiled::Map {
    println!("cargo:rerun-if-changed={filename}");
    loader.load_tmx_map(filename).expect("failed to load map")
}

enum Entity {
    Sword,
    Slime,
    Hero,
    Stairs,
    Door,
    Key,
    Switch,
    SwitchPressed,
    SwitchedOpenDoor,
    SwitchedClosedDoor,
    SpikesUp,
    SpikesDown,
    SquidUp,
    SquidDown,
    Ice,
    MovableBlock,
    Glove,
    Teleporter,
}

impl FromStr for Entity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Entity::*;

        Ok(match s {
            "SWORD" => Sword,
            "SLIME" => Slime,
            "HERO" => Hero,
            "STAIRS" => Stairs,
            "DOOR" => Door,
            "KEY" => Key,
            "SWITCH" => Switch,
            "SWITCH_PRESSED" => SwitchPressed,
            "DOOR_SWITCHED" => SwitchedClosedDoor,
            "DOOR_SWITCHED_OPEN" => SwitchedOpenDoor,
            "SPIKES" => SpikesUp,
            "SPIKES_DOWN" => SpikesDown,
            "SQUID_UP" => SquidUp,
            "SQUID_DOWN" => SquidDown,
            "ICE" => Ice,
            "BLOCK" => MovableBlock,
            "GLOVE" => Glove,
            "TELEPORTER" => Teleporter,
            _ => return Err(()),
        })
    }
}

impl quote::ToTokens for Entity {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Entity::*;

        tokens.append_all(match self {
            Sword => quote!(Item::Sword),
            Slime => quote!(Item::Slime),
            Hero => quote!(Item::Hero),
            Stairs => quote!(Item::Stairs),
            Door => quote!(Item::Door),
            Key => quote!(Item::Key),
            Switch => quote!(Item::Switch),
            SwitchPressed => quote!(Item::SwitchPressed),
            SwitchedOpenDoor => quote!(Item::SwitchedOpenDoor),
            SwitchedClosedDoor => quote!(Item::SwitchedClosedDoor),
            SpikesUp => quote!(Item::SpikesUp),
            SpikesDown => quote!(Item::SpikesDown),
            SquidUp => quote!(Item::SquidUp),
            SquidDown => quote!(Item::SquidDown),
            Ice => quote!(Item::Ice),
            MovableBlock => quote!(Item::MovableBlock),
            Glove => quote!(Item::Glove),
            Teleporter => quote!(Item::Teleporter),
        })
    }
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(c: char) -> Result<Self, Self::Error> {
        use Direction::*;

        Ok(match c {
            'U' => Up,
            'D' => Down,
            'L' => Left,
            'R' => Right,
            _ => return Err(()),
        })
    }
}

impl quote::ToTokens for Direction {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use Direction::*;

        tokens.append_all(match self {
            Up => quote!(Direction::Up),
            Down => quote!(Direction::Down),
            Left => quote!(Direction::Left),
            Right => quote!(Direction::Right),
        });
    }
}

struct EntityWithPosition(Entity, (i32, i32));

impl quote::ToTokens for EntityWithPosition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let pos_x = self.1 .0;
        let pos_y = self.1 .1;
        let location = quote!(Vector2D::new(#pos_x, #pos_y));
        let item = &self.0;

        tokens.append_all(quote!(Entity(#item, #location)))
    }
}

struct Level {
    starting_items: Vec<Entity>,
    fixed_positions: Vec<EntityWithPosition>,
    directions: Vec<Direction>,
    wall_bitmap: Vec<u8>,
    name: String,
}

impl quote::ToTokens for Level {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let wall_bitmap = &self.wall_bitmap;
        let fixed_positions = &self.fixed_positions;
        let directions = &self.directions;
        let starting_items = &self.starting_items;
        let name = &self.name;

        tokens.append_all(quote! {
            Level::new(
                Map::new(11, 10, &[#(#wall_bitmap),*]),
                &[#(#fixed_positions),*],
                &[#(#directions),*],
                &[#(#starting_items),*],
                #name,
            )
        })
    }
}

fn export_level(map: &tiled::Map) -> Level {
    let objects = map.get_layer(1).unwrap().as_object_layer().unwrap();

    let fixed_positions = objects.objects().map(|obj| {
        let entity: Entity = obj
            .name
            .parse()
            .unwrap_or_else(|_| panic!("unknown object type {}", obj.name));

        let x = (obj.x / 16.0) as i32;
        let y = (obj.y / 16.0) as i32;

        EntityWithPosition(entity, (x, y))
    });

    let Some(tiled::PropertyValue::StringValue(starting_items)) = map.properties.get("ITEMS")
    else {
        panic!("Starting items must be a string")
    };

    let Some(tiled::PropertyValue::StringValue(level_name)) = map.properties.get("NAME") else {
        panic!("Level name must be a string")
    };

    let starting_items = starting_items.split(',').map(|starting_item| {
        starting_item
            .parse()
            .unwrap_or_else(|_| panic!("unknown object type {}", starting_item))
    });

    let Some(tiled::PropertyValue::StringValue(directions)) = map.properties.get("DIRECTIONS")
    else {
        panic!("Starting items must be a string")
    };

    let directions = directions.chars().map(|starting_item| {
        starting_item
            .try_into()
            .unwrap_or_else(|_| panic!("unknown object type {}", starting_item))
    });

    let Some(tiled::TileLayer::Finite(tiles)) = map.get_layer(0).unwrap().as_tile_layer() else {
        panic!("Not a finite layer")
    };

    let are_walls = (0..10 * 11).map(|id| {
        let tile_x = id % 11;
        let tile_y = id / 11;

        let is_wall = tiles
            .get_tile(tile_x, tile_y)
            .map(|tile| {
                let tileset = tile.get_tileset();
                let tile_data = &tileset.get_tile(tile.id()).unwrap();
                tile_data
                    .user_type
                    .as_ref()
                    .map(|user_type| user_type == "WALL")
                    .unwrap_or(false)
            })
            .unwrap_or(true);

        is_wall
    });

    Level {
        starting_items: starting_items.collect(),
        fixed_positions: fixed_positions.collect(),
        directions: directions.collect(),
        wall_bitmap: bool_to_bit(&are_walls.collect::<Vec<_>>()),
        name: level_name.clone(),
    }
}

fn export_tiles(map: &tiled::Map, background: TokenStream) -> TokenStream {
    let map_tiles = map.get_layer(0).unwrap().as_tile_layer().unwrap();

    let width = map_tiles.width().unwrap() * 2;
    let height = map_tiles.height().unwrap() * 2;

    let map_tiles = (0..(height * width)).map(|pos| {
        let x = pos % width;
        let y = pos / width;

        let tile = map_tiles.get_tile(x as i32 / 2, y as i32 / 2);

        match tile {
            Some(tile) => {
                let vflip = tile.flip_v;
                let hflip = tile.flip_h;

                // calculate the actual tile ID based on the properties here
                // since the tiles in tiled are 16x16, but we want to export to 8x8, we have to work this out carefully

                let tile_tileset_x = tile.id() % 9;
                let tile_tileset_y = tile.id() / 9;

                let x_offset = if (x % 2 == 0) ^ hflip { 0 } else { 1 };
                let y_offset = if (y % 2 == 0) ^ vflip { 0 } else { 1 };
                let gba_tile_id =
                    tile_tileset_x * 2 + x_offset + tile_tileset_y * 9 * 4 + y_offset * 9 * 2;
                let gba_tile_id = gba_tile_id as u16;

                let palette_id =
                    quote! { backgrounds::#background.palette_assignments[#gba_tile_id as usize] };
                quote! { TileSetting::new(#gba_tile_id, #hflip, #vflip, #palette_id) }
            }
            None => {
                quote! { TileSetting::new(1023, false, false, 0) }
            }
        }
    });

    quote! {&[#(#map_tiles),*]}
}

fn export_ui_tiles(map: &tiled::Map, background: TokenStream) -> TokenStream {
    let map_tiles = map.get_layer(0).unwrap().as_tile_layer().unwrap();

    let width = map_tiles.width().unwrap();
    let height = map_tiles.height().unwrap();

    let map_tiles = (0..(height * width)).map(|pos| {
        let x = pos % width;
        let y = pos / width;

        let tile = map_tiles.get_tile(x as i32, y as i32);

        match tile {
            Some(tile) => {
                let tile_id = tile.id() as u16;
                let vflip = tile.flip_v;
                let hflip = tile.flip_h;
                let palette_id =
                    quote! { backgrounds::#background.palette_assignments[#tile_id as usize] };
                quote! { TileSetting::new(#tile_id, #hflip, #vflip, #palette_id) }
            }
            None => {
                quote! { TileSetting::new(1023, false, false, 0) }
            }
        }
    });

    quote! {&[#(#map_tiles),*]}
}

fn bool_to_bit(bools: &[bool]) -> Vec<u8> {
    bools
        .chunks(8)
        .map(|x| {
            x.iter()
                .enumerate()
                .fold(0u8, |bits, (idx, &bit)| bits | ((bit as u8) << idx))
        })
        .collect()
}

#[test]
fn check_bool_to_bit() {
    let bools = [true, false, false, false, true, true, true, true];
    assert_eq!(bool_to_bit(&bools), [0b11110001]);
}
