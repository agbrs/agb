#![no_std]
#![no_main]

extern crate agb;

use agb::sound::mixer::SoundChannel;
use agb::{include_wav, timer::Timer, Gba};

// Music - "Let it in" by Josh Woodward, free download at http://joshwoodward.com
const LET_IT_IN: &[u8] = include_wav!("examples/JoshWoodward-LetItIn.wav");

#[agb::entry]
fn main() -> ! {
    let mut gba = Gba::new();
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut timer_controller = gba.timer;
    timer_controller.set_overflow_amount(Timer::Timer1, u16::MAX);
    timer_controller.set_enabled(Timer::Timer1, true);
    timer_controller.set_overflow_amount(Timer::Timer2, u16::MAX);
    timer_controller.set_cascade(Timer::Timer2, true);

    let mut mixer = gba.mixer.mixer();
    mixer.enable();

    let mut channel = SoundChannel::new(LET_IT_IN);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    let mut frame_counter = 0i32;
    loop {
        vblank_provider.wait_for_vblank();
        let before_mixing_cycles_lo = timer_controller.get_value(Timer::Timer1);
        let before_mixing_cycles_hi = timer_controller.get_value(Timer::Timer2);
        mixer.vblank();
        let after_mixing_cycles_lo = timer_controller.get_value(Timer::Timer1);
        let after_mixing_cycles_hi = timer_controller.get_value(Timer::Timer2);

        frame_counter = frame_counter.wrapping_add(1);

        if frame_counter % 128 == 0 {
            let before_mixing_cycles =
                ((before_mixing_cycles_hi as u32) << 16) | before_mixing_cycles_lo as u32;
            let after_mixing_cycles =
                ((after_mixing_cycles_hi as u32) << 16) | after_mixing_cycles_lo as u32;

            let total_cycles = after_mixing_cycles.wrapping_sub(before_mixing_cycles);

            let percent = (total_cycles * 100) / 280896;
            agb::println!(
                "Took {} cycles to calculate mixer ~= {}% of total frame",
                total_cycles,
                percent
            );
        }
    }
}
