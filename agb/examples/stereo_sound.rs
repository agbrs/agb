#![no_std]
#![no_main]

extern crate agb;

use agb::sound::mixer::SoundChannel;
use agb::{include_wav, Gba};

// Music - "Let it in" by Josh Woodward, free download at http://joshwoodward.com
const LET_IT_IN: &[u8] = include_wav!("examples/JoshWoodward-LetItIn.wav");

#[agb::entry]
fn main() -> ! {
    let mut gba = Gba::new();
    let vblank_provider = agb::interrupt::VBlank::get();

    let mut timer_controller = gba.timers.timers();
    let mut timer = timer_controller.timer1;
    timer.set_enabled(true);

    let mut mixer = gba.mixer.mixer(&mut timer_controller.timer0);
    mixer.enable();

    let mut channel = SoundChannel::new(LET_IT_IN);
    channel.stereo();
    mixer.play_sound(channel).unwrap();

    let mut frame_counter = 0i32;
    loop {
        vblank_provider.wait_for_vblank();
        let before_mixing_cycles = timer.get_value();
        mixer.after_vblank();
        mixer.frame();
        let after_mixing_cycles = timer.get_value();

        frame_counter = frame_counter.wrapping_add(1);

        if frame_counter % 128 == 0 {
            let total_cycles = after_mixing_cycles.wrapping_sub(before_mixing_cycles) as u32;

            let percent = (total_cycles * 100) / 280896;
            agb::println!(
                "Took {} cycles to calculate mixer ~= {}% of total frame",
                total_cycles,
                percent
            );
        }
    }
}
