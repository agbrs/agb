use agb_fixnum::Num;
use arbitrary_int::*;
use bitbybit::{bitenum, bitfield};

use crate::display::tiled::BackgroundId;

#[bitfield(u6)]
#[derive(Default)]
pub(crate) struct BlendTarget {
    #[bits(0..=3, rw)]
    backgrounds: u4,
    #[bits(4..=4, rw)]
    object: bool,
    #[bits(5..=5, rw)]
    backdrop: bool,
}

impl BlendTarget {
    pub fn enable_background(&mut self, background_id: impl Into<BackgroundId>) {
        self.set_backgrounds(self.backgrounds() | u4::new(1u8 << background_id.into().0));
    }

    pub fn enable_object(&mut self) {
        self.set_object(true);
    }

    pub fn enable_backdrop(&mut self) {
        self.set_backdrop(true);
    }
}

#[bitenum(u2, exhaustive = true)]
#[derive(Default)]
pub(crate) enum Effect {
    #[default]
    None,
    Alpha,
    Increase,
    Decrease,
}

#[bitfield(u16)]
#[derive(Default)]
pub(crate) struct BlendControlRegister {
    #[bits(0..=5, rw)]
    first_target: BlendTarget,
    #[bits(6..=7, rw)]
    colour_effect: Effect,
    #[bits(8..=13, rw)]
    second_target: BlendTarget,
}

#[bitfield(u16)]
#[derive(Default)]
pub(crate) struct BlendControlAlpha {
    #[bits(0..=4, rw)]
    first: u5,
    #[bits(8..=12, rw)]
    second: u5,
}

impl BlendControlAlpha {
    pub(crate) fn set_first_blend(&mut self, value: Num<u8, 4>) {
        self.set_first(u5::new(value.to_raw()));
    }

    pub(crate) fn set_second_blend(&mut self, value: Num<u8, 4>) {
        self.set_second(u5::new(value.to_raw()));
    }
}

#[bitfield(u16)]
#[derive(Default)]
pub(crate) struct BlendControlBrightness {
    #[bits(0..=4, rw)]
    brightness: u5,
}

impl BlendControlBrightness {
    pub(crate) fn set(&mut self, value: Num<u8, 4>) {
        self.set_brightness(u5::new(value.to_raw()));
    }
}
