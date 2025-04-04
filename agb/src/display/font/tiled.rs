use alloc::vec::Vec;

use super::LetterGroup;
use crate::{
    display::tiled::{DynamicTile, RegularBackgroundTiles, TileEffect},
    fixnum::{Vector2D, vec2},
};

pub struct RegularBackgroundTextRenderer {
    tiles: Vec<Vec<Option<DynamicTile>>>,
    origin: Vector2D<i32>,
}

impl RegularBackgroundTextRenderer {
    pub fn new(origin: impl Into<Vector2D<i32>>) -> Self {
        Self {
            tiles: Vec::new(),
            origin: origin.into(),
        }
    }

    pub fn show(&mut self, bg: &mut RegularBackgroundTiles, group: &LetterGroup) {
        let dynamic_origin = vec2(self.origin.x % 8, self.origin.y % 8);

        for pixel in group.pixels() {
            self.put_pixel(
                dynamic_origin + pixel + group.position(),
                group.palette_index(),
            );
        }

        let tile_offset = vec2(self.origin.x / 8, self.origin.y / 8);
        for (y, row) in self.tiles.iter().enumerate() {
            for (x, tile) in row.iter().enumerate() {
                let Some(tile) = tile else {
                    continue;
                };

                let tile_pos = vec2(x as i32, y as i32);

                bg.set_tile_dynamic(tile_pos + tile_offset, tile, TileEffect::default());
            }
        }
    }

    fn put_pixel(&mut self, pos: Vector2D<i32>, palette_index: u8) {
        let x = pos.x as usize / 8;
        let y = pos.y as usize / 8;

        if self.tiles.len() <= y {
            self.tiles.resize_with(y + 1, Vec::new);
        }

        let row = &mut self.tiles[y];
        if row.len() <= x {
            row.resize_with(x + 1, || None);
        }

        let tile = row[x].get_or_insert_with(|| DynamicTile::new().fill_with(0));

        let inner_x = (pos.x as usize).rem_euclid(8);
        let inner_y = (pos.y as usize).rem_euclid(8);

        tile.set_pixel(inner_x, inner_y, palette_index);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        Gba,
        display::{
            Priority,
            font::{AlignmentKind, ChangeColour, Font, Layout},
            palette16::Palette16,
            tiled::{RegularBackgroundSize, TileFormat, VRAM_MANAGER},
        },
        test_runner::assert_image_output,
    };

    use alloc::format;

    static FONT: Font = include_font!("fnt/ark-pixel-10px-proportional-latin.ttf", 10);

    #[test_case]
    fn background_text_render_english(gba: &mut Gba) {
        let mut gfx = gba.display.graphics.get();

        static PALETTE: Palette16 = const {
            let mut palette = [0x0; 16];
            palette[1] = 0xFF_FF;
            palette[2] = 0x10_7C;

            Palette16::new(palette)
        };

        VRAM_MANAGER.set_background_palette(0, &PALETTE);

        let mut bg = RegularBackgroundTiles::new(
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

        assert_image_output("gfx/test_output/bg_font_render_simple.png");
    }

    #[test_case]
    fn background_text_render_japanese(gba: &mut Gba) {
        let mut gfx = gba.display.graphics.get();

        static PALETTE: Palette16 = const {
            let mut palette = [0x0; 16];
            palette[1] = 0xFF_FF;
            Palette16::new(palette)
        };

        VRAM_MANAGER.set_background_palette(0, &PALETTE);

        let mut bg = RegularBackgroundTiles::new(
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
    fn background_text_single_group(_gba: &mut Gba) {
        static PALETTE: Palette16 = const {
            let mut palette = [0x0; 16];
            palette[1] = 0xFF_FF;
            Palette16::new(palette)
        };

        VRAM_MANAGER.set_background_palette(0, &PALETTE);

        let mut bg = RegularBackgroundTiles::new(
            Priority::P0,
            RegularBackgroundSize::Background32x64,
            TileFormat::FourBpp,
        );

        let mut layout = Layout::new(
            "現代社会において、情報技術の進化は目覚ましい。それは、私たちの生活様式だけでなく、思考様式にも大きな影響を与えている。例えば、スマートフォンやタブレット端末の普及により、いつでもどこでも情報にアクセスできるようになった。これにより、知識の共有やコミュニケーションが容易になり、新しい文化や価値観が生まれている。しかし、一方で、情報過多やプライバシーの問題など、新たな課題も浮上している。私たちは、これらの課題にどのように向き合い、情報技術をどのように活用していくべきだろうか。それは、私たち一人ひとりが真剣に考えるべき重要なテーマである。",
            &FONT,
            AlignmentKind::Left,
            16,
            200,
        );
        let mut bg_text_render = RegularBackgroundTextRenderer::new((20, 20));

        bg_text_render.show(&mut bg, &layout.next().unwrap());
    }
}
