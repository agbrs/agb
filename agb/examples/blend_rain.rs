//! This example uses blend to make a thunder effect fill the screen, but not change the
//! character sprite
#![no_main]
#![no_std]

use agb::{
    display::{
        GraphicsFrame, Priority,
        tiled::{
            RegularBackground, RegularBackgroundId, RegularBackgroundSize, TileFormat, VRAM_MANAGER,
        },
    },
    fixnum::{Num, num},
    include_aseprite, include_background_gfx, include_wav, rng,
    sound::mixer::{Frequency, Mixer, SoundChannel, SoundData},
};

include_aseprite!(mod sprites, "examples/gfx/crab.aseprite");
include_background_gfx!(mod backgrounds,
    BEACH => deduplicate "examples/gfx/beach-background-rain.aseprite",
);

// Just 1s of generated static
static RAIN: SoundData = include_wav!("examples/sfx/rain.wav");
// From here: https://opengameart.org/content/100-cc0-sfx-2 (CC0)
// For a real game, you might want a few different options for thunder effects, because just one
// sounds quite repetitive.
static THUNDER: SoundData = include_wav!("examples/sfx/thunder.wav");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Set up the background palettes as needed. These are produced by the include_background_gfx! macro call above.
    VRAM_MANAGER.set_background_palettes(backgrounds::PALETTES);

    // Get access to the graphics struct which is used to manage the frame lifecycle
    let mut gfx = gba.graphics.get();

    let mut bg_tiles = RegularBackground::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    bg_tiles.fill_with(&backgrounds::BEACH);

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    let mut rain_channel = SoundChannel::new(RAIN);
    rain_channel.should_loop().volume(num!(0.15));
    mixer.play_sound(rain_channel);

    let mut thunder = ThunderEffect::new();

    loop {
        thunder.update(&mut mixer);

        let mut frame = gfx.frame();

        // Blends apply to specific backgrounds. So we get the background ID here to enable blending
        // on this one.
        let bg_id = bg_tiles.show(&mut frame);
        thunder.show(&mut frame, bg_id);

        // `.commit()` on frame will ensure that everything is drawn to the screen, and also wait
        // for the frame to finish rendering before returning control.
        mixer.frame();
        frame.commit();
    }
}

#[derive(Clone, Copy, Default)]
enum ThunderStatus {
    Flash(Num<u8, 4>),
    Wait(i16),
    Rumble,
    #[default]
    None,
}

impl ThunderStatus {
    fn start(&mut self) {
        *self = ThunderStatus::Flash(num!(1));
    }

    fn update(&mut self, mixer: &mut Mixer) {
        match self {
            ThunderStatus::Flash(num) => {
                *num -= Num::from_raw(1); // the smallest positive number
                if *num == num!(0) {
                    *self = ThunderStatus::Wait(((rng::next_i32() as u16) % 128) as i16);
                }
            }
            ThunderStatus::Wait(i) => {
                *i -= 1;
                if *i == 0 {
                    *self = ThunderStatus::Rumble;
                }
            }
            ThunderStatus::Rumble => {
                mixer.play_sound(SoundChannel::new(THUNDER));
                *self = ThunderStatus::None;
            }
            ThunderStatus::None => {}
        }
    }

    fn show(&self, frame: &mut GraphicsFrame, bg_id: RegularBackgroundId) {
        if let ThunderStatus::Flash(value) = self {
            frame.blend().brighten(*value).enable_background(bg_id);
        }
    }
}

struct ThunderEffect {
    next_thunder: usize,
    status: ThunderStatus,
}

impl ThunderEffect {
    fn new() -> Self {
        Self {
            next_thunder: 60 * 5, // start the next one within 5 seconds
            status: Default::default(),
        }
    }

    fn update(&mut self, mixer: &mut Mixer) {
        self.status.update(mixer);

        if matches!(self.status, ThunderStatus::None) {
            self.next_thunder -= 1;
            if self.next_thunder <= (rng::next_i32() as usize) % 256 {
                self.next_thunder = 60 * 10;
                self.status.start();
            }
        }
    }

    fn show(&self, frame: &mut GraphicsFrame, bg_id: RegularBackgroundId) {
        self.status.show(frame, bg_id);
    }
}
