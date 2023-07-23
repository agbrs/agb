#![no_std]
#![no_main]

use agb::sound::mixer::Frequency;
use agb::Gba;
use agb_tracker::{include_xm, Track, Tracker};

const DB_TOFFE: Track = include_xm!("examples/db_toffe.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    mixer.enable();

    let mut tracker = Tracker::new(&DB_TOFFE);

    loop {
        tracker.step(&mut mixer);
        mixer.frame();

        vblank_provider.wait_for_vblank();
    }
}
