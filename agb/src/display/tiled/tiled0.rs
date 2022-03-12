use core::cell::RefCell;

use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, set_graphics_settings, DisplayMode, GraphicsSettings, Priority},
};

use super::{MapLoan, RegularMap};

pub struct Tiled0 {
    regular: RefCell<Bitarray<1>>,
}

impl Tiled0 {
    pub(crate) unsafe fn new() -> Self {
        set_graphics_settings(GraphicsSettings::empty() | GraphicsSettings::SPRITE1_D);
        set_graphics_mode(DisplayMode::Tiled0);

        Self {
            regular: Default::default(),
        }
    }

    pub fn background(&self, priority: Priority) -> MapLoan<'_, RegularMap> {
        let mut regular = self.regular.borrow_mut();
        let new_background = regular.first_zero().unwrap();
        if new_background >= 4 {
            panic!("can only have 4 active backgrounds");
        }

        let bg = RegularMap::new(new_background as u8, (new_background + 16) as u8, priority);

        regular.set(new_background, true);

        MapLoan::new(bg, new_background as u8, &self.regular)
    }
}
