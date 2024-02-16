#![no_std]
#![no_main]

use agb::{
    display::{
        tiled::{RegularBackgroundSize, RegularMap, TileFormat, TiledMap, VRamManager},
        Font, Priority,
    },
    include_font, include_wav,
    sound::mixer::{Frequency, SoundChannel},
    Gba,
};

use core::fmt::Write;

// Music - "Crazy glue" by Josh Woodward, free download at http://joshwoodward.com
const CRAZY_GLUE: &[u8] = include_wav!("examples/JoshWoodward-CrazyGlue.wav");

const FONT: Font = include_font!("examples/font/yoster.ttf", 12);

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let (gfx, mut vram) = gba.display.video.tiled0();
    let mut bg = gfx.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    init_background(&mut bg, &mut vram);

    let mut title_renderer = FONT.render_text((0u16, 3u16).into());
    let mut writer = title_renderer.writer(1, 0, &mut bg, &mut vram);

    writeln!(&mut writer, "Crazy Glue by Josh Woodward").unwrap();

    writer.commit();

    bg.commit(&mut vram);
    bg.set_visible(true);

    let timer_controller = gba.timers.timers();
    let mut timer = timer_controller.timer2;
    let mut timer2 = timer_controller.timer3;
    timer.set_enabled(true);
    timer2.set_cascade(true).set_enabled(true);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut channel = SoundChannel::new(CRAZY_GLUE);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    let mut frame_counter = 0i32;
    let mut has_written_frame_time = false;

    let mut stats_renderer = FONT.render_text((0u16, 6u16).into());
    loop {
        vblank_provider.wait_for_vblank();
        bg.commit(&mut vram);

        let before_mixing_cycles_high = timer2.value();
        let before_mixing_cycles_low = timer.value();

        mixer.frame();

        let after_mixing_cycles_low = timer.value();
        let after_mixing_cycles_high = timer2.value();

        frame_counter = frame_counter.wrapping_add(1);

        if frame_counter % 128 == 0 && !has_written_frame_time {
            let before_mixing_cycles =
                ((before_mixing_cycles_high as u32) << 16) + before_mixing_cycles_low as u32;
            let after_mixing_cycles =
                ((after_mixing_cycles_high as u32) << 16) + after_mixing_cycles_low as u32;
            let total_cycles = after_mixing_cycles.wrapping_sub(before_mixing_cycles);

            let percent = (total_cycles * 100) / 280896;

            stats_renderer.clear(&mut vram);

            let mut writer = stats_renderer.writer(1, 0, &mut bg, &mut vram);
            writeln!(&mut writer, "{total_cycles} cycles").unwrap();
            writeln!(&mut writer, "{percent} percent").unwrap();

            writer.commit();

            has_written_frame_time = true;
        }
    }
}

fn init_background(bg: &mut RegularMap, vram: &mut VRamManager) {
    let background_tile = vram.new_dynamic_tile().fill_with(0);

    vram.set_background_palette_raw(&[
        0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ]);

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile(
                vram,
                (x, y).into(),
                &background_tile.tile_set(),
                background_tile.tile_setting(),
            );
        }
    }

    vram.remove_dynamic_tile(background_tile);
}
