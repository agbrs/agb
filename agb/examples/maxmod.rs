#![no_std]
#![no_main]

extern crate alloc;

use agb::{
    input::{Button, ButtonController},
    sound::tracker::{include_sounds, MixMode, SoundEffectOptions},
    Gba,
};

include_sounds!("examples/pk_lz0_07.xm", "examples/accept.wav");

#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let mut input = ButtonController::new();

    let vblank_provider = agb::interrupt::VBlank::get();
    let tracker = gba.mixer.tracker::<music::Music>(10, MixMode::Hz31);

    tracker.start(music::ModFiles::MOD_PK_LZ0_07);

    let mut timer = gba.timers.timers().timer2;
    timer.set_enabled(true);

    loop {
        vblank_provider.wait_for_vblank();
        input.update();

        if input.is_just_pressed(Button::A) {
            let mut sfx = SoundEffectOptions::new(music::SfxFiles::SFX_ACCEPT);
            sfx.volume(255);
            let handle = tracker.effect(sfx);

            agb::println!("Plaing 'accept' with handle {:?}", handle);
        }

        let before = timer.value();
        tracker.frame();
        let after = timer.value();

        agb::println!(
            "Took {}% cpu time",
            after.wrapping_sub(before) as u32 * 100 / 280896
        );
    }
}
