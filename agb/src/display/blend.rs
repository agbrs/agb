#![warn(missing_docs)]
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

    fn reset(&mut self) {
        self.blend_control = Default::default();
        self.alpha = Default::default();
        self.brightness = Default::default();
    }

    /// Sets this blend effect to `alpha` which allows for a configurable amount of each layer to
    /// be rendered onto the screen.
    ///
    /// The final colour will be a weighted sum of the colours of each layer multiplied by `value`.
    /// So a `value` of `num!(0.5)` for both the top and the bottom layers will mean you get
    /// half of each colour added together.
    ///
    /// Any pixels which aren't shared by both layers will be drawn at their full pixel value.
    ///
    /// Values must be between 0 and 1 inclusive. This function panics if value > 1.
    pub fn alpha(
        &mut self,
        top_layer_alpha: Num<u8, 4>,
        bottom_layer_alpha: Num<u8, 4>,
    ) -> BlendAlphaEffect<'_> {
        self.reset();

        self.blend_control
            .set_colour_effect(registers::Effect::Alpha);
        self.set_layer_alpha(Layer::Top, top_layer_alpha);
        self.set_layer_alpha(Layer::Bottom, bottom_layer_alpha);

        BlendAlphaEffect { blend: self }
    }

    /// Fade the `Top` layer towards white by a configurable amount.
    ///
    /// The `amount` must be between 0 and 1 inclusive. This function panics if `amount` > 1.
    /// Since the amount is a `Num<u8, 4>`, there are only 6 possible levels of fading.
    pub fn brighten(&mut self, amount: Num<u8, 4>) -> BlendFadeEffect<'_> {
        self.reset();

        self.blend_control
            .set_colour_effect(registers::Effect::Increase);
        self.set_fade(amount);

        BlendFadeEffect { blend: self }
    }

    /// Fade the `Top` layer towards black by a configurable amount.
    ///
    /// The `amount` must be between 0 and 1 inclusive. This function panics if `amount` > 1.
    /// Since the amount is a `Num<u8, 4>`, there are only 6 possible levels of fading.
    pub fn darken(&mut self, amount: Num<u8, 4>) -> BlendFadeEffect<'_> {
        self.reset();

        self.blend_control
            .set_colour_effect(registers::Effect::Decrease);
        self.set_fade(amount);

        BlendFadeEffect { blend: self }
    }

    /// Enable object transparency for every object which has its
    /// [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending`.
    ///
    /// The final colour will be a weighted sum of the colours of each pixel in the object multiplied
    /// by `top_layer_alpha` and any enabled background layers by `bottom_layer_alpha`.
    ///
    /// So an `alpha` of `num!(0.5)` for both the top and the bottom layers will mean you get
    /// half of each colour added together. You'll probably want `top_layer_alpha` + `bottom_layer_alpha`
    /// to equal `1`, although this isn't required if you want some crazy effects.
    pub fn object_transparency(
        &mut self,
        top_layer_alpha: Num<u8, 4>,
        bottom_layer_alpha: Num<u8, 4>,
    ) -> BlendObjectTransparency<'_> {
        self.reset();

        assert!(top_layer_alpha <= 1.into());
        assert!(bottom_layer_alpha <= 1.into());

        self.blend_control
            .set_colour_effect(registers::Effect::None);
        self.set_layer_alpha(Layer::Top, top_layer_alpha);
        self.set_layer_alpha(Layer::Bottom, bottom_layer_alpha);

        BlendObjectTransparency { blend: self }
    }

    fn set_background_enable(&mut self, layer: Layer, background_id: impl Into<BackgroundId>) {
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
        assert!(value <= 1.into());

        match layer {
            Layer::Top => self.alpha.set_first_blend(value),
            Layer::Bottom => self.alpha.set_second_blend(value),
        }
    }

    fn set_fade(&mut self, value: Num<u8, 4>) {
        assert!(value <= 1.into());
        self.brightness.set(value);
    }

    pub(crate) fn commit(self) {
        BLEND_CONTROL.set(self.blend_control);
        BLEND_ALPHA.set(self.alpha);
        BLEND_BRIGHTNESS.set(self.brightness);
    }
}

/// Configure the alpha setting for an alpha blend
pub struct BlendAlphaEffect<'blend> {
    blend: &'blend mut Blend,
}

