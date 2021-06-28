#![no_std]
#![no_main]

extern crate agb;

use agb::input::{Button, ButtonController, Tri};
use agb::number::Num;
use agb::sound::mixer::SoundChannel;
use agb::Gba;

// Music - "I will not let you let me down" by Josh Woodward, free download at http://joshwoodward.com
const I_WILL_NOT_LET_YOU_LET_ME_DOWN: &[u8] = include_bytes!("i-will-not-let-you-let-me-down.raw");

#[no_mangle]
pub fn main() -> ! {
    let mut gba = Gba::new();
    let mut input = ButtonController::new();
    let vblank_provider = agb::interrupt::VBlank::new();

    let mut mixer = gba.mixer.mixer();
    mixer.enable();

    let channel = SoundChannel::new(I_WILL_NOT_LET_YOU_LET_ME_DOWN);
    let channel_id = mixer.play_sound(channel).unwrap();

    loop {
        input.update();

        {
            if let Some(channel) = mixer.get_channel(&channel_id) {
                let half: Num<i16, 4> = Num::new(1) / 2;
                let half_usize: Num<usize, 8> = Num::new(1) / 2;
                match input.x_tri() {
                    Tri::Negative => channel.panning(-half),
                    Tri::Zero => channel.panning(0.into()),
                    Tri::Positive => channel.panning(half),
                };

                match input.y_tri() {
                    Tri::Negative => channel.playback(half_usize.change_base() + 1),
                    Tri::Zero => channel.playback(1.into()),
                    Tri::Positive => channel.playback(half_usize),
                };

                if input.is_pressed(Button::L) {
                    channel.volume(half);
                } else {
                    channel.volume(1.into());
                }
            }
        }

        vblank_provider.wait_for_vblank();
        mixer.vblank();
    }
}
