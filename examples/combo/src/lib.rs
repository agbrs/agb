#![no_std]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

extern crate alloc;
use alloc::boxed::Box;

use agb::{
    display::{
        tiled::{InfiniteScrolledMap, RegularBackgroundSize, TileFormat, TileSet},
        Priority,
    },
    fixnum::{Num, Vector2D},
    include_background_gfx,
    input::Button,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Game {
    TheHatChoosesTheWizard,
    ThePurpleNight,
    HyperspaceRoll,
    TheDungeonPuzzlersLament,
    Amplitude,
}

impl Game {
    fn launch_game(self, gba: agb::Gba) -> ! {
        match self {
            Game::TheHatChoosesTheWizard => the_hat_chooses_the_wizard::main(gba),
            Game::ThePurpleNight => the_purple_night::main(gba),
            Game::HyperspaceRoll => hyperspace_roll::main(gba),
            Game::TheDungeonPuzzlersLament => the_dungeon_puzzlers_lament::entry(gba),
            Game::Amplitude => amplitude::main(gba),
        }
    }

    fn from_index(index: i32) -> Game {
        match index.rem_euclid(4) {
            0 => Game::TheHatChoosesTheWizard,
            1 => Game::ThePurpleNight,
            2 => Game::HyperspaceRoll,
            3 => Game::TheDungeonPuzzlersLament,
            4 => Game::Amplitude,
            _ => unreachable!("game out of index in an unreachable manner"),
        }
    }
}

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

    let hat = TileSet::new(games::hat.tiles, TileFormat::EightBpp);
    let purple = TileSet::new(games::purple.tiles, TileFormat::EightBpp);
    let hyperspace = TileSet::new(games::hyperspace.tiles, TileFormat::EightBpp);
    let dungeon_puzzler = TileSet::new(games::dungeon_puzzler.tiles, TileFormat::EightBpp);
    let amplitude = TileSet::new(games::amplitude.tiles, TileFormat::EightBpp);

    let tiles = [hat, purple, hyperspace, dungeon_puzzler, amplitude];

    let tile_settings = &[
        games::hat.tile_settings,
        games::purple.tile_settings,
        games::hyperspace.tile_settings,
        games::dungeon_puzzler.tile_settings,
        games::amplitude.tile_settings,
    ];

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

            let game = (pos.x).rem_euclid(tiles.len() as i32 * 30) as usize / 30;
            let tile_id = (y * 30 + x) as usize;
            (&tiles[game], tile_settings[game][tile_id])
        }),
    );

    bg.init(&mut vram, (0, 0).into(), &mut || {});

    bg.set_pos(&mut vram, (0, 0).into());
    bg.commit(&mut vram);
    bg.show();

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
            break Game::from_index(game_idx);
        }
    };

    bg.hide();
    bg.clear(&mut vram);
    bg.commit(&mut vram);

    game
}

pub fn main(mut gba: agb::Gba) -> ! {
    get_game(&mut gba).launch_game(gba)
}
