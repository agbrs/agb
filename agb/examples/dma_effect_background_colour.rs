//! This is an example of using palette colour DMA to change the colour each scan line
//! resulting in more colours than can fit in the Game Boy Advance's palette.
//!
//! There is banding in the final image because of the 15-bit colour used by the
//! Game Boy Advance so we can't get enough precision to show a smooth colour gradient.
#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    display::{
        Rgb, Rgb15,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    dma::HBlankDmaDefinition,
    include_background_gfx, include_colours,
    input::{Button, ButtonController},
};

const DARKEST_SKY_BLUE: Rgb = Rgb::new(0x00, 0xbd, 0xff);

include_background_gfx!(mod sky_background, "00BDFE", SKY => "examples/gfx/sky-background.aseprite");

static SKY_GRADIENT: [Rgb15; 160] =
    include_colours!("examples/gfx/sky-background-gradient.aseprite");

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
                &SKY_GRADIENT,
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
