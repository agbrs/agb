#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::{
    display::{
        example_logo,
        tiled::{RegularBackgroundSize, TileFormat, TiledMap},
    },
    interrupt::VBlank,
};

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();

    let mut input = agb::input::ButtonController::new();

    let mut map = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    let dma = gba.dma.dma().dma0;

    example_logo::display_logo(&mut map, &mut vram);

    let vblank = VBlank::get();

    let offsets: Box<[_]> = (0..160 * 2).collect();
    let colours: Box<[_]> = (0..160).map(|i| ((i * 0xffff) / 160) as u16).collect();

    let mut frame = 0;

    let mut effect = false;

    loop {
        input.update();

        if input.is_just_pressed(agb::input::Button::A) {
            effect = !effect;
        }

        let _x_scroll_transfer = if effect {
            Some(unsafe { dma.hblank_transfer(&map.x_scroll_dma(), &offsets[frame..]) })
        } else {
            map.set_scroll_pos((0i16, 0i16));
            None
        };

        let _background_color_transfer = if !effect {
            Some(unsafe {
                dma.hblank_transfer(&vram.background_palette_colour_dma(0, 2), &colours)
            })
        } else {
            vram.set_background_palette_colour(0, 0, 0xffff);
            None
        };

        map.commit(&mut vram);
        vblank.wait_for_vblank();
        frame += 1;
        if frame > 160 {
            frame = 0;
        }
    }
}
