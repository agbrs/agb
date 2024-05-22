mod vram_manager;

pub struct BackgroundId(pub(crate) u8);

const TRANSPARENT_TILE_INDEX: u16 = 0xffff;

#[derive(Clone, Copy, Debug, Default)]
pub struct TileSetting {
    tile_id: u16,
    effect_bits: u16,
}

impl TileSetting {
    pub const BLANK: Self = TileSetting::new(TRANSPARENT_TILE_INDEX, false, false, 0);

    #[must_use]
    pub const fn new(tile_id: u16, hflip: bool, vflip: bool, palette_id: u8) -> Self {
        Self {
            tile_id,
            effect_bits: ((hflip as u16) << 10)
                | ((vflip as u16) << 11)
                | ((palette_id as u16) << 12),
        }
    }

    #[must_use]
    pub const fn hflip(self, should_flip: bool) -> Self {
        Self {
            effect_bits: self.effect_bits ^ ((should_flip as u16) << 10),
            ..self
        }
    }

    #[must_use]
    pub const fn vflip(self, should_flip: bool) -> Self {
        Self {
            effect_bits: self.effect_bits ^ ((should_flip as u16) << 11),
            ..self
        }
    }

    #[must_use]
    pub const fn palette(self, palette_id: u8) -> Self {
        Self {
            effect_bits: self.effect_bits ^ ((palette_id as u16) << 12),
            ..self
        }
    }

    fn index(self) -> u16 {
        self.tile_id
    }

    fn setting(self) -> u16 {
        self.effect_bits
    }
}
