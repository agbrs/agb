//! The default if there is no game written yet
#![no_std]
#![no_main]

#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    agb::no_game(gba);
}
