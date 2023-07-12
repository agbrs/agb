#![no_std]
#![no_main]

use agb::sound::mixer::Frequency;
use agb::Gba;
use agb_tracker::{import_xm, Track, Tracker};

const AJOJ: Track = import_xm!("examples/db_toffe.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    mixer.enable();

    let mut tracker = Tracker::new(&AJOJ);

    loop {
        tracker.step(&mut mixer);
        mixer.frame();
        vblank_provider.wait_for_vblank();
    }
}
