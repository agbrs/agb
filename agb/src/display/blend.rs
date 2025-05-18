#![warn(missing_docs)]
mod registers;

use registers::{BlendControlAlpha, BlendControlBrightness, BlendControlRegister};

use super::tiled::RegularBackgroundId;
use crate::{fixnum::Num, memory_mapped::MemoryMapped};

const BLEND_CONTROL: MemoryMapped<BlendControlRegister> = unsafe { MemoryMapped::new(0x0400_0050) };
const BLEND_ALPHA: MemoryMapped<BlendControlAlpha> = unsafe { MemoryMapped::new(0x0400_0052) };
const BLEND_BRIGHTNESS: MemoryMapped<BlendControlBrightness> =
    unsafe { MemoryMapped::new(0x0400_0054) };

/// The layers, top layer will be blended into the bottom layer
#[derive(Clone, Copy, Debug)]
pub enum Layer {
    /// Top layer gets blended into the bottom layer
    Top = 0,
    /// The bottom layer of the blend
    Bottom = 1,
}

/// Control the blending of two layers on the frame.
///
/// The Game Boy Advance offers very little in the way of alpha blending, which is something
/// you may be more familiar with if you've used other game engines or the layer settings in
/// image editors.
///
/// You can blend between a single [Top](Layer::Top) and [Bottom](Layer::Bottom) layer, and
/// only in one mode.
///
/// - [`alpha`](Blend::alpha) where some configurable amount of each layer will be rendered
///   to the screen.
/// - [`brighten`](Blend::brighten) where you fade the `Top` layer towards white
/// - [`darken`](Blend::darken) where you fade the `Top` layer towards black
/// - [`object_transparency`](Blend::object_transparency) which enables object transparency for certain objects
///
/// You can set what is in the `Top` and `Bottom` layer with the [`.layer()`](Blend::layer) method.
///
/// Note that for blending to actually work, whatever is in the `Top` layer has to be drawn after
/// anything in the `Bottom` layer (i.e. the `Top` layer's [`Priority`](crate::display::Priority)
/// must be _lower_ than the `Bottom` layer's `Priority`).
///
/// If you are using [`Windows`](super::Windows), blending won't happen unless you enable blending with
/// them, and then it will only work within the boundary of the window.
pub struct Blend {
    blend_control: registers::BlendControlRegister,
    alpha: registers::BlendControlAlpha,
    brightness: registers::BlendControlBrightness,
}

impl Blend {
    pub(crate) fn new() -> Self {
        Self {
            blend_control: Default::default(),
            alpha: Default::default(),
            brightness: Default::default(),
        }
    }

    /// Setup for a given [`Layer`].
    ///
    /// See the [`BlendLayer`] documentation for what they can be.
    pub fn layer(&mut self, layer: Layer) -> BlendLayer<'_> {
        BlendLayer { blend: self, layer }
    }

    /// Sets this blend effect to `alpha` which allows for a configurable amount of each layer to
    /// be rendered onto the screen.
    pub fn alpha(&mut self) -> BlendAlphaEffect<'_> {
        self.blend_control
            .set_colour_effect(registers::Effect::Alpha);
        BlendAlphaEffect { blend: self }
    }

    /// Fade the `Top` layer towards white by a configurable amount.
    pub fn brighten(&mut self) -> BlendFadeEffect<'_> {
        self.blend_control
            .set_colour_effect(registers::Effect::Increase);
        BlendFadeEffect { blend: self }
    }

    /// Fade the `Top` layer towards black by a configurable amount.
    pub fn darken(&mut self) -> BlendFadeEffect<'_> {
        self.blend_control
            .set_colour_effect(registers::Effect::Decrease);
        BlendFadeEffect { blend: self }
    }

    /// Enable object transparency for every object which has its
    /// [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending`.
    pub fn object_transparency(&mut self) -> BlendObjectTransparency<'_> {
        self.blend_control
            .set_colour_effect(registers::Effect::None);
        BlendObjectTransparency { blend: self }
    }

    fn set_background_enable(&mut self, layer: Layer, background_id: RegularBackgroundId) {
        self.with_target(layer, |mut target| {
            target.enable_background(background_id);
            target
        });
    }

    fn set_object_enable(&mut self, layer: Layer) {
        self.with_target(layer, |mut target| {
            target.enable_object();
            target
        });
    }

    fn set_backdrop_enable(&mut self, layer: Layer) {
        self.with_target(layer, |mut target| {
            target.enable_backdrop();
            target
        });
    }

    fn with_target(
        &mut self,
        layer: Layer,
        f: impl FnOnce(registers::BlendTarget) -> registers::BlendTarget,
    ) {
        match layer {
            Layer::Top => self
                .blend_control
                .set_first_target(f(self.blend_control.first_target())),
            Layer::Bottom => self
                .blend_control
                .set_second_target(f(self.blend_control.second_target())),
        }
    }

    fn set_layer_alpha(&mut self, layer: Layer, value: Num<u8, 4>) {
        match layer {
            Layer::Top => self.alpha.set_first_blend(value),
            Layer::Bottom => self.alpha.set_second_blend(value),
        }
    }

    fn set_fade(&mut self, value: Num<u8, 4>) {
        self.brightness.set(value);
    }

    pub(crate) fn commit(self) {
        BLEND_CONTROL.set(self.blend_control);
        BLEND_ALPHA.set(self.alpha);
        BLEND_BRIGHTNESS.set(self.brightness);
    }
}

