use core::cell::RefCell;

use super::{AffineBackgroundSize, AffineMap, AffineTiledMode, MapLoan, TiledMode};
use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, tiled::AFFINE_BG_ID_OFFSET, DisplayMode, Priority},
};

pub struct Tiled2 {
    affine: RefCell<Bitarray<1>>,
    screenblocks: RefCell<Bitarray<1>>,
}

impl Tiled2 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Tiled2);

        let affine = RefCell::new(Bitarray::new());
        for i in 0..AFFINE_BG_ID_OFFSET {
            affine.borrow_mut().set(i, true);
        }

        Self {
            affine,
            screenblocks: Default::default(),
        }
    }

    pub fn background(
        &self,
        priority: Priority,
        size: AffineBackgroundSize,
    ) -> MapLoan<'_, AffineMap> {
        self.affine_background(priority, size)
    }
}

impl TiledMode for Tiled2 {
    const REGULAR_BACKGROUNDS: usize = 0;
    const AFFINE_BACKGROUNDS: usize = 2;

    fn screenblocks(&self) -> &RefCell<Bitarray<1>> {
        &self.screenblocks
    }

    fn regular(&self) -> &RefCell<Bitarray<1>> {
        unimplemented!()
    }

    fn affine(&self) -> &RefCell<Bitarray<1>> {
        &self.affine
    }
}
