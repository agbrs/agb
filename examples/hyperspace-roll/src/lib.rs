// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::display::object::ObjectController;
use agb::display::tiled::{TiledMap, VRamManager};
use agb::display::Priority;
use agb::interrupt::VBlank;
use agb::{display, sound::mixer::Frequency};

extern crate alloc;
use alloc::vec;
use alloc::vec::Vec;

mod background;
mod battle;
mod customise;
mod graphics;
mod level_generation;
mod save;
mod sfx;

use background::{show_title_screen, StarBackground};
use battle::BattleResult;
use graphics::NumberDisplay;
use sfx::Sfx;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Face {
    Shoot,
    Shield,
    Malfunction,
    Heal,
    Bypass,
    DoubleShot,
    TripleShot,
    Blank,
    Disrupt,
    MalfunctionShot,
    DoubleShield,
    TripleShield,
    DoubleShieldValue,
    DoubleShotValue,
    TripleShotValue,
    BurstShield,
    Invert,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Ship {
    Player,
    Drone,
    PilotedShip,
    Shield,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum EnemyAttackType {
    Attack,
    Shield,
    Heal,
}

#[derive(Debug, Clone)]
pub struct Die {
    faces: [Face; 6],
}

impl Die {
    /// roll this die (potentially using the custom probabilities, should we implement that) and return which face index is showing
    fn roll(&self) -> Face {
        let n = agb::rng::gen().rem_euclid(6);
        self.faces[n as usize]
    }
}

#[derive(Debug, Clone)]
pub struct PlayerDice {
    dice: Vec<Die>,
}

struct Agb<'a> {
    obj: ObjectController,
    vblank: VBlank,
    star_background: StarBackground<'a>,
    vram: VRamManager,
    sfx: Sfx<'a>,
}

pub fn main(mut gba: agb::Gba) -> ! {
    save::init_save(&mut gba).expect("Could not initialize save game");

    if save::load_high_score() > 1000 {
        save::save_high_score(&mut gba, 0).expect("Could not reset high score");
    }

    let gfx = gba.display.object.get();
    let vblank = agb::interrupt::VBlank::get();

    let (tiled, mut vram) = gba.display.video.tiled0();
    let mut background0 = tiled.background(
        Priority::P0,
        display::tiled::RegularBackgroundSize::Background64x32,
    );
    let mut background1 = tiled.background(
        Priority::P0,
        display::tiled::RegularBackgroundSize::Background64x32,
    );
    let mut card_descriptions = tiled.background(
        Priority::P1,
        display::tiled::RegularBackgroundSize::Background32x32,
    );

    let mut help_background = tiled.background(
        Priority::P1,
        display::tiled::RegularBackgroundSize::Background32x32,
    );

    let basic_die = Die {
        faces: [
            Face::Shoot,
            Face::Shield,
            Face::Blank,
            Face::Malfunction,
            Face::Blank,
            Face::Blank,
        ],
    };

    let mut star_background = StarBackground::new(&mut background0, &mut background1, &mut vram);
    star_background.commit(&mut vram);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let sfx = Sfx::new(&mut mixer);

    let mut agb = Agb {
        obj: gfx,
        vblank,
        star_background,
        vram,
        sfx,
    };

    loop {
        let mut dice = PlayerDice {
            dice: vec![basic_die.clone(); 2],
        };

        let mut current_level = 1;

        agb.sfx.title_screen();

        {
            show_title_screen(&mut help_background, &mut agb.vram, &mut agb.sfx);
            let mut score_display = NumberDisplay::new((216, 9).into());
            score_display.set_value(Some(save::load_high_score()), &agb.obj);
            agb.obj.commit();
            agb.star_background.hide();

            let mut input = agb::input::ButtonController::new();
            loop {
                let _ = agb::rng::gen();
                input.update();
                if input.is_just_pressed(agb::input::Button::all()) {
                    break;
                }
                agb.vblank.wait_for_vblank();
                agb.sfx.frame();
            }
        }

        agb.obj.commit();

        help_background.hide();
        help_background.clear(&mut agb.vram);
        help_background.commit(&mut agb.vram);
        agb.sfx.frame();

        background::load_palettes(&mut agb.vram);
        agb.star_background.show();

        loop {
            dice = customise::customise_screen(
                &mut agb,
                dice.clone(),
                &mut card_descriptions,
                &mut help_background,
                current_level,
            );

            let result =
                battle::battle_screen(&mut agb, dice.clone(), current_level, &mut help_background);
            match result {
                BattleResult::Win => {}
                BattleResult::Loss => {
                    agb.obj.commit();
                    agb.sfx.customise();
                    if save::load_high_score() < current_level {
                        save::save_high_score(&mut gba, current_level)
                            .expect("Could not save high score");
                    }
                    break;
                }
            }

            current_level += 1;

            if current_level % 5 == 0 && dice.dice.len() < 5 {
                dice.dice.push(basic_die.clone());
            }
        }
    }
}