/// Configure what can be blended on this layer.
///
/// By default, nothing is configured to be enabled. You can enable any amount of things on
/// a blend layer.
pub struct BlendLayer<'blend> {
    blend: &'blend mut Blend,
    layer: Layer,
}

impl BlendLayer<'_> {
    /// Enables a background for blending on this layer.
    pub fn enable_background(&mut self, background: RegularBackgroundId) -> &mut Self {
        self.blend.set_background_enable(self.layer, background);
        self
    }

    /// Enables object blending on this layer.
    ///
    /// This will only work for objects which have a
    /// [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending`.
    pub fn enable_object(&mut self) -> &mut Self {
        self.blend.set_object_enable(self.layer);
        self
    }

    /// Enables the backdrop for this layer.
    ///
    /// The backdrop is the 0th colour in the palette. It is the colour that is displayed
    /// when there is no background displaying anything at that location.
    pub fn enable_backdrop(&mut self) -> &mut Self {
        self.blend.set_backdrop_enable(self.layer);
        self
    }
}

/// Configure the alpha setting for an alpha blend
pub struct BlendAlphaEffect<'blend> {
    blend: &'blend mut Blend,
}

impl BlendAlphaEffect<'_> {
    /// The amount to blend this layer by.
    ///
    /// The final colour will be a weighted sum of the colours of each layer multiplied by `value`.
    /// So a `value` of `num!(0.5)` for both the top and the bottom layers will mean you get
    /// half of each colour added together.
    ///
    /// Any pixels which aren't shared by both layers will be drawn at their full pixel value.
    ///
    /// `value` must be between 0 and 1 inclusive. This function panics if value > 1.
    pub fn set_layer_alpha(&mut self, layer: Layer, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Layer alpha must be <= 1");
        self.blend.set_layer_alpha(layer, value);
        self
    }
}

/// Configure the fade effect for a darken or lighten blend
///
/// You can also enable object transparency while using `darken` or `lighten` using the
/// [`object_transparency()`](BlendFadeEffect::object_transparency()) function.
///
/// Fade effects will blend the [`Layer::Top`] layer towards either black or white by the amount
/// in [`.set_fade()`](BlendFadeEffect::set_fade()). This is useful if you want to fade part
/// of the screen to white or black, or apply some other effects like adding lightning to the
/// background.
///
/// Due to hardware restrictions, there are only 6 levels of fade available. Therefore, this
/// probably isn't the best effect for smoothly fading in and out as a transition, and that
/// is better left to changing the colour palette.
///
/// ```rust,no_run
/// # #![no_main]
/// # #![no_std]
/// use agb::fixnum::num;
///
/// # fn test(frame: &mut agb::display::GraphicsFrame, bg_id: agb::display::tiled::BackgroundId) {
/// frame
///    .blend()
///    .brighten()
///    .set_fade(num!(0.5))
///    .layer()
///    .enable_background(bg_id);
/// # }
/// ```
pub struct BlendFadeEffect<'blend> {
    blend: &'blend mut Blend,
}

impl BlendFadeEffect<'_> {
    /// Set how much this layer should fade to black or white.
    ///
    /// The `value` must be between 0 and 1 inclusive. This function panics if `value` > 1.
    /// Since the value is a `Num<u8, 4>`, there are only 6 possible levels of fading.
    pub fn set_fade(&mut self, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Layer fade must be <= 1");
        self.blend.set_fade(value);
        self
    }

    /// Control the object transparency as well if needed.
    pub fn object_transparency(&mut self) -> BlendObjectTransparency<'_> {
        BlendObjectTransparency { blend: self.blend }
    }

    /// Get the [`Layer`] the fade will effect.
    ///
    /// Equivalent to `frame.blend().layer(Layer::Top)`.
    pub fn layer(&mut self) -> BlendLayer<'_> {
        self.blend.layer(Layer::Top)
    }
}

/// Make given objects transparent to some level.
///
/// Every object with the [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending`
/// will have the same transparency level.
pub struct BlendObjectTransparency<'blend> {
    blend: &'blend mut Blend,
}

impl BlendObjectTransparency<'_> {
    /// Sets the transparency for all objects with their [`GraphicsMode`](crate::display::object::GraphicsMode)
    /// set to `AlphaBlending` to `value`.
    ///
    /// `value` must be a number between 0 and 1 inclusive, and will panic if `value` is greater than 1.
    pub fn set_alpha(&mut self, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Object alpha must be <= 1");
        self.blend.set_layer_alpha(Layer::Top, value);
        self
    }
}
