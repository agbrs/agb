#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    sound::maxmod::{self, include_sounds},
    Gba,
};

include_sounds!("examples/pk_lz0_07.xm");

#[agb::entry]
fn main(mut _gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    maxmod::init(music::SOUNDBANK_DATA, 8);
    maxmod::start(music::MOD__________);

    loop {
        vblank_provider.wait_for_vblank();
        maxmod::vblank();
        maxmod::frame();
    }
}
