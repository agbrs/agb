use core::{cell::RefCell, marker::PhantomData};

use super::{
    AffineBackgroundSize, AffineMap, AffineTiledMode, CreatableAffineTiledMode, MapLoan, TiledMode,
};
use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, tiled::AFFINE_BG_ID_OFFSET, DisplayMode, Priority},
};

pub struct Tiled2<'gba> {
    affine: RefCell<Bitarray<1>>,
    screenblocks: RefCell<Bitarray<1>>,
    phantom: PhantomData<&'gba ()>,
}

impl Tiled2<'_> {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Tiled2);

        let affine = RefCell::new(Bitarray::new());
        for i in 0..AFFINE_BG_ID_OFFSET {
            affine.borrow_mut().set(i, true);
        }

        Self {
            affine,
            screenblocks: Default::default(),
            phantom: PhantomData,
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

impl TiledMode for Tiled2<'_> {
    fn screenblocks(&self) -> &RefCell<Bitarray<1>> {
        &self.screenblocks
    }
}

impl CreatableAffineTiledMode for Tiled2<'_> {
    const AFFINE_BACKGROUNDS: usize = 2;

    fn affine(&self) -> &RefCell<Bitarray<1>> {
        &self.affine
    }
}
