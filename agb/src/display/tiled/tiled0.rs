use core::cell::RefCell;

use super::{MapLoan, RegularBackgroundSize, RegularMap, RegularTiledMode, TiledMode};
use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, DisplayMode, Priority},
};

pub struct Tiled0 {
    regular: RefCell<Bitarray<1>>,
    screenblocks: RefCell<Bitarray<1>>,
}

impl Tiled0 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Tiled0);

        Self {
            regular: Default::default(),
            screenblocks: Default::default(),
        }
    }

    pub fn background(
        &self,
        priority: Priority,
        size: RegularBackgroundSize,
    ) -> MapLoan<'_, RegularMap> {
        self.regular_background(priority, size)
    }
}

impl TiledMode for Tiled0 {
    const REGULAR_BACKGROUNDS: usize = 4;
    const AFFINE_BACKGROUNDS: usize = 0;

    fn screenblocks(&self) -> &RefCell<Bitarray<1>> {
        &self.screenblocks
    }

    fn regular(&self) -> &RefCell<Bitarray<1>> {
        &self.regular
    }

    fn affine(&self) -> &RefCell<Bitarray<1>> {
        unimplemented!()
    }
}
