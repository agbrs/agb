#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat},
        Graphics, GraphicsFrame, Priority,
    },
    input::{Button, ButtonController},
    interrupt::VBlank,
    sound::mixer::Frequency,
};
use game::{Pausable, PauseSelection};

use sfx::Sfx;

extern crate alloc;

mod backgrounds;
mod level;
mod map;
mod resources;
mod sfx;

mod game;

mod save;

struct Agb<'gba> {
    vblank: VBlank,
    input: ButtonController,
    sfx: Sfx<'gba>,
    gfx: Graphics<'gba>,
}

impl<'gba> Agb<'gba> {
    fn frame<D, U, T, F>(&mut self, data: &mut D, update: U, render: F) -> T
    where
        U: FnOnce(&mut D, &ButtonController, &mut Sfx<'gba>) -> T,
        F: FnOnce(&D, &mut GraphicsFrame),
    {
        let mut frame = self.gfx.frame();
        render(data, &mut frame);

        self.vblank.wait_for_vblank();
        frame.commit();

        self.sfx.frame();
        self.input.update();

        update(data, &self.input, &mut self.sfx)
    }
}

pub fn entry(mut gba: agb::Gba) -> ! {
    let vblank = VBlank::get();

    let _ = save::init_save(&mut gba);

    let gfx = gba.display.graphics.get();
    let mut ui_bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut ending_bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    backgrounds::load_ending_page(&mut ending_bg);
    ending_bg.commit();

    backgrounds::load_palettes();
    backgrounds::load_ui(&mut ui_bg);

    ui_bg.commit();

    let mut input = agb::input::ButtonController::new();
    input.update();

    if input.is_pressed(Button::START | Button::SELECT | Button::L | Button::R) {
        let _ = save::save_max_level(&mut gba.save, 0);
    }

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    let sfx = Sfx::new(&mut mixer);

    let mut g = Agb {
        vblank,
        input,
        sfx,
        gfx,
    };

    let saved_level = save::load_max_level() as usize;

    let mut current_level = saved_level;
    let mut maximum_level = saved_level;
    loop {
        if current_level >= level::Level::num_levels() {
            current_level = 0;

            loop {
                if g.frame(
                    &mut (),
                    |_, input, _| input.is_just_pressed(Button::SELECT),
                    |_, frame| {
                        ending_bg.show(frame);
                    },
                ) {
                    break;
                }
            }
        } else {
            if current_level > maximum_level {
                maximum_level = current_level;
                let _ = save::save_max_level(&mut gba.save, maximum_level as u32);
            }
            let mut game = g.frame(
                &mut (),
                |_, _, _| Pausable::new(current_level, maximum_level),
                |_, frame| {
                    ui_bg.show(frame);
                },
            );

            loop {
                if let Some(option) = g.frame(
                    &mut game,
                    |game, input, sfx| game.update(input, sfx),
                    |game, frame| {
                        ui_bg.show(frame);
                        game.render(frame)
                    },
                ) {
                    match option {
                        game::UpdateResult::MenuSelection(PauseSelection::Restart) => break,
                        game::UpdateResult::MenuSelection(PauseSelection::LevelSelect(level)) => {
                            current_level = level;
                            break;
                        }
                        game::UpdateResult::NextLevel => {
                            current_level += 1;
                            break;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[agb::entry]
fn agb_test_main(_gba: agb::Gba) -> ! {
    loop {
        // full implementation provided by the #[entry]
        agb::syscall::halt();
    }
}
