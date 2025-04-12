use agb_fixnum::Vector2D;

use crate::display::object::{DynamicSprite, Object, PaletteVramSingle, Size};

use super::LetterGroup;

pub struct SpriteTextRenderer {
    palette: PaletteVramSingle,
    size: Size,
}

impl SpriteTextRenderer {
    #[must_use]
    pub fn new(palette: PaletteVramSingle, size: Size) -> Self {
        Self { palette, size }
    }

    #[must_use]
    pub fn show(&self, group: &LetterGroup, offset: impl Into<Vector2D<i32>>) -> Object {
        let offset = offset.into();
        let mut sprite = DynamicSprite::new(self.size);
        let pal_index = group.palette_index();

        for pixel in group.pixels() {
            sprite.set_pixel(pixel.x as usize, pixel.y as usize, pal_index as usize);
        }

        let mut object = Object::new(sprite.to_vram(self.palette.clone()));
        object.set_position(offset + group.position());

        object
    }
}

#[cfg(test)]
mod tests {
    use agb_fixnum::vec2;
    use alloc::{format, vec::Vec};

    use crate::{
        display::{
            Rgb15,
            font::{AlignmentKind, ChangeColour, Font, Layout},
            palette16::Palette16,
        },
        test_runner::assert_image_output,
    };
    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    use super::*;

    #[test_case]
    fn check_font_rendering_simple(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();

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
        let sprite_text_render = SpriteTextRenderer::new((&PALETTE).into(), Size::S16x16);

        let objects: Vec<_> = layout
            .map(|x| sprite_text_render.show(&x, vec2(16, 16)))
            .collect();

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
        let sprite_text_render = SpriteTextRenderer::new((&PALETTE).into(), Size::S32x16);

        let objects: Vec<_> = layout
            .map(|x| sprite_text_render.show(&x, vec2(16, 16)))
            .collect();

        let mut frame = gfx.frame();

        for object in objects.iter() {
            object.show(&mut frame);
        }

        frame.commit();

        assert_image_output("gfx/test_output/sprite_font_render_japanese.png");
    }
}
