#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        object::{OamIterator, OamUnmanaged, SpriteLoader},
        tiled::{
            BackgroundIterator, RegularBackgroundSize, RegularBackgroundTiles, TileFormat,
            TiledBackground,
        },
        Priority,
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
    loader: SpriteLoader,
    sfx: Sfx<'gba>,
    oam: OamUnmanaged<'gba>,
    gfx: TiledBackground<'gba>,
}

impl<'gba> Agb<'gba> {
    fn frame<D, U, T, F>(&mut self, data: &mut D, update: U, render: F) -> T
    where
        U: FnOnce(&mut D, &ButtonController, &mut SpriteLoader, &mut Sfx<'gba>) -> T,
        F: FnOnce(&D, &mut OamIterator, &mut SpriteLoader, &mut BackgroundIterator<'_>),
    {
        self.vblank.wait_for_vblank();

        self.input.update();
        {
            let mut bg_iter = self.gfx.iter();
            render(data, &mut self.oam.iter(), &mut self.loader, &mut bg_iter);

            bg_iter.commit();
        }
        self.sfx.frame();

        update(data, &self.input, &mut self.loader, &mut self.sfx)
    }
}

pub fn entry(mut gba: agb::Gba) -> ! {
    let vblank = VBlank::get();

    let _ = save::init_save(&mut gba);

    let gfx = gba.display.video.tiled();
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

    let (unmanaged, sprite_loader) = gba.display.object.get_unmanaged();

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
        loader: sprite_loader,
        sfx,
        oam: unmanaged,
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
                    |_, input, _, _| input.is_just_pressed(Button::SELECT),
                    |_, _, _, bg_iter| {
                        ending_bg.show(bg_iter);
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
                |_, _, loader, _| Pausable::new(current_level, maximum_level, loader),
                |_, _, _, _| {},
            );

            loop {
                if let Some(option) = g.frame(
                    &mut game,
                    |game, input, loader, sfx| game.update(input, sfx, loader),
                    |game, oam, loader, bg_iter| {
                        ui_bg.show(bg_iter);
                        game.render(loader, oam, bg_iter)
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
fn agb_test_main(gba: agb::Gba) -> ! {
    loop {
        // full implementation provided by the #[entry]
        agb::syscall::halt();
    }
}
