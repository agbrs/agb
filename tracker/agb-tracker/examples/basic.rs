#![no_std]
#![no_main]

extern crate alloc;

use agb::Gba;
use agb::sound::mixer::Frequency;
use agb_tracker::{Track, Tracker, include_xm};

static SPECTRUM: Track = include_xm!("examples/tracks/peak_and_drozerix_-_spectrum.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);
    mixer.enable();

    let mut tracker = Tracker::new(&SPECTRUM);

    loop {
        tracker.step(&mut mixer);
        mixer.frame();

        vblank_provider.wait_for_vblank();
    }
}
