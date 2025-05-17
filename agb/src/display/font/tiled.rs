use alloc::vec::Vec;

use super::LetterGroup;
use crate::{
    display::tiled::{DynamicTile16, RegularBackground, TileEffect},
    fixnum::{Vector2D, vec2},
};

/// The background tile based renderer backend for [`LetterGroup`]s. A simple
/// use of the renderer is
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// # core::include!("../../doctest_runner.rs");
/// use agb::display::{
///     Palette16, Rgb15, Priority,
///     font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
///     tiled::{RegularBackground, VRAM_MANAGER, RegularBackgroundSize, TileFormat},
/// };
///
/// static SIMPLE_PALETTE: &Palette16 = {
///     let mut palette = [Rgb15::BLACK; 16];
///     palette[1] = Rgb15::WHITE;
///     &Palette16::new(palette)
/// };
/// static FONT: Font = agb::include_font!("examples/font/pixelated.ttf", 8);
///
/// # fn test(mut gba: agb::Gba) {
/// VRAM_MANAGER.set_background_palette(0, SIMPLE_PALETTE);
/// let mut bg = RegularBackground::new(
///     Priority::P0,
///     RegularBackgroundSize::Background32x32,
///     TileFormat::FourBpp,
/// );
///
/// // the actual text rendering
///
/// let layout = Layout::new("Hello, world!", &FONT, AlignmentKind::Left, 40, 200);
/// let mut text_renderer = RegularBackgroundTextRenderer::new((0, 0));
///
/// for letter_group in layout {
///     text_renderer.show(&mut bg, &letter_group);
/// }
///
/// // display the background in the usual means
///
/// let mut gfx = gba.graphics.get();
/// let mut frame = gfx.frame();
///
/// bg.show(&mut frame);
/// # }
/// ```
pub struct RegularBackgroundTextRenderer {
    tiles: Vec<Vec<Option<DynamicTile16>>>,
    origin: Vector2D<i32>,
}

impl RegularBackgroundTextRenderer {
    /// Creates a new background renderer with a given origin. All text is
    /// rendered with respect to this origin.
    pub fn new(origin: impl Into<Vector2D<i32>>) -> Self {
        Self {
            tiles: Vec::new(),
            origin: origin.into(),
        }
    }

    /// Displays the given letter group on the given background.
    pub fn show(&mut self, bg: &mut RegularBackground, group: &LetterGroup) {
        self.ensure_drawing_space(bg, group);

        let dynamic_origin = vec2(self.origin.x.rem_euclid(8), self.origin.y.rem_euclid(8));

        for (px_start, px) in group.pixels_packed() {
            let pos = px_start + dynamic_origin + group.position();

            let x = pos.x as usize / 8;
            let y = pos.y as usize / 8;

            let row = &mut self.tiles[y];

            let x_in_tile = pos.x.rem_euclid(8) * 4;

            let tile_left = row[x].as_mut().expect("should have ensured space");
            tile_left.data()[pos.y.rem_euclid(8) as usize] |= px << x_in_tile;

            if x_in_tile > 0 {
                let tile_right = row[x + 1].as_mut().expect("should have ensured space");
                tile_right.data()[pos.y.rem_euclid(8) as usize] |= px >> (32 - x_in_tile);
            }
        }
    }

