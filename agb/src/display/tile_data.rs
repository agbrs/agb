#[non_exhaustive]
pub struct TileData {
    pub tiles: &'static [u8],
    pub palette_assignments: &'static [u8],
}

impl TileData {
    #[must_use]
    pub const fn new(tiles: &'static [u8], palette_assignments: &'static [u8]) -> Self {
        TileData {
            tiles,
            palette_assignments,
        }
    }
}
