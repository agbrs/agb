use core::{cell::RefCell, marker::PhantomData};

use super::{
    CreatableRegularTiledMode, MapLoan, RegularBackgroundSize, RegularMap, RegularTiledMode,
    TileFormat, TiledMode,
};
use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, DisplayMode, Priority},
};

pub struct Tiled0<'gba> {
    regular: RefCell<Bitarray<1>>,
    screenblocks: RefCell<Bitarray<1>>,
    phantom: PhantomData<&'gba ()>,
}

impl Tiled0<'_> {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_mode(DisplayMode::Tiled0);

        Self {
            regular: Default::default(),
            screenblocks: Default::default(),
            phantom: PhantomData,
        }
    }

    pub fn background(
        &self,
        priority: Priority,
        size: RegularBackgroundSize,
        colours: TileFormat,
    ) -> MapLoan<'_, RegularMap> {
        self.regular_background(priority, size, colours)
    }
}

impl TiledMode for Tiled0<'_> {
    fn screenblocks(&self) -> &RefCell<Bitarray<1>> {
        &self.screenblocks
    }
}

impl CreatableRegularTiledMode for Tiled0<'_> {
    const REGULAR_BACKGROUNDS: usize = 4;

    fn regular(&self) -> &RefCell<Bitarray<1>> {
        &self.regular
    }
}
