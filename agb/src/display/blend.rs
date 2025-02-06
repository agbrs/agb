mod registers;

use registers::{BlendControlAlpha, BlendControlBrightness, BlendControlRegister};

use super::tiled::BackgroundId;
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

    pub fn layer<'blend>(&'blend mut self, layer: Layer) -> BlendLayer<'blend> {
        BlendLayer { blend: self, layer }
    }

    pub fn alpha<'blend>(&'blend mut self) -> BlendAlphaEffect<'blend> {
        self.blend_control
            .set_colour_effect(registers::Effect::Alpha);
        BlendAlphaEffect { blend: self }
    }

    pub fn brighten<'blend>(&'blend mut self) -> BlendFadeEffect<'blend> {
        self.blend_control
            .set_colour_effect(registers::Effect::Increase);
        BlendFadeEffect { blend: self }
    }

    pub fn darken<'blend>(&'blend mut self) -> BlendFadeEffect<'blend> {
        self.blend_control
            .set_colour_effect(registers::Effect::Decrease);
        BlendFadeEffect { blend: self }
    }

    pub fn object_transparency<'blend>(&'blend mut self) -> BlendObjectTransparency<'blend> {
        self.blend_control
            .set_colour_effect(registers::Effect::None);
        BlendObjectTransparency { blend: self }
    }

    fn set_background_enable(&mut self, layer: Layer, background_id: BackgroundId) {
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

pub struct BlendLayer<'blend> {
    blend: &'blend mut Blend,
    layer: Layer,
}

impl BlendLayer<'_> {
    /// Enables a background for blending on this layer
    pub fn enable_background(&mut self, background: BackgroundId) -> &mut Self {
        self.blend.set_background_enable(self.layer, background);
        self
    }

    pub fn enable_object(&mut self) -> &mut Self {
        self.blend.set_object_enable(self.layer);
        self
    }

    pub fn enable_backdrop(&mut self) -> &mut Self {
        self.blend.set_backdrop_enable(self.layer);
        self
    }
}

pub struct BlendAlphaEffect<'blend> {
    blend: &'blend mut Blend,
}

impl BlendAlphaEffect<'_> {
    pub fn set_layer_alpha(&mut self, layer: Layer, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Layer alpha must be <= 1");
        self.blend.set_layer_alpha(layer, value);
        self
    }
}

pub struct BlendFadeEffect<'blend> {
    blend: &'blend mut Blend,
}

impl BlendFadeEffect<'_> {
    pub fn set_fade(&mut self, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Layer fade must be <= 1");
        self.blend.set_fade(value);
        self
    }

    pub fn set_object_alpha(&mut self, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Object alpha must be <= 1");
        self.blend.set_layer_alpha(Layer::Top, value);
        self
    }
}

pub struct BlendObjectTransparency<'blend> {
    blend: &'blend mut Blend,
}

impl BlendObjectTransparency<'_> {
    pub fn set_alpha(&mut self, value: Num<u8, 4>) -> &mut Self {
        assert!(value <= 1.into(), "Object alpha must be <= 1");
        self.blend.set_layer_alpha(Layer::Top, value);
        self
    }
}
