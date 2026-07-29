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
        font::{AlignmentKind, Font, Layout, LayoutSettings, RegularBackgroundTextRenderer},
        tiled::{RegularBackground, RegularBackgroundSize, TileFormat},
    },
    fixnum::num,
    include_font, include_wav,
    input::{ButtonController, Tri},
    sound::mixer::{Frequency, SoundChannel, SoundData},
};

// Music - "Crazy glue" by Josh Woodward, free download at http://joshwoodward.com
static CRAZY_GLUE: SoundData = include_wav!("examples/JoshWoodward-CrazyGlue.wav");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let mut gfx = gba.graphics.get();
    gfx.set_background_palette_colour(0, 1, Rgb15::WHITE);

    let mut bg = RegularBackground::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    init_background(&mut bg);

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);

    let mut channel = SoundChannel::new_high_priority(CRAZY_GLUE);
    channel.stereo().should_loop();
    let channel_id = mixer.play_sound(channel).unwrap();

    let mut input = ButtonController::new();

    loop {
        input.update();
        let volume = match input.y_tri() {
            Tri::Positive => num!(0.5),
            Tri::Zero => num!(1),
            Tri::Negative => num!(1.5),
        };

        mixer.channel(&channel_id).unwrap().volume(volume);

        let mut frame = gfx.frame();
        bg.show(&mut frame);

        frame.commit();
        mixer.frame();
    }
}

fn init_background(bg: &mut RegularBackground) {
    static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

    let text_layout = Layout::new(
        "Crazy glue by Josh Woodward\njoshwoodward.com\n\nUP to go louder, DOWN for quieter",
        &FONT,
        &LayoutSettings::new()
            .with_max_line_length(WIDTH)
            .with_alignment(AlignmentKind::Centre),
    );

    let mut renderer = RegularBackgroundTextRenderer::new((0, 0), 0);
    for lg in text_layout {
        renderer.show(bg, &lg);
    }
}
