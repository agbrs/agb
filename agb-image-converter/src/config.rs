use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

use crate::{Colour, Colours, TileSize};

pub(crate) fn parse(filename: &str) -> Box<dyn Config> {
    let config_toml =
        fs::read_to_string(filename).unwrap_or_else(|_| panic!("Failed to read file {}", filename));

    let config: ConfigV1 = toml::from_str(&config_toml).expect("Failed to parse file");

    if config.version != "1.0" {
        panic!(
            "Expected version of {} to be 1.0, got {}",
            filename, config.version
        );
    }

    Box::new(config)
}

pub(crate) trait Config {
    fn crate_prefix(&self) -> String;
    fn images(&self) -> HashMap<String, &dyn Image>;
    fn transparent_colour(&self) -> Option<Colour>;
}

pub(crate) trait Image {
    fn filename(&self) -> String;
    fn tile_size(&self) -> TileSize;
    fn colours(&self) -> Colours;
}

#[derive(Deserialize)]
pub struct ConfigV1 {
    version: String,
    crate_prefix: Option<String>,
    transparent_colour: Option<String>,

    image: HashMap<String, ImageV1>,
}

impl Config for ConfigV1 {
    fn crate_prefix(&self) -> String {
        self.crate_prefix
            .clone()
            .unwrap_or_else(|| "agb".to_owned())
    }

    fn images(&self) -> HashMap<String, &dyn Image> {
        self.image
            .iter()
            .map(|(filename, image)| (filename.clone(), image as &dyn Image))
            .collect()
    }

    fn transparent_colour(&self) -> Option<Colour> {
        if let Some(colour) = &self
            .transparent_colour
            .as_ref()
            .map(|colour| colour.parse().unwrap())
        {
            return Some(*colour);
        }

        self.image
            .values()
            .flat_map(|image| image.transparent_colour())
            .next()
    }
}

#[derive(Deserialize)]
pub struct ImageV1 {
    filename: String,
    transparent_colour: Option<String>,
    tile_size: TileSizeV1,
    colours: Option<u32>,
}

impl Image for ImageV1 {
    fn filename(&self) -> String {
        self.filename.clone()
    }

    fn tile_size(&self) -> TileSize {
        self.tile_size.into()
    }

    fn colours(&self) -> Colours {
        match self.colours {
            None | Some(16) => Colours::Colours16,
            Some(256) => Colours::Colours256,
            _ => panic!("colours must either not be set or 16 or 256"),
        }
    }
}

impl ImageV1 {
    fn transparent_colour(&self) -> Option<Colour> {
        self.transparent_colour
            .as_ref()
            .map(|colour| colour.parse().unwrap())
    }
}

#[derive(Deserialize, Clone, Copy)]
pub enum TileSizeV1 {
    #[serde(rename = "8x8")]
    Tile8,
    #[serde(rename = "16x16")]
    Tile16,
    #[serde(rename = "32x32")]
    Tile32,
}

impl From<TileSizeV1> for TileSize {
    fn from(item: TileSizeV1) -> Self {
        match item {
            TileSizeV1::Tile8 => TileSize::Tile8,
            TileSizeV1::Tile16 => TileSize::Tile16,
            TileSizeV1::Tile32 => TileSize::Tile32,
        }
    }
}
