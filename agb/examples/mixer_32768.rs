//! This example shows the audio quality you can get from using the mixer at a frequency
//! of 32768Hz. This is the highest audio quality supported by agb, but also uses the most
//! space and CPU time (approximately 5% of frame time).
#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    Gba,
    display::{
        Priority, Rgb15, WIDTH,
        font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat, VRAM_MANAGER},
    },
    include_font, include_wav,
    sound::mixer::{Frequency, SoundChannel, SoundData},
};

// Music - "Crazy glue" by Josh Woodward, free download at http://joshwoodward.com
static CRAZY_GLUE: SoundData = include_wav!("examples/JoshWoodward-CrazyGlue.wav");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let mut gfx = gba.graphics.get();
    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    init_background(&mut bg);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut channel = SoundChannel::new(CRAZY_GLUE);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    loop {
        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.commit();
        mixer.frame();
    }
}

fn init_background(bg: &mut RegularBackground) {
    static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

    VRAM_MANAGER.set_background_palette_colour(0, 1, Rgb15::WHITE);

    let text_layout = Layout::new(
        "Crazy glue by Josh Woodward\njoshwoodward.com",
        &FONT,
        AlignmentKind::Centre,
        WIDTH,
        WIDTH,
    );

    let mut renderer = RegularBackgroundTextRenderer::new((0, 0));
    for lg in text_layout {
        renderer.show(bg, &lg);
    }
}
