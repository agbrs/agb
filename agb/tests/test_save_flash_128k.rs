#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(agb::test_runner::test_runner)]

mod save_test_common;

use agb::save::SaveSlotManager;
use portable_atomic::{AtomicBool, Ordering};
use save_test_common::TestMetadata;

const NUM_SLOTS: usize = 3;
const MAGIC: [u8; 32] = *b"agb-test-flash128k______________";
const MIN_SECTOR_SIZE: usize = 4096;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn save_setup(gba: &mut agb::Gba) -> SaveSlotManager<TestMetadata> {
    let timers = gba.timers.timers();
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        // Already initialized, use reopen
        gba.save
            .reopen(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE, Some(timers.timer2))
            .expect("Failed to reopen Flash 128K")
    } else {
        // First call, use init
        gba.save
            .init_flash_128k(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE, Some(timers.timer2))
            .expect("Failed to init Flash 128K")
    }
}

fn save_reopen(gba: &mut agb::Gba) -> SaveSlotManager<TestMetadata> {
    let timers = gba.timers.timers();
    gba.save
        .reopen(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE, Some(timers.timer2))
        .expect("Failed to reopen Flash 128K")
}

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::halt();
    }
}
