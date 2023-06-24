use core::{cell::RefCell, marker::PhantomData};

use super::{
    AffineBackgroundSize, AffineMap, AffineTiledMode, CreatableAffineTiledMode,
    CreatableRegularTiledMode, MapLoan, RegularBackgroundSize, RegularMap, RegularTiledMode,
    TileFormat, TiledMode,
};
use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, tiled::AFFINE_BG_ID_OFFSET, DisplayMode, Priority},
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

    pub fn regular(
        &self,
        priority: Priority,
        size: RegularBackgroundSize,
        colours: TileFormat,
    ) -> MapLoan<'_, RegularMap> {
        self.regular_background(priority, size, colours)
    }

    pub fn affine(&self, priority: Priority, size: AffineBackgroundSize) -> MapLoan<'_, AffineMap> {
        self.affine_background(priority, size)
    }
}

impl TiledMode for Tiled1 {
    fn screenblocks(&self) -> &RefCell<Bitarray<1>> {
        &self.screenblocks
    }
}

impl CreatableRegularTiledMode for Tiled1 {
    const REGULAR_BACKGROUNDS: usize = 2;

    fn regular(&self) -> &RefCell<Bitarray<1>> {
        &self.regular
    }
}

impl CreatableAffineTiledMode for Tiled1 {
    const AFFINE_BACKGROUNDS: usize = 1;

    fn affine(&self) -> &RefCell<Bitarray<1>> {
        &self.affine
    }
}
