use super::sfx::MusicBox;
use agb::{
    display::background::{RegularMap, TileFormat, TileSet, TileSetting, VRamManager},
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
    map.set_scroll_pos((0u16, 0u16).into());
    let tile_set_ref = match which {
        SplashScreen::Start => {
            let tile_set_ref = vram.add_tileset(TileSet::new(
                splash_screens::splash.tiles,
                TileFormat::FourBpp,
            ));

            vram.set_background_palettes(splash_screens::splash.palettes);

            tile_set_ref
        }
        SplashScreen::End => {
            let tile_set_ref = vram.add_tileset(TileSet::new(
                splash_screens::thanks_for_playing.tiles,
                TileFormat::FourBpp,
            ));

            vram.set_background_palettes(splash_screens::thanks_for_playing.palettes);

            tile_set_ref
        }
    };

    let vblank = agb::interrupt::VBlank::get();

    let mut input = agb::input::ButtonController::new();

    for y in 0..20u16 {
        for x in 0..30u16 {
            map.set_tile(
                vram,
                (x, y).into(),
                tile_set_ref,
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

        if let Some(ref mut mixer) = mixer {
            mixer.after_vblank();
        }
    }

    map.commit();
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

    map.hide();
    map.clear(vram);

    vram.remove_tileset(tile_set_ref);
}
