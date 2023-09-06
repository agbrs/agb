use super::tiled::{TileSet, TileSetting};

#[non_exhaustive]
pub struct TileData {
    pub tiles: TileSet<'static>,
    pub tile_settings: &'static [TileSetting],
}

impl TileData {
    #[must_use]
    pub const fn new(tiles: TileSet<'static>, tile_settings: &'static [TileSetting]) -> Self {
        TileData {
            tiles,
            tile_settings,
        }
    }
}
