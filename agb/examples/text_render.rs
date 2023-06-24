#![no_std]
#![no_main]

use agb::{
    display::{
        tiled::{RegularBackgroundSize, TileFormat, TileSetting, TiledMap},
        video::Tiled0Vram,
        Font, Priority,
    },
    include_font,
};

use core::fmt::Write;

const FONT: Font = include_font!("examples/font/yoster.ttf", 12);

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, vram) = &mut *gba.display.video.get::<Tiled0Vram>();
    let vblank = agb::interrupt::VBlank::get();

    vram.set_background_palette_raw(&[
        0x0000, 0x0ff0, 0x00ff, 0xf00f, 0xf0f0, 0x0f0f, 0xaaaa, 0x5555, 0x0000, 0x0000, 0x0000,
        0x0000, 0x0000, 0x0000, 0x0000, 0x0000,
    ]);

    let background_tile = vram.new_dynamic_tile().fill_with(0);

    let mut bg = gfx.background(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for y in 0..20u16 {
        for x in 0..30u16 {
            bg.set_tile(
                vram,
                (x, y).into(),
                &background_tile.tile_set(),
                TileSetting::from_raw(background_tile.tile_index()),
            );
        }
    }

    vram.remove_dynamic_tile(background_tile);

    let mut renderer = FONT.render_text((0u16, 3u16).into());
    let mut writer = renderer.writer(1, 2, &mut bg, vram);

    writeln!(&mut writer, "Hello, World!").unwrap();
    writeln!(&mut writer, "This is a font rendering example").unwrap();

    writer.commit();

    bg.commit(vram);
    bg.show();

    let mut frame = 0;

    loop {
        let mut renderer = FONT.render_text((4u16, 0u16).into());
        let mut writer = renderer.writer(1, 2, &mut bg, vram);

        writeln!(&mut writer, "Frame {frame}").unwrap();
        writer.commit();

        frame += 1;

        vblank.wait_for_vblank();
        bg.commit(vram);

        renderer.clear(vram);
    }
}
