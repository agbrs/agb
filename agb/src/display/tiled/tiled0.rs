use core::cell::RefCell;

use crate::{
    bitarray::Bitarray,
    display::{set_graphics_mode, DisplayMode, Priority},
};

use super::{MapLoan, RegularBackgroundSize, RegularMap};

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
        let mut regular = self.regular.borrow_mut();
        let new_background = regular.first_zero().unwrap();
        if new_background >= 4 {
            panic!("can only have 4 active backgrounds");
        }

        let num_screenblocks = size.num_screen_blocks();
        let mut screenblocks = self.screenblocks.borrow_mut();

        let screenblock = find_screenblock_gap(&screenblocks, num_screenblocks);
        for id in screenblock..(screenblock + num_screenblocks) {
            screenblocks.set(id, true);
        }

        let bg = RegularMap::new(new_background as u8, screenblock as u8 + 16, priority, size);

        regular.set(new_background, true);

        MapLoan::new(
            bg,
            new_background as u8,
            screenblock as u8,
            num_screenblocks as u8,
            &self.regular,
            &self.screenblocks,
        )
    }
}

fn find_screenblock_gap(screenblocks: &Bitarray<1>, gap: usize) -> usize {
    let mut candidate = 0;

    'outer: while candidate < 16 - gap {
        let starting_point = candidate;
        for attempt in starting_point..(starting_point + gap) {
            if screenblocks.get(attempt) == Some(true) {
                candidate = attempt + 1;
                continue 'outer;
            }
        }

        return candidate;
    }

    panic!(
        "Failed to find screenblock gap of at least {} elements",
        gap
    );
}
