#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

use agb::{
    display::{
        Rgb,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    dma::HBlankDmaDefinition,
    fixnum::Num,
    include_background_gfx,
    input::{Button, ButtonController},
};

const LIGHTEST_SKY_BLUE: Rgb = Rgb::new(0xd8, 0xf2, 0xff);
const DARKEST_SKY_BLUE: Rgb = Rgb::new(0x00, 0xbd, 0xff);

include_background_gfx!(sky_background, "00BDFE", SKY => "examples/gfx/sky-background.aseprite");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    let mut map = RegularBackgroundTiles::new(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    map.fill_with(&sky_background::SKY);
    VRAM_MANAGER.set_background_palettes(sky_background::PALETTES);

    let colours: Box<[_]> = (0..160)
        .map(|i| {
            let amount = Num::from(i) / 160;
            LIGHTEST_SKY_BLUE
                .interpolate(DARKEST_SKY_BLUE, amount)
                .to_rgb15()
        })
        .collect();

    let background_colour_index = VRAM_MANAGER
        .find_colour_index_16(0, DARKEST_SKY_BLUE.to_rgb15())
        .expect("Should contain darkest sky blue colour");

    let mut button_controller = ButtonController::new();

    let mut should_do_dma = true;

    loop {
        button_controller.update();
        should_do_dma ^= button_controller.is_just_pressed(Button::A);

        let mut frame = gfx.frame();

        if should_do_dma {
            HBlankDmaDefinition::new(
                VRAM_MANAGER.background_palette_colour_dma(0, background_colour_index),
                &colours,
            )
            .show(&mut frame);
        } else {
            // set the background colour back to whatever it was
            VRAM_MANAGER.set_background_palette_colour(
                0,
                background_colour_index,
                DARKEST_SKY_BLUE.to_rgb15(),
            );
        }

        map.show(&mut frame);
        frame.commit();
    }
}
