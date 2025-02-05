#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(agb::test_runner::test_runner)]

mod save_test_common;

fn save_setup(gba: &mut agb::Gba) {
    gba.save.init_eeprom_8k();
}

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::syscall::halt();
    }
}
