#![no_std]
#![no_main]

use agb::sound::mixer::Frequency;
use agb::Gba;
use agb_tracker::{include_xm, Track, Tracker};

const DB_TOFFE: Track = include_xm!("examples/db_toffe.xm");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let vblank_provider = agb::interrupt::VBlank::get();

    let timer_controller = gba.timers.timers();
    let mut timer = timer_controller.timer2;
    let mut timer2 = timer_controller.timer3;
    timer.set_enabled(true);
    timer2.set_cascade(true).set_enabled(true);

    let mut mixer = gba.mixer.mixer(Frequency::Hz18157);
    mixer.enable();

    let mut tracker = Tracker::new(&DB_TOFFE);

    loop {
        let before_mixing_cycles_high = timer2.value();
        let before_mixing_cycles_low = timer.value();

        tracker.step(&mut mixer);

        let after_step_cycles_high = timer2.value();
        let after_step_cycles_low = timer.value();

        mixer.frame();
        let after_mixing_cycles_low = timer.value();
        let after_mixing_cycles_high = timer2.value();

        vblank_provider.wait_for_vblank();

        let before_mixing_cycles =
            ((before_mixing_cycles_high as u32) << 16) + before_mixing_cycles_low as u32;
        let after_mixing_cycles =
            ((after_mixing_cycles_high as u32) << 16) + after_mixing_cycles_low as u32;
        let after_step_cycles =
            ((after_step_cycles_high as u32) << 16) + after_step_cycles_low as u32;

        let step_cycles = after_step_cycles - before_mixing_cycles;
        let mixing_cycles = after_mixing_cycles - before_mixing_cycles;
        let total_cycles = after_mixing_cycles.wrapping_sub(before_mixing_cycles);

        agb::println!(
            "step = {step_cycles}, mixing = {mixing_cycles}, total = {total_cycles} cycles"
        );
    }
}
