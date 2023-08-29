use super::tiled::TileSetting;

#[non_exhaustive]
pub struct TileData {
    pub tiles: &'static [u8],
    pub tile_settings: &'static [TileSetting],
}

impl TileData {
    #[must_use]
    pub const fn new(tiles: &'static [u8], tile_settings: &'static [TileSetting]) -> Self {
        TileData {
            tiles,
            tile_settings,
        }
    }
}