impl BlendAlphaEffect<'_> {
    /// Enables a background for blending on `layer`.
    pub fn enable_background(
        &mut self,
        layer: Layer,
        background: impl Into<BackgroundId>,
    ) -> &mut Self {
        self.blend.set_background_enable(layer, background);
        self
    }

    /// Enables object blending on `layer`.
    ///
    /// This will only work for objects which have a
    /// [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending`.
    pub fn enable_object(&mut self, layer: Layer) -> &mut Self {
        self.blend.set_object_enable(layer);
        self
    }

    /// Enables the backdrop for `layer`.
    ///
    /// The backdrop is the 0th colour in the palette. It is the colour that is displayed
    /// when there is no background displaying anything at that location.
    pub fn enable_backdrop(&mut self, layer: Layer) -> &mut Self {
        self.blend.set_backdrop_enable(layer);
        self
    }
}

/// Configure the fade effect for a darken or lighten blend.
///
/// Fade effects will blend the [`Layer::Top`] layer towards either black or white by the amount
/// given to the `.brighten()` or `.darken()` methods on [`Blend`]. This is useful if you want to fade part
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
///    .brighten(num!(0.5))
///    .enable_background(bg_id);
/// # }
/// ```
pub struct BlendFadeEffect<'blend> {
    blend: &'blend mut Blend,
}

impl BlendFadeEffect<'_> {
    /// Enables a background for blending.
    ///
    /// This will cause the background to be faded towards black or white (depending on whether
    /// this is darken or lighten).
    pub fn enable_background(&mut self, background: impl Into<BackgroundId>) -> &mut Self {
        self.blend.set_background_enable(Layer::Top, background);
        self
    }

    /// Enables object blending.
    ///
    /// This will only work for objects which have a
    /// [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending` and will
    /// cause any object with that graphics mode to blend towards white / black (depending on
    /// whether this is lighten or darken).
    ///
    /// This interacts with object transparency in a strange way. If object transparency is also
    /// enabled as part of this blend, then the parts of the sprite which overlap with the enabled
    /// backgrounds will show as transparent but anything which doesn't overlap with the background
    /// will be shown as faded towards black or white.
    pub fn enable_object(&mut self) -> &mut Self {
        self.blend.set_object_enable(Layer::Top);
        self
    }

    /// Enables the backdrop for this layer.
    ///
    /// The backdrop is the 0th colour in the palette. It is the colour that is displayed
    /// when there is no background displaying anything at that location.
    pub fn enable_backdrop(&mut self) -> &mut Self {
        self.blend.set_backdrop_enable(Layer::Top);
        self
    }

    /// Configure object transparency in addition to the fading.
    ///
    /// Any part of the object which intersects with the enabled layer will show as transparent by
    /// the given amount, and any which isn't will (if `enable_object()` was called) show
    /// as faded towards black / white.
    pub fn object_transparency(
        &mut self,
        top_fade_amount: Num<u8, 4>,
        bottom_fade_amount: Num<u8, 4>,
    ) -> BlendObjectTransparency<'_> {
        self.blend.set_layer_alpha(Layer::Top, top_fade_amount);
        self.blend
            .set_layer_alpha(Layer::Bottom, bottom_fade_amount);

        BlendObjectTransparency { blend: self.blend }
    }
}

/// Configure the fade effect for making objects optionally transparent.
///
/// Every object with the [`GraphicsMode`](crate::display::object::GraphicsMode) set to `AlphaBlending`
/// will have the same transparency level. Objects will only blend into backgrounds which have
/// been enabled via the [`enable_background`](BlendObjectTransparency::enable_background) method
/// called on them. Otherwise, the object will be rendered as-if there was no blending at all.
pub struct BlendObjectTransparency<'blend> {
    blend: &'blend mut Blend,
}

