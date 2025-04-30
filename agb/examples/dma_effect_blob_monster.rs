#![no_main]
#![no_std]

use agb::{
    display::{
        GraphicsFrame, HEIGHT, Priority, WIDTH,
        tiled::{
            RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileSetting, VRAM_MANAGER,
        },
    },
    dma::HBlankDmaDefinition,
    fixnum::{Num, Vector2D, num, vec2},
    include_background_gfx,
};

use alloc::{vec, vec::Vec};

extern crate alloc;

include_background_gfx!(mod backgrounds, "639bff",
    MONSTER => "examples/gfx/blob-monster-tiles.aseprite",
);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    let mut gfx = gba.graphics.get();
    let mut monster = BlobMonster::new();

    loop {
        monster.update();

        let mut frame = gfx.frame();

        monster.show(&mut frame);

        frame.commit();
    }
}

struct BlobMonster {
    widths: Vec<Num<i32, 12>>,
    mouth_offset: Vec<Num<i32, 12>>,
    position: Vector2D<i32>,
    bg: RegularBackgroundTiles,

    frame: i32,
}

impl BlobMonster {
    const BLOB_MONSTER_HEIGHT: i32 = 90;

    const BLOB_MONSTER_MOUTH_HEIGHT: i32 = 40;
    const BLOB_MONSTER_MOUTH_SIZE: i32 = 20;

    pub fn new() -> Self {
        let blob_monster_widths = (0..Self::BLOB_MONSTER_HEIGHT)
            .map(|y| (Num::new(y).sqrt() * 10).sqrt() * 10)
            .collect();

        let mouth_offset = vec![Num::new(0); 160];

        Self {
            widths: blob_monster_widths,
            mouth_offset,
            bg: blob_monster_background(),
            position: vec2(WIDTH / 2, HEIGHT / 2),

            frame: 0,
        }
    }

    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);

        let frame_frac = Num::<_, 12>::new(self.frame) / 128;

        self.position = vec2(2 * WIDTH / 3, HEIGHT / 2)
            + (vec2(frame_frac.sin(), frame_frac.cos()) * 10).trunc();

        let mouth_depth = num!(15) - frame_frac.cos() * 5;

        self.mouth_offset = (0..Self::BLOB_MONSTER_HEIGHT)
            .map(|y| {
                if y < Self::BLOB_MONSTER_MOUTH_HEIGHT - Self::BLOB_MONSTER_MOUTH_SIZE / 2 {
                    num!(0)
                } else if y < Self::BLOB_MONSTER_MOUTH_HEIGHT + Self::BLOB_MONSTER_MOUTH_SIZE / 2 {
                    let y = Num::new(
                        y - Self::BLOB_MONSTER_MOUTH_HEIGHT + Self::BLOB_MONSTER_MOUTH_SIZE / 2,
                    );

                    (num!(1) - (y / Self::BLOB_MONSTER_MOUTH_SIZE).cos()) * mouth_depth
                } else {
                    num!(0)
                }
            })
            .collect();
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        let width_starts_pairs = (0..160)
            .map(|y| {
                if y < self.position.y {
                    vec2(0, 0)
                } else if y < self.position.y + Self::BLOB_MONSTER_HEIGHT {
                    let monster_y = y - self.position.y;
                    let mut width = self.widths[monster_y as usize];
                    let lean = Num::new(monster_y) / 20;
                    let mut x = Num::new(self.position.x) - width / 2 + lean;

                    let mouth_width = self.mouth_offset[monster_y as usize];
                    x += mouth_width;
                    width -= mouth_width;

                    vec2(-x.trunc() as u16, width.trunc() as u16)
                } else {
                    let max_width = self.widths[Self::BLOB_MONSTER_HEIGHT as usize - 1];

                    let x = Num::new(self.position.x) - max_width / 2;
                    vec2(-x.trunc() as u16, max_width.trunc() as u16)
                }
            })
            .enumerate()
            .map(|(y, pos)| {
                // we have to subtract the y position because the gba takes this into account when rendering
                vec2(pos.x, pos.y.wrapping_sub(y as u16))
            })
            .collect::<Vec<_>>();

        let bg_id = self.bg.show(frame);

        HBlankDmaDefinition::new(bg_id.scroll_dma(), &width_starts_pairs).show(frame);
    }
}

fn blob_monster_background() -> RegularBackgroundTiles {
    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        backgrounds::MONSTER.tiles.format(),
    );

    let tileset = &backgrounds::MONSTER.tiles;
    fn tid(id: usize) -> TileSetting {
        TileSetting::new(id as u16, TileEffect::default())
    }

    bg.set_tile((0, 0), tileset, tid(0));
    bg.set_tile((0, 1), tileset, tid(2));
    bg.set_tile((1, 1), tileset, tid(3));

    for y in 2..32 {
        bg.set_tile((0, y), tileset, tid(4));
        bg.set_tile((y - 1, y), tileset, tid(5));
        bg.set_tile((y, y), tileset, tid(3));

        for x in 1..(y - 1) {
            bg.set_tile((x, y), tileset, tid(1));
        }
    }

    bg
}
