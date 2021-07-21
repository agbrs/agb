use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

use crate::{TileSize, Colour};

pub(crate) fn parse(filename: &str) -> Box<dyn Config> {
    let config_toml = fs::read_to_string(filename).expect(&format!("Failed to read file {}", filename));
    
    let config: ConfigV1 = toml::from_str(&config_toml).expect("Failed to parse file");

    if config.version != "1.0" {
        panic!("Expected version of {} to be 1.0, got {}", filename, config.version);
    }

    Box::new(config)
}

pub(crate) trait Config {
    fn crate_prefix(&self) -> String;
    fn images(&self) -> HashMap<String, &dyn Image>;
}

pub(crate) trait Image {
    fn filename(&self) -> String;
    fn transparent_colour(&self) -> Option<Colour>;
    fn tilesize(&self) -> TileSize;
}

#[derive(Deserialize)]
pub struct ConfigV1 {
    version: String,
    crate_prefix: Option<String>,

    image: HashMap<String, ImageV1>,
}

impl Config for ConfigV1 {
    fn crate_prefix(&self) -> String {
        self.crate_prefix.clone().unwrap_or("agb".to_owned())
    }

    fn images(&self) -> HashMap<String, &dyn Image> {
        self.image.iter()
        .map(|(filename, image)| (
            filename.clone(), image as &dyn Image
        )).collect()
    }
}

#[derive(Deserialize)]
pub struct ImageV1 {
    filename: String,
    transparent_colour: Option<String>,
    tile_size: TileSizeV1,
}

impl Image for ImageV1 {
    fn filename(&self) -> String {
        self.filename.clone()
    }

    fn transparent_colour(&self) -> Option<Colour> {
        if let Some(colour) = &self.transparent_colour {
            if colour.len() != 6 {
                panic!("Expected colour to be 6 characters, got {}", colour);
            }

            let r = u8::from_str_radix(&colour[0..2], 16).unwrap();
            let g = u8::from_str_radix(&colour[2..4], 16).unwrap();
            let b = u8::from_str_radix(&colour[4..6], 16).unwrap();

            return Some(Colour::from_rgb(r, g, b));
        }

        None
    }

    fn tilesize(&self) -> TileSize {
        self.tile_size.into()
    }
}

#[derive(Deserialize, Clone, Copy)]
pub enum TileSizeV1 {
    #[serde(rename = "8x8")]
    Tile8,
    #[serde(rename = "16x16")]
    Tile16,
}

impl Into<TileSize> for TileSizeV1 {
    fn into(self) -> TileSize {
        match self {
            TileSizeV1::Tile8 => TileSize::Tile8,
            TileSizeV1::Tile16 => TileSize::Tile16,
        }
    }
}