use crate::{fixnum::Num, memory_mapped::set_bits};

use super::tiled::BackgroundID;

#[derive(Clone, Copy, Debug)]
pub enum Layer {
    Top = 0,
    Bottom = 1,
}

#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Off = 0,
    Normal = 0b01,
    FadeToWhite = 0b10,
    FadeToBlack = 0b11,
}

pub struct Blend {
    targets: u16,
    blend_weights: u16,
    fade_weight: u16,
}

const BLEND_CONTROL: *mut u16 = 0x0400_0050 as *mut _;
const BLEND_ALPHAS: *mut u16 = 0x0400_0052 as *mut _;

const BLEND_FADES: *mut u16 = 0x0400_0054 as *mut _;

impl Blend {
    pub fn reset_targets(&mut self) -> &mut Self {
        self.targets = 0;

        self
    }

    pub fn reset_weights(&mut self) -> &mut Self {
        self.blend_weights = 0;

        self
    }

    pub fn reset_fades(&mut self) -> &mut Self {
        self.fade_weight = 0;

        self
    }

    pub fn reset(&mut self) -> &mut Self {
        self.reset_targets().reset_fades().reset_weights()
    }

    pub fn set_background_enable(
        &mut self,
        layer: Layer,
        background: BackgroundID,
        enable: bool,
    ) -> &mut Self {
        let bit_to_modify = (background.0 as usize) + (layer as usize * 8);
        self.targets = set_bits(self.targets, enable as u16, 1, bit_to_modify);

        self
    }

    pub fn set_object_enable(&mut self, layer: Layer, enable: bool) -> &mut Self {
        let bit_to_modify = 0x4 + (layer as usize * 8);
        self.targets = set_bits(self.targets, enable as u16, 1, bit_to_modify);

        self
    }

    pub fn set_backdrop_enable(&mut self, layer: Layer, enable: bool) -> &mut Self {
        let bit_to_modify = 0x5 + (layer as usize * 8);
        self.targets = set_bits(self.targets, enable as u16, 1, bit_to_modify);

        self
    }

    pub fn set_blend_weight(&mut self, layer: Layer, value: Num<u8, 4>) -> &mut Self {
        self.blend_weights = set_bits(
            self.blend_weights,
            value.to_raw() as u16,
            5,
            (layer as usize) * 8,
        );

        self
    }

    pub fn set_fade(&mut self, value: Num<u8, 4>) -> &mut Self {
        self.fade_weight = value.to_raw() as u16;

        self
    }

    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) -> &mut Self {
        self.targets = set_bits(self.targets, blend_mode as u16, 2, 0x6);

        self
    }

    pub fn commit(&self) {
        unsafe {
            BLEND_CONTROL.write_volatile(self.targets);
            BLEND_ALPHAS.write_volatile(self.blend_weights);
            BLEND_FADES.write_volatile(self.fade_weight);
        }
    }
}
