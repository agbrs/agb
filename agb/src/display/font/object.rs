use agb_fixnum::Vector2D;

use crate::display::object::{DynamicSprite16, Object, PaletteVramSingle, Size};

use super::LetterGroup;

/// The sprite based render backend for [`LetterGroup`]s. Takes a
/// [`LetterGroup`] and gives an [`Object`] containing the text. A simple use of
/// the renderer is
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// extern crate alloc;
/// use alloc::vec::Vec;
/// use agb::display::{
///     Palette16, Rgb15,
///     font::{AlignmentKind, Font, Layout, ObjectTextRenderer},
///     object::Size,
/// };
///
/// static SIMPLE_PALETTE: &Palette16 = {
///     let mut palette = [Rgb15::BLACK; 16];
///     palette[1] = Rgb15::WHITE;
///     &Palette16::new(palette)
/// };
/// static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
///
/// # #[agb::doctest]
/// # fn test(mut gba: agb::Gba) {
/// let mut text_elements = Vec::new();
///
/// // the actual text rendering
///
/// let layout = Layout::new("Hello, world!", &FONT, AlignmentKind::Left, 16, 200);
/// let text_renderer = ObjectTextRenderer::new(SIMPLE_PALETTE.into(), Size::S16x16);
///
/// for letter_group in layout {
///     text_elements.push(text_renderer.show(&letter_group, (0, 0)));
/// }
///
/// // display the objects in the usual means
///
/// let mut gfx = gba.graphics.get();
/// let mut frame = gfx.frame();
///
/// for obj in text_elements.iter() {
///     obj.show(&mut frame);
/// }
/// # }
/// ```
pub struct ObjectTextRenderer {
    palette: PaletteVramSingle,
    size: Size,
}

impl ObjectTextRenderer {
    #[must_use]
    /// Creates a [`ObjectTextRenderer`]. The palette is the palette that will
    /// be used by each [`Object`] returned by [`ObjectTextRenderer::show`]. The
    /// [`Size`] is the size of each sprite used by each [`Object`], the
    /// [`Size`] should be larger than the letter group size given to the
    /// [`Layout`][super::Layout].
    pub fn new(palette: PaletteVramSingle, size: Size) -> Self {
        Self { palette, size }
    }

    #[must_use]
    /// Generates an object that represents the given [`LetterGroup`]. The
    /// position of the text can be adjusted using the offset parameter.
    pub fn show(&self, group: &LetterGroup, offset: impl Into<Vector2D<i32>>) -> Object {
        let offset = offset.into();
        let mut sprite = DynamicSprite16::new(self.size);
        let pal_index = group.palette_index();

        for pixel in group.pixels() {
            sprite.set_pixel(pixel.x as usize, pixel.y as usize, pal_index as usize);
        }

        let mut object = Object::new(sprite.to_vram(self.palette.clone()));
        object.set_pos(offset + group.position());

        object
    }
}

#[cfg(test)]
mod tests {
    use agb_fixnum::vec2;
    use alloc::{format, vec::Vec};

    use crate::{
        display::{
            Rgb, Rgb15,
            font::{AlignmentKind, ChangeColour, Font, Layout},
            palette16::Palette16,
            tiled::VRAM_MANAGER,
        },
        test_runner::assert_image_output,
    };
    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    use super::*;

    #[test_case]
    fn check_font_rendering_simple(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();

        VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb::new(0xff, 0, 0xff).to_rgb15());

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        const CHANGE1: ChangeColour = ChangeColour::new(1);
        const CHANGE2: ChangeColour = ChangeColour::new(2);

        let layout = Layout::new(
            &format!("Hello, world! {CHANGE2}This is in red{CHANGE1} and back to white"),
            &FONT,
            AlignmentKind::Left,
            16,
            200,
        );
        let text_render = ObjectTextRenderer::new((&PALETTE).into(), Size::S16x16);

        let objects: Vec<_> = layout.map(|x| text_render.show(&x, vec2(16, 16))).collect();

        let mut frame = gfx.frame();

        for object in objects.iter() {
            object.show(&mut frame);
        }

        frame.commit();

        assert_image_output("gfx/test_output/sprite_font_render_simple.png");
    }

    #[test_case]
    fn check_japanese_rendering(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();

        VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb::new(0xff, 0, 0xff).to_rgb15());

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        let layout = Layout::new(
            "現代社会において、情報技術の進化は目覚ましい。それは、私たちの生活様式だけでなく、思考様式にも大きな影響を与えている。例えば、スマートフォンやタブレット端末の普及により、いつでもどこでも情報にアクセスできるようになった。これにより、知識の共有やコミュニケーションが容易になり、新しい文化や価値観が生まれている。しかし、一方で、情報過多やプライバシーの問題など、新たな課題も浮上している。私たちは、これらの課題にどのように向き合い、情報技術をどのように活用していくべきだろうか。それは、私たち一人ひとりが真剣に考えるべき重要なテーマである。",
            &FONT,
            AlignmentKind::Left,
            32,
            200,
        );
        let text_render = ObjectTextRenderer::new((&PALETTE).into(), Size::S32x16);

        let objects: Vec<_> = layout.map(|x| text_render.show(&x, vec2(16, 16))).collect();

        let mut frame = gfx.frame();

        for object in objects.iter() {
            object.show(&mut frame);
        }

        frame.commit();

        assert_image_output("gfx/test_output/sprite_font_render_japanese.png");
    }
}
