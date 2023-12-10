#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;
use alloc::boxed::Box;

use agb::{
    display::{
        tile_data::TileData,
        tiled::{InfiniteScrolledMap, RegularBackgroundSize, TileFormat},
        Priority,
    },
    fixnum::{Num, Vector2D},
    include_background_gfx,
    input::Button,
};

type Game = fn(agb::Gba) -> !;

struct GameWithTiles {
    game: fn(agb::Gba) -> !,
    tiles: &'static TileData,
}

impl GameWithTiles {
    const fn new(tiles: &'static TileData, game: fn(agb::Gba) -> !) -> Self {
        GameWithTiles { game, tiles }
    }
}

static GAMES: &[GameWithTiles] = &[
    GameWithTiles::new(&games::hat, the_hat_chooses_the_wizard::main),
    GameWithTiles::new(&games::purple, the_purple_night::main),
    GameWithTiles::new(&games::hyperspace, hyperspace_roll::main),
    GameWithTiles::new(&games::dungeon_puzzler, the_dungeon_puzzlers_lament::entry),
    GameWithTiles::new(&games::amplitude, amplitude::main),
];

include_background_gfx!(
    games, "121105",
    hat => 256 deduplicate "gfx/hat.png",
    purple => 256 deduplicate "gfx/purple.png",
    hyperspace => 256 deduplicate "gfx/hyperspace.png",
    dungeon_puzzler => 256 deduplicate "gfx/dungeon_puzzler.png",
    amplitude => 256 deduplicate "gfx/amplitude.png",
);

fn get_game(gba: &mut agb::Gba) -> Game {
    let mut input = agb::input::ButtonController::new();
    let vblank = agb::interrupt::VBlank::get();

    let (tile, mut vram) = gba.display.video.tiled0();

    vram.set_background_palettes(games::PALETTES);

    let mut bg = InfiniteScrolledMap::new(
        tile.background(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::EightBpp,
        ),
        Box::new(|pos| {
            let y = pos.y.rem_euclid(20);
            let x = pos.x.rem_euclid(30);

            let game = (pos.x).rem_euclid(GAMES.len() as i32 * 30) as usize / 30;
            let tile_id = (y * 30 + x) as usize;
            (
                &GAMES[game].tiles.tiles,
                GAMES[game].tiles.tile_settings[tile_id],
            )
        }),
    );

    bg.init(&mut vram, (0, 0).into(), &mut || {});

    bg.set_pos(&mut vram, (0, 0).into());
    bg.commit(&mut vram);
    bg.set_visible(true);

    let mut position: Vector2D<Num<i32, 8>> = (0, 0).into();
    let mut game_idx = 0;
    let game = loop {
        let lr: agb::input::Tri = (
            input.is_just_pressed(Button::LEFT),
            input.is_just_pressed(Button::RIGHT),
        )
            .into();

        game_idx += lr as i32;

        if (position.x - game_idx * 30 * 8).abs() < Num::new(1) / 2 {
            position.x = Num::new(game_idx * 30 * 8);
        }

        position.x +=
            ((Num::new(game_idx * 30 * 8) - position.x) / 8).clamp(-Num::new(8), Num::new(8));

        bg.set_pos(&mut vram, position.floor());

        vblank.wait_for_vblank();
        bg.commit(&mut vram);
        input.update();

        if input.is_just_pressed(Button::A) {
            break GAMES[game_idx.rem_euclid(GAMES.len() as i32) as usize].game;
        }
    };

    bg.set_visible(false);
    bg.clear(&mut vram);
    bg.commit(&mut vram);

    game
}

pub fn main(mut gba: agb::Gba) -> ! {
    get_game(&mut gba)(gba)
}
