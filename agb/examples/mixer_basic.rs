#![no_std]
#![no_main]

extern crate agb;

use core::mem;

// Music - "I will not let you let me down" by Josh Woodward, free download at http://joshwoodward.com
const I_WILL_NOT_LET_YOU_LET_ME_DOWN: &[u8] = include_bytes!("i-will-not-let-you-let-me-down.raw");

#[no_mangle]
pub fn main() -> ! {
    let gba = agb::Gba::new();
    let mixer = gba.mixer;
    mixer.enable();

    mixer.play_sound_starting_at(I_WILL_NOT_LET_YOU_LET_ME_DOWN);

    loop {}
}
