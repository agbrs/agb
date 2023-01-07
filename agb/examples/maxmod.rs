#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    sound::maxmod::{include_sounds, MixMode},
    Gba,
};

include_sounds!("examples/pk_lz0_07.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();
    let tracker = gba.mixer.tracker::<music::Music>(8, MixMode::Hz31);

    tracker.start(music::ModFiles::MOD_PK_LZ0_07);

    let mut timer = gba.timers.timers().timer2;
    timer.set_enabled(true);

    loop {
        vblank_provider.wait_for_vblank();

        let before = timer.value();
        tracker.frame();
        let after = timer.value();

        agb::println!(
            "Took {}% cpu time",
            after.wrapping_sub(before) as u32 * 100 / 280896
        );
    }
}
