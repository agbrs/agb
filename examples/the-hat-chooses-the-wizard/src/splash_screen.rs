use super::sfx::MusicBox;
use agb::{
    display::tiled::{RegularMap, TileFormat, TileSet, TileSetting, TiledMap, VRamManager},
    sound::mixer::Mixer,
};

agb::include_gfx!("gfx/splash_screens.toml");

pub enum SplashScreen {
    Start,
    End,
}

pub fn show_splash_screen(
    which: SplashScreen,
    mut mixer: Option<&mut Mixer>,
    mut music_box: Option<&mut MusicBox>,
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

    if let Some(ref mut mixer) = mixer {
        if let Some(ref mut music_box) = music_box {
            music_box.before_frame(mixer);
        }
        mixer.frame();
    }

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

        if let Some(ref mut mixer) = mixer {
            if let Some(ref mut music_box) = music_box {
                music_box.before_frame(mixer);
            }
            mixer.frame();
        }

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
        if let Some(mixer) = &mut mixer {
            if let Some(music_box) = &mut music_box {
                music_box.before_frame(mixer);
            }
            mixer.frame();
        }
        vblank.wait_for_vblank();
    }

    map.hide();
    map.clear(vram);
}
