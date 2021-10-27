#![no_std]
#![no_main]

extern crate agb;

use agb::sound::mixer::SoundChannel;
use agb::{include_wav, Gba};

// Music - "Let it in" by Josh Woodward, free download at http://joshwoodward.com
const LET_IT_IN: &[u8] = include_wav!("examples/JoshWoodward-LetItIn.wav");

#[agb::entry]
fn main() -> ! {
    let mut gba = Gba::new();
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut mixer = gba.mixer.mixer();
    mixer.enable();

    let mut channel = SoundChannel::new(LET_IT_IN);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    loop {
        vblank_provider.wait_for_vblank();
        mixer.vblank();
    }
}
