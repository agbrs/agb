use super::sfx::SfxPlayer;
use agb::display::tiled::{RegularMap, TileFormat, TileSet, TiledMap, VRamManager};

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
    map.set_scroll_pos((0i16, 0i16).into());
    let (tileset, settings) = match which {
        SplashScreen::Start => (
            TileSet::new(splash_screens::splash.tiles, TileFormat::FourBpp),
            splash_screens::splash.tile_settings,
        ),

        SplashScreen::End => (
            TileSet::new(
                splash_screens::thanks_for_playing.tiles,
                TileFormat::FourBpp,
            ),
            splash_screens::thanks_for_playing.tile_settings,
        ),
    };

    let vblank = agb::interrupt::VBlank::get();

    let mut input = agb::input::ButtonController::new();

    sfx.frame();
    vblank.wait_for_vblank();

    for y in 0..20u16 {
        for x in 0..30u16 {
            map.set_tile(
                vram,
                (x, y).into(),
                &tileset,
                settings[(y * 30 + x) as usize],
            );
        }

        sfx.frame();
        vblank.wait_for_vblank();
    }

    map.commit(vram);
    vram.set_background_palettes(splash_screens::PALETTES);
    map.show();

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

    map.hide();
    map.clear(vram);
}
