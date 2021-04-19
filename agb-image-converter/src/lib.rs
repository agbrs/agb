#[derive(Debug, Clone, Copy)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
}

impl Colour {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Colour { r, g, b }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TileSize {
    Tile8,
    Tile16,
}

pub struct ImageConverter {}

pub struct ImageConverterConfigBuilder {
    transparent_colour: Option<Colour>,
    tile_size: TileSize,
}

impl ImageConverterConfigBuilder {
    pub fn new_with_tile_size(tile_size: TileSize) -> Self {
        ImageConverterConfigBuilder {
            tile_size,
            transparent_colour: None,
        }
    }

    pub fn with_transparent_colour(&self, transparent_colour: Colour) -> Self {
        ImageConverterConfigBuilder {
            transparent_colour: Some(transparent_colour),
            ..*self
        }
    }
}
