#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(agb::test_runner::test_runner)]

// Dummy test for now to satisfy clippy --tests flag
// TODO: Add actual tests

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::halt();
    }
}
