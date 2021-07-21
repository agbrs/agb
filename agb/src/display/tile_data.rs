use crate::display::palette16::Palette16;

pub struct TileData {
    pub palettes: &'static [Palette16],
    pub tiles: &'static [u32],
    pub palette_assignments: &'static [u8],
}

impl TileData {
    pub const fn new(
        palettes: &'static [Palette16],
        tiles: &'static [u32],
        palette_assignments: &'static [u8],
    ) -> Self {
        TileData {
            palettes,
            tiles,
            palette_assignments,
        }
    }
}
