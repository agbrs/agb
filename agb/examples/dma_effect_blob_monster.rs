// This example demonstrates a way to use a combination of a background and an object to show a
// more dynamic object on screen.
//
// The monster is made up of a single regular background and a sprite for the eye. You could
// produce this effect with many large sprites, but with this technique, you could make the outside
// be effected by the player as well e.g. if they attack a certain part of it, it could
// mark an indent in that location. This wouldn't be possible with sprites.
#![no_main]
#![no_std]

use agb::{
    display::{
        GraphicsFrame, HEIGHT, Priority, WIDTH,
        object::Object,
        tiled::{
            RegularBackgroundSize, RegularBackgroundTiles, TileEffect, TileSetting, VRAM_MANAGER,
        },
    },
    dma::HBlankDmaDefinition,
    fixnum::{Num, Vector2D, num, vec2},
    include_aseprite, include_background_gfx,
};

use alloc::{vec, vec::Vec};

extern crate alloc;

include_background_gfx!(mod backgrounds, "639bff",
    MONSTER => "examples/gfx/blob-monster-tiles.aseprite",
);

include_aseprite!(mod sprites, "examples/gfx/monster-features.aseprite");

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
    /// The widths of the individual heights of the blob monster. These are calculated using a
    /// quartic root to create a pleasing shape for the monster.
    widths: Vec<Num<i32, 12>>,
    /// The monster needs a mouth. These are the offsets needed to give some animation while the
    /// monster is breathing.
    mouth_offset: Vec<Num<i32, 12>>,
    /// The current location of the top middle of the blob monster.
    position: Vector2D<i32>,

    /// The background in use. This is a simple triangle pattern, but we use scroll offset DMA
    /// to breathe some life into the monster.
    bg: RegularBackgroundTiles,

    /// The current frame, used for animations
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
        let bg_id = self.bg.show(frame);

        // The eye sprite is a blinking eye animation. It will close and re-open every 128 frames.
        let eye_sprite = sprites::EYE
            .sprites()
            .get((self.frame % 128) as usize)
            .unwrap_or_else(|| sprites::EYE.sprite(0));
        let eye_pos = self.position + vec2(-15, 10);

        Object::new(eye_sprite)
            .set_position(eye_pos)
            .set_priority(Priority::P0)
            .show(frame);

        HBlankDmaDefinition::new(bg_id.scroll_dma(), &self.width_start_pairs()).show(frame);
    }

    fn width_start_pairs(&self) -> Vec<Vector2D<u16>> {
        (0..160)
            .map(|y| {
                if y < self.position.y {
                    // If we're higher than the current monster, return 0, 0 which is a blank line
                    vec2(0, 0)
                } else if y < self.position.y + Self::BLOB_MONSTER_HEIGHT {
                    // Here we're within the blob monster itself
                    let monster_y = y - self.position.y;
                    let mut width = self.widths[monster_y as usize];

                    // We'd like it to lean forward slightly to give it a bit more life
                    let lean = Num::new(monster_y) / 20;
                    let mut x = Num::new(self.position.x) - width / 2 + lean;

                    // Put a dent in the front to represent the monster's mouth
                    let mouth_width = self.mouth_offset[monster_y as usize];
                    x += mouth_width;
                    width -= mouth_width;

                    vec2(-x.trunc() as u16, width.trunc() as u16)
                } else {
                    // Here we're below the monster, so just make it a rectangle from here.
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
            .collect()
    }
}

// The blob monster background is a simple triangle. We use DMA to control the scroll position
// in order to show the blob monster as something more than just a single triangle
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

    // These slightly awkward tile setups are to draw the triangle given the
    // 6 tiles we need for it.
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
