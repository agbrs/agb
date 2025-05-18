use agb_fixnum::Num;
use bilge::prelude::*;

use crate::display::tiled::RegularBackgroundId;

#[bitsize(6)]
#[derive(FromBits, Default, Clone, Copy)]
pub(crate) struct BlendTarget {
    backgrounds: u4,
    object: bool,
    backdrop: bool,
}

impl BlendTarget {
    pub fn enable_background(&mut self, background_id: RegularBackgroundId) {
        self.set_backgrounds(self.backgrounds() | u4::new(1u8 << background_id.0));
    }

    pub fn enable_object(&mut self) {
        self.set_object(true);
    }

    pub fn enable_backdrop(&mut self) {
        self.set_backdrop(true);
    }
}

#[bitsize(2)]
#[derive(FromBits, Default, Clone, Copy)]
pub(crate) enum Effect {
    #[default]
    None,
    Alpha,
    Increase,
    Decrease,
}

#[bitsize(16)]
#[derive(Default, Clone, Copy)]
pub(crate) struct BlendControlRegister {
    pub(crate) first_target: BlendTarget,
    pub(crate) colour_effect: Effect,
    pub(crate) second_target: BlendTarget,
    _unused: u2,
}

#[bitsize(16)]
#[derive(Default, Clone, Copy)]
pub(crate) struct BlendControlAlpha {
    first: u5,
    _unused: u3,
    second: u5,
    _unused2: u3,
}

impl BlendControlAlpha {
    pub(crate) fn set_first_blend(&mut self, value: Num<u8, 4>) {
        self.set_first(u5::new(value.to_raw()));
    }

    pub(crate) fn set_second_blend(&mut self, value: Num<u8, 4>) {
        self.set_second(u5::new(value.to_raw()));
    }
}

#[bitsize(16)]
#[derive(Default, Clone, Copy)]
pub(crate) struct BlendControlBrightness {
    brightness: u5,
    _unused: u11,
}

impl BlendControlBrightness {
    pub(crate) fn set(&mut self, value: Num<u8, 4>) {
        self.set_brightness(u5::new(value.to_raw()));
    }
}
