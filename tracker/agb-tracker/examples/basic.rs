#![no_std]
#![no_main]

extern crate alloc;

use agb::sound::mixer::Frequency;
use agb::Gba;
use agb_tracker::{include_xm, Track, Tracker};

// Found on: https://modarchive.org/index.php?request=view_by_moduleid&query=36662
static DB_TOFFE: Track = include_xm!("examples/db_toffe.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut tracker = Tracker::new(&DB_TOFFE);

    loop {
        tracker.step(&mut mixer);
        mixer.frame();

        vblank_provider.wait_for_vblank();
    }
}
