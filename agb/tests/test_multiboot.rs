#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(agb::test_runner::test_runner)]

fn hello() {}

#[test_case]
fn multiboot_test(_gba: &mut agb::Gba) {
    let address: usize = hello as usize;

    if option_env!("AGB_MULTIBOOT").is_some() {
        assert!(
            (0x0200_0000..0x0204_0000).contains(&address),
            "multiboot functions should all be in ewram 0x0300_0000 and 0x0300_8000, but was actually found to be at {address:#010X}"
        );
    } else {
        assert!(
            address >= 0x0800_0000,
            "functions should all be in ROM >= 0x0800_0000, but was actually found to be at {address:#010X}"
        );
    }
}

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::syscall::halt();
    }
}