impl BlendObjectTransparency<'_> {
    /// Enables a given background to have transparent objects on top.
    ///
    /// The [`BackgroundId`] will likely come from a call to `show()`
    /// from either [regular backgrounds](crate::display::tiled::RegularBackground::show) or
    /// [affine backgrounds](crate::display::tiled::AffineBackground::show).
    pub fn enable_background(&mut self, background: impl Into<BackgroundId>) -> &mut Self {
        self.blend.set_background_enable(Layer::Bottom, background);
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Gba,
        display::{
            AffineMatrix, Priority, WinIn,
            object::{AffineMatrixObject, AffineMode, GraphicsMode, Object, ObjectAffine},
            tiled::{
                AffineBackground, AffineBackgroundSize, AffineBackgroundWrapBehaviour,
                RegularBackground, RegularBackgroundSize, VRAM_MANAGER,
            },
        },
        fixnum::{Num, Rect, num, vec2},
        include_aseprite, include_background_gfx,
        test_runner::assert_image_output,
    };

    include_background_gfx!(crate, mod background,
        LOGO => deduplicate "gfx/test_logo.aseprite",
        LOGO_256 => 256 "gfx/test_logo.aseprite",
    );

    include_aseprite!(
        mod sprites,
        "examples/gfx/crab.aseprite",
    );

    #[test_case]
    fn can_blend_to_white(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        frame.blend().brighten(num!(0.5)).enable_background(bg_id);

        frame.commit();

        assert_image_output("gfx/test_output/blend/regular_to_white.png");
    }

    #[test_case]
    fn can_blend_to_white_in_window(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        frame.blend().brighten(num!(0.5)).enable_background(bg_id);
        frame
            .windows()
            .win_in(WinIn::Win0)
            .enable_background(bg_id)
            .enable_blending()
            .set_pos(Rect::new(vec2(20, 20), vec2(100, 100)));

        frame.windows.win_out().enable_background(bg_id);

        frame.commit();

        assert_image_output("gfx/test_output/blend/regular_to_white_in_window.png");
    }

    #[test_case]
    fn can_blend_two_layers_into_each_other(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        let bg1_id = bg.show(&mut frame);

        bg.set_scroll_pos((40, 40));
        let bg2_id = bg.show(&mut frame);

        frame
            .blend()
            .alpha(num!(0.8), num!(0.2))
            .enable_background(Layer::Top, bg1_id)
            .enable_background(Layer::Bottom, bg2_id);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_two_layers_into_each_other.png");
    }

    #[test_case]
    fn can_blend_affine_backgrounds(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = AffineBackground::new(
            Priority::P0,
            AffineBackgroundSize::Background32x32,
            AffineBackgroundWrapBehaviour::Wrap,
        );

        for i in 0..32 {
            for j in 0..32 {
                bg.set_tile(
                    (i, j),
                    &background::LOGO_256.tiles,
                    3 * 30 + 3 + (i + j) as u16 % 5,
                );
            }
        }

        bg.set_transform(AffineMatrix::<Num<i32, 8>>::from_rotation::<8>(num!(0.125)));

        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        frame.blend().darken(num!(0.5)).enable_background(bg_id);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_affine_darken.png");
    }

    #[test_case]
    fn can_blend_objects_to_create_transparency_effects(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        frame
            .blend()
            .object_transparency(num!(0.5), num!(0.5))
            .enable_background(bg_id);

        Object::new(sprites::IDLE.sprite(0))
            .set_pos((100, 100))
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_object_transparency.png");
    }

    #[test_case]
    fn can_blend_object_to_white(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.blend().brighten(num!(0.5)).enable_object();

        Object::new(sprites::IDLE.sprite(0))
            .set_pos((100, 100))
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_object_lighten.png");
    }

    #[test_case]
    fn can_blend_object_to_black(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.blend().darken(num!(0.75)).enable_object();

        Object::new(sprites::IDLE.sprite(0))
            .set_pos((100, 100))
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_object_darken.png");
    }

    #[test_case]
    fn can_blend_object_shape_to_black(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        frame.blend().darken(num!(0.75)).enable_background(bg_id);
        frame
            .windows()
            .win_obj()
            .enable_blending()
            .enable_background(bg_id);
        frame.windows().win_out().enable_background(bg_id);

        Object::new(sprites::IDLE.sprite(0))
            .set_pos((100, 100))
            .set_graphics_mode(GraphicsMode::Window)
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_object_darken_window.png");
    }

    #[test_case]
    fn can_blend_affine_object_to_black(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);

        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.blend().darken(num!(0.75)).enable_object();

        let matrix =
            AffineMatrixObject::from(AffineMatrix::<Num<i32, 8>>::from_rotation::<8>(num!(0.125)));

        ObjectAffine::new(sprites::IDLE.sprite(0), matrix, AffineMode::AffineDouble)
            .set_pos((100, 100))
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_object_affine_darken.png");
    }

    #[test_case]
    fn can_fade_and_have_object_transparency(gba: &mut Gba) {
        VRAM_MANAGER.set_background_palettes(background::PALETTES);
        let mut gfx = gba.graphics.get();

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            background::LOGO.tiles.format(),
        );

        bg.fill_with(&background::LOGO);
        bg.set_scroll_pos((-116, -116));

        let mut frame = gfx.frame();
        let bg_id = bg.show(&mut frame);

        frame
            .blend()
            .darken(num!(0.75))
            .enable_object()
            .object_transparency(num!(0.5), num!(0.5))
            .enable_background(bg_id);

        Object::new(sprites::IDLE.sprite(0))
            .set_pos((100, 100))
            .set_graphics_mode(GraphicsMode::AlphaBlending)
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/blend/blend_object_fade_transparent.png");
    }
}
