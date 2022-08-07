use core::cell::RefCell;

use super::TiledMode;
use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, tiled::AFFINE_BG_ID_OFFSET, DisplayMode},
};

pub struct Tiled1 {
    regular: RefCell<Bitarray<1>>,
    affine: RefCell<Bitarray<1>>,
    screenblocks: RefCell<Bitarray<1>>,
}

impl Tiled1 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Tiled1);

        let affine = RefCell::new(Bitarray::new());
        for i in 0..AFFINE_BG_ID_OFFSET {
            affine.borrow_mut().set(i, true);
        }

        Self {
            regular: Default::default(),
            affine,
            screenblocks: Default::default(),
        }
    }
}

impl TiledMode for Tiled1 {
    const REGULAR_BACKGROUNDS: usize = 2;
    const AFFINE_BACKGROUNDS: usize = 1;

    fn screenblocks(&self) -> &RefCell<Bitarray<1>> {
        &self.screenblocks
    }

    fn regular(&self) -> &RefCell<Bitarray<1>> {
        &self.regular
    }

    fn affine(&self) -> &RefCell<Bitarray<1>> {
        &self.affine
    }
}
