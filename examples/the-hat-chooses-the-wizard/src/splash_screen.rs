use super::sfx::SfxPlayer;
use agb::display::tiled::{RegularMap, TiledMap, VRamManager};

agb::include_background_gfx!(splash_screens,
    splash => deduplicate "gfx/splash.png",
    thanks_for_playing => deduplicate "gfx/thanks_for_playing.png",
);

pub enum SplashScreen {
    Start,
    End,
}

pub fn show_splash_screen(
    which: SplashScreen,
    sfx: &mut SfxPlayer,
    map: &mut RegularMap,
    vram: &mut VRamManager,
) {
    map.set_scroll_pos((0i16, 0i16));
    let tile_data = match which {
        SplashScreen::Start => splash_screens::splash,
        SplashScreen::End => splash_screens::thanks_for_playing,
    };

    let vblank = agb::interrupt::VBlank::get();

    let mut input = agb::input::ButtonController::new();

    sfx.frame();
    vblank.wait_for_vblank();

    map.fill_with(vram, &tile_data);

    map.commit(vram);
    vram.set_background_palettes(splash_screens::PALETTES);
    map.set_visible(true);

    loop {
        input.update();
        if input.is_just_pressed(
            agb::input::Button::A
                | agb::input::Button::B
                | agb::input::Button::START
                | agb::input::Button::SELECT,
        ) {
            break;
        }

        sfx.frame();
        vblank.wait_for_vblank();
    }

    map.set_visible(false);
    map.clear(vram);
}
