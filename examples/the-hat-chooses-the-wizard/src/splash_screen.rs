use super::sfx::SfxPlayer;
use agb::display::tiled::{RegularMap, TileFormat, TileSet, TileSetting, TiledMap, VRamManager};

agb::include_gfx!("gfx/splash_screens.toml");

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
    let tileset = match which {
        SplashScreen::Start => TileSet::new(splash_screens::splash.tiles, TileFormat::FourBpp),

        SplashScreen::End => TileSet::new(
            splash_screens::thanks_for_playing.tiles,
            TileFormat::FourBpp,
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
                TileSetting::from_raw(y * 30 + x),
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
