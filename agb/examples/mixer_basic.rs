//! Shows how you can use mono sounds and change pitch, panning and volume.
#![no_std]
#![no_main]

use agb::{
    Gba,
    display::{
        Priority, Rgb15, WIDTH,
        font::{AlignmentKind, Font, Layout, RegularBackgroundTextRenderer},
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    fixnum::num,
    include_font, include_wav,
    input::{Button, ButtonController, Tri},
    sound::mixer::{Frequency, SoundChannel, SoundData},
};

// Music - "Dead Code" by Josh Woodward, free download at http://joshwoodward.com
static DEAD_CODE: SoundData = include_wav!("examples/JoshWoodward-DeadCode.wav");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let mut input = ButtonController::new();

    let mut gfx = gba.graphics.get();
    let mut bg = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    init_background(&mut bg);

    let mut mixer = gba.mixer.mixer(Frequency::Hz10512);
    mixer.enable();

    let channel = SoundChannel::new(DEAD_CODE);
    let channel_id = mixer.play_sound(channel).unwrap();

    loop {
        let mut frame = gfx.frame();

        input.update();

        {
            if let Some(channel) = mixer.channel(&channel_id) {
                match input.x_tri() {
                    Tri::Negative => channel.panning(num!(0.5)),
                    Tri::Zero => channel.panning(0),
                    Tri::Positive => channel.panning(num!(-0.5)),
                };

                match input.y_tri() {
                    Tri::Negative => channel.playback(num!(1.5)),
                    Tri::Zero => channel.playback(1),
                    Tri::Positive => channel.playback(num!(0.5)),
                };

                if input.is_pressed(Button::L) {
                    channel.volume(num!(0.5));
                } else if input.is_pressed(Button::R) {
                    channel.volume(20); // intentionally introduce clipping
                } else {
                    channel.volume(1);
                }

                if input.is_pressed(Button::A) {
                    channel.resume();
                }

                if input.is_pressed(Button::B) {
                    channel.pause();
                }
            }
        }

        mixer.frame();
        bg.show(&mut frame);
        frame.commit();
    }
}

fn init_background(bg: &mut RegularBackgroundTiles) {
    static FONT: Font = include_font!("examples/font/ark-pixel-10px-proportional-ja.ttf", 10);

    VRAM_MANAGER.set_background_palette_colour(0, 1, Rgb15::WHITE);

    let text_layout = Layout::new(
        "Dead code by Josh Woodward\njoshwoodward.com\n
L to play half volume
R to play 20x volume
B to pause
A to resume
D-pad left and right to change panning
D-pad up and down to change playback speed",
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
