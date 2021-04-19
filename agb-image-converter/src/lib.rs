use std::path;

mod colour;

pub use colour::Colour;

#[derive(Debug, Clone, Copy)]
pub enum TileSize {
    Tile8,
    Tile16,
}

pub struct ImageConverterConfig {
    pub transparent_colour: Option<Colour>,
    pub tile_size: TileSize,
    pub input_file: path::PathBuf,
    pub output_file: path::PathBuf,
}

pub fn convert_image(setting: &ImageConverterConfig) {}
