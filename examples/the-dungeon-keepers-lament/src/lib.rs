#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{
        object::{OamIterator, OamUnmanaged, SpriteLoader},
        tiled::{RegularBackgroundSize, TileFormat, TiledMap, VRamManager},
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
    vram: VRamManager,
    oam: OamUnmanaged<'gba>,
}

impl<'gba> Agb<'gba> {
    fn frame<D, U, T, F>(&mut self, data: &mut D, update: U, render: F) -> T
    where
        U: FnOnce(
            &mut D,
            &ButtonController,
            &mut SpriteLoader,
            &mut Sfx<'gba>,
            &mut VRamManager,
        ) -> T,
        F: FnOnce(&D, &mut OamIterator, &mut SpriteLoader),
    {
        self.vblank.wait_for_vblank();
        self.input.update();
        {
            render(data, &mut self.oam.iter(), &mut self.loader);
        }
        self.sfx.frame();

        update(
            data,
            &self.input,
            &mut self.loader,
            &mut self.sfx,
            &mut self.vram,
        )
    }
}

pub fn entry(mut gba: agb::Gba) -> ! {
    let vblank = VBlank::get();

    let _ = save::init_save(&mut gba);

    let (tiled, mut vram) = gba.display.video.tiled0();
    let mut ui_bg = tiled.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut level_bg = tiled.background(
        Priority::P1,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let mut ending_bg = tiled.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    backgrounds::load_ending_page(&mut ending_bg, &mut vram);
    ending_bg.commit(&mut vram);

    backgrounds::load_palettes(&mut vram);
    backgrounds::load_ui(&mut ui_bg, &mut vram);

    ui_bg.commit(&mut vram);
    ui_bg.show();

    let (unmanaged, sprite_loader) = gba.display.object.get_unmanaged();

    let mut input = agb::input::ButtonController::new();
    input.update();

    if input.is_pressed(Button::START | Button::SELECT | Button::L | Button::R) {
        let _ = save::save_max_level(&mut gba.save, 0);
    }

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    let sfx = Sfx::new(&mut mixer);

    let mut g = Agb {
        vblank,
        input,
        loader: sprite_loader,
        sfx,
        vram,
        oam: unmanaged,
    };

    let mut current_level = 0;
    let mut maximum_level = save::load_max_level() as usize;
    loop {
        if current_level >= level::Level::num_levels() {
            current_level = 0;
            ui_bg.hide();
            level_bg.hide();
            ending_bg.show();
            loop {
                if g.frame(
                    &mut (),
                    |_, input, _, _, _| input.is_just_pressed(Button::SELECT),
                    |_, _, _| {},
                ) {
                    break;
                }
            }
            ui_bg.show();
            ending_bg.hide();
        } else {
            if current_level > maximum_level {
                maximum_level = current_level;
                let _ = save::save_max_level(&mut gba.save, maximum_level as u32);
            }
            let mut game = g.frame(
                &mut (),
                |_, _, loader, _, _| {
                    Pausable::new(current_level, maximum_level, &mut level_bg, loader)
                },
                |_, _, _| {},
            );

            loop {
                if let Some(option) = g.frame(
                    &mut game,
                    |game, input, loader, sfx, vram| game.update(input, sfx, loader, vram),
                    |game, oam, loader| game.render(loader, oam),
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
