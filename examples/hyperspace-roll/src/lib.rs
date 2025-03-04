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

use agb::sound::mixer::Frequency;
use agb::{display::Graphics, interrupt::VBlank};

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
        let n = agb::rng::next_i32().rem_euclid(6);
        self.faces[n as usize]
    }
}

#[derive(Debug, Clone)]
pub struct PlayerDice {
    dice: Vec<Die>,
}

struct Agb<'a> {
    gfx: Graphics<'a>,
    vblank: VBlank,
    star_background: StarBackground,
    sfx: Sfx<'a>,
}

pub fn main(mut gba: agb::Gba) -> ! {
    save::init_save(&mut gba).expect("Could not initialize save game");

    if save::load_high_score() > 1000 {
        save::save_high_score(&mut gba.save, 0).expect("Could not reset high score");
    }

    let gfx = gba.display.graphics.get();
    let vblank = agb::interrupt::VBlank::get();

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

    let mut star_background = StarBackground::new();
    star_background.commit();

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let sfx = Sfx::new(&mut mixer);

    let mut agb = Agb {
        gfx,
        vblank,
        star_background,
        sfx,
    };

    loop {
        let mut dice = PlayerDice {
            dice: vec![basic_die.clone(); 2],
        };

        let mut current_level = 1;

        agb.sfx.title_screen();

        {
            let title_screen_bg = show_title_screen(&mut agb.sfx);
            let mut score_display = NumberDisplay::new((216, 9).into());
            score_display.set_value(Some(save::load_high_score()));

            let mut input = agb::input::ButtonController::new();
            loop {
                let _ = agb::rng::next_i32();
                input.update();
                if input.is_just_pressed(agb::input::Button::all()) {
                    break;
                }

                let mut frame = agb.gfx.frame();
                score_display.show(&mut frame);
                title_screen_bg.show(&mut frame);
                agb.vblank.wait_for_vblank();
                frame.commit();

                agb.sfx.frame();
            }
        }

        agb.sfx.frame();

        background::load_palettes();

        loop {
            dice = customise::customise_screen(&mut agb, dice.clone(), current_level);

            let result = battle::battle_screen(&mut agb, dice.clone(), current_level);
            match result {
                BattleResult::Win => {}
                BattleResult::Loss => {
                    agb.sfx.customise();
                    if save::load_high_score() < current_level {
                        save::save_high_score(&mut gba.save, current_level)
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
