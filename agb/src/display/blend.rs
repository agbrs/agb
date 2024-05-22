//! This controls the blending modes on the GBA.
//!
//! For now a description of how blending can be used is found on [the tonc page
//! for graphic
//! effects](https://www.coranac.com/tonc/text/gfx.htm#ssec-bld-gba). See the
//! [Blend] struct for all the functions to manage blend effects. You acquire
//! the Blend struct through the [Display][super::Display] struct.
//! ```no_run
//! # #![no_main]
//! # #![no_std]
//! # fn blend(mut gba: agb::Gba) {
//! let mut blend = gba.display.blend.get();
//! // ...
//! # }
//! ```
//! where `gba` is a mutable [Gba][crate::Gba] struct.

use core::marker::PhantomData;

use crate::{fixnum::Num, memory_mapped::set_bits};

use super::tiled::BackgroundId;

/// The layers, top layer will be blended into the bottom layer
#[derive(Clone, Copy, Debug)]
pub enum Layer {
    /// Top layer gets blended into the bottom layer
    Top = 0,
    /// The bottom layer of the blend
    Bottom = 1,
}

/// The different blend modes available on the GBA
#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    // No blending
    Off = 0,
    // Additive blending, use the [Blend::set_blend_weight] function to use this
    Normal = 0b01,
    // Brighten, use the [Blend::set_fade] to use this
    FadeToWhite = 0b10,
    // Darken, use the [Blend::set_fade] to use this
    FadeToBlack = 0b11,
}

/// Manages the blending, won't cause anything to change unless [Blend::commit]
/// is called.
pub struct Blend<'gba> {
    targets: u16,
    blend_weights: u16,
    fade_weight: u16,
    phantom: PhantomData<&'gba ()>,
}

/// When making many modifications to a layer, it is convenient to operate on
/// that layer directly. This is created by the [Blend::layer] function and
/// operates on that layer.
pub struct BlendLayer<'blend, 'gba> {
    blend: &'blend mut Blend<'gba>,
    layer: Layer,
}

impl BlendLayer<'_, '_> {
    /// Set whether a background is enabled for blending on this layer.
    pub fn set_background_enable(&mut self, background: BackgroundId, enable: bool) -> &mut Self {
        self.blend
            .set_background_enable(self.layer, background, enable);

        self
    }

    /// Set whether objects are enabled for blending on this layer.
    pub fn set_object_enable(&mut self, enable: bool) -> &mut Self {
        self.blend.set_object_enable(self.layer, enable);

        self
    }

    /// Set whether the backdrop contributes to the blend on this layer.
    /// The backdrop is transparent colour, the colour rendered when nothing is
    /// in it's place.
    pub fn set_backdrop_enable(&mut self, enable: bool) -> &mut Self {
        self.blend.set_backdrop_enable(self.layer, enable);

        self
    }

    /// Set the weight for the blend on this layer.
    pub fn set_blend_weight(&mut self, value: Num<u8, 4>) -> &mut Self {
        self.blend.set_blend_weight(self.layer, value);

        self
    }
}

const BLEND_CONTROL: *mut u16 = 0x0400_0050 as *mut _;
const BLEND_ALPHAS: *mut u16 = 0x0400_0052 as *mut _;

const BLEND_FADES: *mut u16 = 0x0400_0054 as *mut _;

impl<'gba> Blend<'gba> {
    pub(crate) fn new() -> Self {
        let blend = Self {
            targets: 0,
            blend_weights: 0,
            fade_weight: 0,
            phantom: PhantomData,
        };
        blend.commit();

        blend
    }

    /// Reset the targets to all disabled, the targets control which layers are
    /// enabled for blending.
    pub fn reset_targets(&mut self) -> &mut Self {
        self.targets = 0;

        self
    }

    /// Reset the blend weights
    pub fn reset_weights(&mut self) -> &mut Self {
        self.blend_weights = 0;

        self
    }

    /// Reset the brighten and darken weights
    pub fn reset_fades(&mut self) -> &mut Self {
        self.fade_weight = 0;

        self
    }

    /// Reset targets, blend weights, and fades
    pub fn reset(&mut self) -> &mut Self {
        self.reset_targets().reset_fades().reset_weights()
    }

    /// Creates a layer object whose functions work only on that layer,
    /// convenient when performing multiple operations on that layer without the
    /// need of specifying the layer every time.
    pub fn layer(&mut self, layer: Layer) -> BlendLayer<'_, 'gba> {
        BlendLayer { blend: self, layer }
    }

    /// Set whether a background is enabled for blending on a particular layer.
    pub fn set_background_enable(
        &mut self,
        layer: Layer,
        background: BackgroundId,
        enable: bool,
    ) -> &mut Self {
        let bit_to_modify = (background.0 as usize) + (layer as usize * 8);
        self.targets = set_bits(self.targets, enable as u16, 1, bit_to_modify);

        self
    }

    /// Set whether objects are enabled for blending on a particular layer
    pub fn set_object_enable(&mut self, layer: Layer, enable: bool) -> &mut Self {
        let bit_to_modify = 0x4 + (layer as usize * 8);
        self.targets = set_bits(self.targets, enable as u16, 1, bit_to_modify);

        self
    }

    /// Set whether the backdrop contributes to the blend on a particular layer.
    /// The backdrop is transparent colour, the colour rendered when nothing is
    /// in it's place.
    pub fn set_backdrop_enable(&mut self, layer: Layer, enable: bool) -> &mut Self {
        let bit_to_modify = 0x5 + (layer as usize * 8);
        self.targets = set_bits(self.targets, enable as u16, 1, bit_to_modify);

        self
    }

    /// Set the weight for the blend on a particular layer.
    pub fn set_blend_weight(&mut self, layer: Layer, value: Num<u8, 4>) -> &mut Self {
        self.blend_weights = set_bits(
            self.blend_weights,
            value.to_raw() as u16,
            5,
            (layer as usize) * 8,
        );

        self
    }

    /// Set the fade of brighten or darken
    pub fn set_fade(&mut self, value: Num<u8, 4>) -> &mut Self {
        self.fade_weight = value.to_raw() as u16;

        self
    }

    /// Set the current blend mode
    pub fn set_blend_mode(&mut self, blend_mode: BlendMode) -> &mut Self {
        self.targets = set_bits(self.targets, blend_mode as u16, 2, 0x6);

        self
    }

    /// Commits the current state, should be called near after a call to wait
    /// for next vblank.
    pub fn commit(&self) {
        unsafe {
            BLEND_CONTROL.write_volatile(self.targets);
            BLEND_ALPHAS.write_volatile(self.blend_weights);
            BLEND_FADES.write_volatile(self.fade_weight);
        }
    }
}

impl Drop for Blend<'_> {
    fn drop(&mut self) {
        self.reset().commit();
    }
}