    fn ensure_drawing_space(&mut self, bg: &mut RegularBackground, group: &LetterGroup) {
        let dynamic_origin = vec2(self.origin.x.rem_euclid(8), self.origin.y.rem_euclid(8));
        let tile_offset = vec2(self.origin.x / 8, self.origin.y / 8);

        let bounds = group.bounds();
        let top_left_tile = group.position() / 8;

        let bottom_right_tile = (dynamic_origin + bounds + group.position()) / 8 + vec2(1, 0);
        if self.tiles.len() <= bottom_right_tile.y as usize {
            self.tiles
                .resize_with(bottom_right_tile.y as usize + 1, Vec::new);
        }

        for row_idx in top_left_tile.y..(bottom_right_tile.y + 1) {
            let row = &mut self.tiles[row_idx as usize];
            if row.len() <= bottom_right_tile.x as usize {
                row.resize_with(bottom_right_tile.x as usize + 1, || None);
            }

            for column_idx in top_left_tile.x..(bottom_right_tile.x + 1) {
                if row[column_idx as usize].is_none() {
                    let tile_pos = vec2(column_idx, row_idx) + tile_offset;
                    let tile = DynamicTile16::new().fill_with(0);
                    bg.set_tile_dynamic16(tile_pos, &tile, TileEffect::default());

                    row[column_idx as usize] = Some(tile);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Gba,
        display::{
            Priority, Rgb15,
            font::{AlignmentKind, ChangeColour, Font, Layout},
            palette16::Palette16,
            tiled::{RegularBackgroundSize, TileFormat, VRAM_MANAGER},
        },
        test_runner::assert_image_output,
        timer::Divider,
    };

    use alloc::format;

    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    #[test_case]
    fn background_text_render_english(gba: &mut Gba) {
        let mut gfx = gba.graphics.get();

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        VRAM_MANAGER.set_background_palette(0, &PALETTE);

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );

        const CHANGE1: ChangeColour = ChangeColour::new(1);
        const CHANGE2: ChangeColour = ChangeColour::new(2);

        let layout = Layout::new(
            &format!("Hello, world! {CHANGE2}This is in red{CHANGE1} and back to white"),
            &FONT,
            AlignmentKind::Left,
            128,
            200,
        );

        let mut bg_text_render = RegularBackgroundTextRenderer::new((20, 20));

        for lg in layout {
            bg_text_render.show(&mut bg, &lg);
        }

        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();

        assert_image_output("gfx/test_output/bg_font_render_simple.png");
    }

    #[test_case]
    fn background_text_render_japanese(gba: &mut Gba) {
        let mut gfx = gba.graphics.get();

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        VRAM_MANAGER.set_background_palette(0, &PALETTE);

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x64,
            TileFormat::FourBpp,
        );

        let layout = Layout::new(
            "現代社会において、情報技術の進化は目覚ましい。それは、私たちの生活様式だけでなく、思考様式にも大きな影響を与えている。例えば、スマートフォンやタブレット端末の普及により、いつでもどこでも情報にアクセスできるようになった。これにより、知識の共有やコミュニケーションが容易になり、新しい文化や価値観が生まれている。しかし、一方で、情報過多やプライバシーの問題など、新たな課題も浮上している。私たちは、これらの課題にどのように向き合い、情報技術をどのように活用していくべきだろうか。それは、私たち一人ひとりが真剣に考えるべき重要なテーマである。",
            &FONT,
            AlignmentKind::Left,
            16,
            200,
        );
        let mut bg_text_render = RegularBackgroundTextRenderer::new((20, 20));

        for lg in layout {
            bg_text_render.show(&mut bg, &lg);
        }

        let mut frame = gfx.frame();
        bg.show(&mut frame);
        frame.commit();

        assert_image_output("gfx/test_output/bg_font_render_japanese.png");
    }

    #[test_case]
    fn background_text_single_group(gba: &mut Gba) {
        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        VRAM_MANAGER.set_background_palette(0, &PALETTE);

        let mut bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x64,
            TileFormat::FourBpp,
        );

        let mut layout = Layout::new(
            "現代社会において、情報技術の進化は目覚ましい。それは、私たちの生活様式だけでなく、思考様式にも大きな影響を与えている。例えば、スマートフォンやタブレット端末の普及により、いつでもどこでも情報にアクセスできるようになった。これにより、知識の共有やコミュニケーションが容易になり、新しい文化や価値観が生まれている。しかし、一方で、情報過多やプライバシーの問題など、新たな課題も浮上している。私たちは、これらの課題にどのように向き合い、情報技術をどのように活用していくべきだろうか。それは、私たち一人ひとりが真剣に考えるべき重要なテーマである。",
            &FONT,
            AlignmentKind::Left,
            32,
            200,
        );
        let mut bg_text_render = RegularBackgroundTextRenderer::new((20, 20));
        let letter_group = layout.next().unwrap();

        let mut timer = gba.timers.timers().timer2;
        timer
            .set_divider(Divider::Divider256)
            .set_overflow_amount(u16::MAX)
            .set_cascade(false)
            .set_enabled(true);

        let before_show = timer.value();
        bg_text_render.show(&mut bg, &core::hint::black_box(letter_group));
        let after_show = timer.value();

        let total = u32::from(after_show.wrapping_sub(before_show)) * 256;

        crate::println!("rendering time: {total}");
    }
}
