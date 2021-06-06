#![no_std]
#![no_main]

extern crate agb;

use agb::sound::mixer::SoundChannel;
use agb::Gba;

// Music - "I will not let you let me down" by Josh Woodward, free download at http://joshwoodward.com
const I_WILL_NOT_LET_YOU_LET_ME_DOWN: &[u8] = include_bytes!("i-will-not-let-you-let-me-down.raw");

#[no_mangle]
pub fn main() -> ! {
    let mut gba = Gba::new();
    let vblank_provider = gba.display.vblank.get();

    let mut mixer = gba.mixer.mixer();
    mixer.enable();

    let channel = SoundChannel::new(I_WILL_NOT_LET_YOU_LET_ME_DOWN);
    mixer.play_sound(channel);

    loop {
        vblank_provider.wait_for_VBlank();
        mixer.vblank();
    }
}
