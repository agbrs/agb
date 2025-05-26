// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        GraphicsFrame, Priority,
        tiled::{
            InfiniteScrolledMap, RegularBackground, RegularBackgroundSize, TileFormat, TileSetting,
            VRAM_MANAGER,
        },
    },
    fixnum::{Num, Rect, Vector2D, vec2},
    include_background_gfx,
};

extern crate alloc;

impl Level {
    fn collides(&self, tile: Vector2D<i32>) -> bool {
        if tile.x < 0 || tile.x > self.width as i32 || tile.y < 0 || tile.y > self.height as i32 {
            return false;
        }

        let idx = (tile.x + tile.y * self.width as i32) as usize;

        self.collision_map[idx / 8] & (1 << (idx % 8)) != 0
    }
}

impl Level {
    fn bounds(&self) -> Rect<i32> {
        Rect::new(
            vec2(0, 0),
            vec2(self.width as i32 - 1, self.height as i32 - 1),
        )
    }
}

include_background_gfx!(mod tiles, "2ce8f4", TILES => "gfx/tilesheet.png");

struct Level {
    width: u32,
    height: u32,
    background: &'static [TileSetting],
    collision_map: &'static [u8],
    winning_map: &'static [u8],
    player_start: (i32, i32),
}

mod levels {
    use super::Level;
    use agb::display::tiled::TileSetting;
    static TILES: &[TileSetting] = super::tiles::TILES.tile_settings;

    include!(concat!(env!("OUT_DIR"), "/levels.rs"));
}

// define a common set of number and vector type to use throughout
type Number = Num<i32, 8>;
type Vector = Vector2D<Number>;

struct Player {
    position: Vector,
    velocity: Vector,
}

impl Player {
    // fn rect(&self) -> Rect<Number> {
    //     Rect::new(self.position - Self::SIZE / 2, Self::SIZE)
    // }
}

struct World {
    level: &'static Level,
    bg: InfiniteScrolledMap,
}

impl World {
    fn new(level: &'static Level) -> Self {
        let bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let bg = InfiniteScrolledMap::new(bg);

        World { level, bg }
    }

    fn set_pos(&mut self, pos: Vector2D<i32>) {
        self.bg.set_scroll_pos(pos, |pos| {
            let tile = if self.level.bounds().contains_point(pos) {
                self.level.background[pos.x as usize + pos.y as usize * self.level.width as usize]
            } else {
                TileSetting::BLANK
            };

            (&tiles::TILES.tiles, tile)
        });
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
    }
}

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);
    let mut bg = World::new(levels::LEVELS[0]);

    loop {
        bg.set_pos(vec2(0, 0));

        let mut frame = gfx.frame();

        bg.show(&mut frame);

        frame.commit();
    }
}

#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}
