use super::sfx::MusicBox;
use agb::sound::mixer::Mixer;

agb::include_gfx!("gfx/splash_screens.toml");

pub enum SplashScreen {
    Start,
    End,
}

pub fn show_splash_screen(
    agb: &mut agb::Gba,
    which: SplashScreen,
    mut mixer: Option<&mut Mixer>,
    mut music_box: Option<&mut MusicBox>,
) {
    let mut tiled = agb.display.video.tiled0();

    match which {
        SplashScreen::Start => {
            tiled.set_background_tilemap(0, splash_screens::splash.tiles);
            tiled.set_background_palettes(splash_screens::splash.palettes);
        }
        SplashScreen::End => {
            tiled.set_background_tilemap(0, splash_screens::thanks_for_playing.tiles);
            tiled.set_background_palettes(splash_screens::thanks_for_playing.palettes);
        }
    }
    let vblank = agb::interrupt::VBlank::get();
    let mut splash_screen_display = tiled.get_regular().unwrap();

    let mut entries: [u16; 30 * 20] = [0; 30 * 20];
    for tile_id in 0..(30 * 20) {
        entries[tile_id as usize] = tile_id;
    }
    let mut input = agb::input::ButtonController::new();
    splash_screen_display.set_map(agb::display::background::Map::new(
        &entries,
        (30_u32, 20_u32).into(),
        0,
    ));
    splash_screen_display.set_position((0, 0).into());
    splash_screen_display.commit();
    splash_screen_display.show();
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
        if let Some(ref mut mixer) = mixer {
            if let Some(ref mut music_box) = music_box {
                music_box.before_frame(mixer);
            }
            mixer.frame();
        }
        vblank.wait_for_vblank();

        if let Some(ref mut mixer) = mixer {
            mixer.after_vblank();
        }
    }
    splash_screen_display.hide();
}
