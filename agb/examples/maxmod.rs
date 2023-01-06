#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    sound::maxmod::{self, include_sounds, MixMode},
    Gba,
};

include_sounds!("examples/pk_lz0_07.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    maxmod::init(music::SOUNDBANK_DATA, 8, MixMode::Hz31);
    maxmod::start(music::MOD_PK_LZ0_07);

    let mut timer = gba.timers.timers().timer2;
    timer.set_enabled(true);

    loop {
        vblank_provider.wait_for_vblank();
        maxmod::vblank();

        let before = timer.value();
        maxmod::frame();
        let after = timer.value();

        agb::println!(
            "Took {}% cpu time",
            after.wrapping_sub(before) as u32 * 100 / 280896
        );
    }
}
