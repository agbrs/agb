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
        let dynamic_origin = vec2(self.origin.x.rem_euclid(8), self.origin.y.rem_euclid(8));

        let bounds = group.bounds();
        let top_left_tile = group.position() / 8;

        let bottom_right_tile = (dynamic_origin + bounds + group.position()) / 8 + vec2(1, 0);
        if self.tiles.len() <= bottom_right_tile.y as usize {
            self.tiles
                .resize_with(bottom_right_tile.y as usize + 1, Vec::new);
        }

        for row in top_left_tile.y..(bottom_right_tile.y + 1) {
            let row = &mut self.tiles[row as usize];
            if row.len() <= bottom_right_tile.x as usize {
                row.resize_with(bottom_right_tile.x as usize + 1, || None);
            }

            for column in top_left_tile.x..(bottom_right_tile.x + 1) {
                row[column as usize].get_or_insert_with(|| DynamicTile::new().fill_with(0));
            }
        }

        for (px_start, px) in group.pixels_packed() {
            let pos = px_start + dynamic_origin + group.position();

            let x = pos.x as usize / 8;
            let y = pos.y as usize / 8;

            let row = unsafe { self.tiles.get_unchecked_mut(y) };

            let x_in_tile = pos.x.rem_euclid(8) * 4;

            let tile_left = unsafe { row.get_unchecked_mut(x).as_mut().unwrap_unchecked() };
            tile_left.tile_data[pos.y.rem_euclid(8) as usize] |= px << x_in_tile;

            if x_in_tile > 0 {
                let tile_right =
                    unsafe { row.get_unchecked_mut(x + 1).as_mut().unwrap_unchecked() };
                tile_right.tile_data[pos.y.rem_euclid(8) as usize] |= px >> (32 - x_in_tile);
            }
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

    // #[test_case]
    // fn check_shifting(_gba: &mut Gba) {
    //     assert_eq!((0xFFFF_FFFF as u32) >> 32, 0);
    // }

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
