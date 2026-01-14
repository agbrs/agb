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
const MAGIC: [u8; 32] = *b"agb-test-sram___________________";
const MIN_SECTOR_SIZE: usize = 128;

static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn save_setup(gba: &mut agb::Gba) -> SaveSlotManager<TestMetadata> {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        // Already initialized, use reopen
        gba.save
            .reopen(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE, None)
            .expect("Failed to reopen SRAM")
    } else {
        // First call, use init
        gba.save
            .init_sram(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE)
            .expect("Failed to init SRAM")
    }
}

fn save_reopen(gba: &mut agb::Gba) -> SaveSlotManager<TestMetadata> {
    gba.save
        .reopen(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE, None)
        .expect("Failed to reopen SRAM")
}

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::halt();
    }
}
