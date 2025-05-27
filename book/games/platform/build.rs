use std::error::Error;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use tiled::{
    FilesystemResourceReader, FiniteTileLayer, Layer, Map, ObjectLayer, PropertyValue,
    ResourceReader, TileLayer,
};

use std::io::Write;

static LEVELS: &[&str] = &["level_01.tmx"];

struct BuildResourceReader;

impl ResourceReader for BuildResourceReader {
    type Resource = <FilesystemResourceReader as ResourceReader>::Resource;

    type Error = <FilesystemResourceReader as ResourceReader>::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        println!("cargo::rerun-if-changed={}", path.to_string_lossy());
        FilesystemResourceReader.read_from(path)
    }
}

#[derive(Debug, Clone, Copy)]
struct TileInfo {
    id: Option<u32>,
    colliding: bool,
    win: bool,
}

#[derive(Debug, Clone)]
struct Level {
    size: (u32, u32),
    tiles: Vec<TileInfo>,
    player_start: (i32, i32),
}

impl ToTokens for Level {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let background_tiles = self.tiles.iter().map(|x| match x.id {
            Some(x) => quote! { TILES[#x as usize] },
            None => quote! { TileSetting::BLANK },
        });
        let collision_map = self.tiles.chunks(8).map(|x| {
            x.iter()
                .map(|x| x.colliding as u8)
                .fold(0u8, |a, b| (a >> 1) | (b << 7))
        });

        let winning_map = self.tiles.chunks(8).map(|x| {
            x.iter()
                .map(|x| x.win as u8)
                .fold(0u8, |a, b| (a >> 1) | (b << 7))
        });

        let (player_x, player_y) = self.player_start;
        let (width, height) = self.size;

        quote! {
            Level {
                width: #width,
                height: #height,
                background: &[#(#background_tiles),*],
                collision_map: &[#(#collision_map),*],
                winning_map: &[#(#winning_map),*],
                player_start: (#player_x, #player_y),
            }
        }
        .to_tokens(tokens)
    }
}

fn import_level(level: &str) -> Result<Level, Box<dyn Error>> {
    let level = tiled::Loader::with_reader(BuildResourceReader).load_tmx_map(level)?;

    let map = level.get_tile_layer("Level");
    let objs = level.get_object_layer("Objects");

    let width = map.width();
    let height = map.height();

    let mut tiles = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let tile = match map.get_tile(x as i32, y as i32) {
                Some(tile) => {
                    let properties = &tile.get_tile().unwrap().properties;

                    let colliding = properties["COLLISION"] == PropertyValue::BoolValue(true);
                    let win = properties["WIN"] == PropertyValue::BoolValue(true);
                    TileInfo {
                        colliding,
                        win,
                        id: Some(tile.id()),
                    }
                }
                None => TileInfo {
                    colliding: false,
                    win: false,
                    id: None,
                },
            };

            tiles.push(tile);
        }
    }

    let player = objs
        .objects()
        .find(|x| x.name == "PLAYER")
        .expect("Should be able to find the player");

    let player_x = player.x as i32;
    let player_y = player.y as i32;

    Ok(Level {
        size: (width, height),
        tiles,
        player_start: (player_x, player_y),
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut file =
        std::fs::File::create(format!("{}/levels.rs", std::env::var("OUT_DIR").unwrap()))?;

    for (number, level) in LEVELS.iter().enumerate() {
        let ident = quote::format_ident!("LEVEL_{}", number);
        let level = import_level(&format!("tiled/{level}"))?;
        let content = quote! {
            static #ident: Level = #level;
        };
        writeln!(file, "{content}")?;
    }

    let levels = (0..LEVELS.len()).map(|x| quote::format_ident!("LEVEL_{}", x));
    writeln!(
        file,
        "{}",
        quote! {
            pub static LEVELS: &[&Level] = &[#(&#levels),*];
        }
    )?;

    Ok(())
}

trait GetLayer {
    fn get_layer_by_name(&self, name: &str) -> Layer;
    fn get_tile_layer(&self, name: &str) -> FiniteTileLayer;
    fn get_object_layer(&self, name: &str) -> ObjectLayer;
}

impl GetLayer for Map {
    fn get_layer_by_name(&self, name: &str) -> Layer {
        self.layers().find(|x| x.name == name).unwrap()
    }
    fn get_tile_layer(&self, name: &str) -> FiniteTileLayer {
        match self.get_layer_by_name(name).as_tile_layer().unwrap() {
            TileLayer::Finite(finite_tile_layer) => finite_tile_layer,
            TileLayer::Infinite(_) => panic!("Infinite tile layer not supported"),
        }
    }

    fn get_object_layer(&self, name: &str) -> ObjectLayer {
        self.get_layer_by_name(name).as_object_layer().unwrap()
    }
}
