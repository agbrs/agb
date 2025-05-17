use super::sfx::SfxPlayer;
use agb::display::{
    Graphics, Priority,
    tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
};

agb::include_background_gfx!(mod splash_screens,
    splash => deduplicate "gfx/splash.png",
    thanks_for_playing => deduplicate "gfx/thanks_for_playing.png",
);

pub enum SplashScreen {
    Start,
    End,
}

pub fn show_splash_screen(gfx: &mut Graphics, which: SplashScreen, sfx: &mut SfxPlayer) {
    let mut map = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    map.set_scroll_pos((0i16, 0i16));
    let tile_data = match which {
        SplashScreen::Start => &splash_screens::splash,
        SplashScreen::End => &splash_screens::thanks_for_playing,
    };

    let mut input = agb::input::ButtonController::new();

    sfx.frame();

    map.fill_with(tile_data);

    VRAM_MANAGER.set_background_palettes(splash_screens::PALETTES);

    loop {
        let mut frame = gfx.frame();
        map.show(&mut frame);

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
        frame.commit();
    }
}
