#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(agb::test_runner::test_runner)]

// Note: EEPROM 512B has very limited space (512 bytes = 4 sectors of 128 bytes)
// So we only run a single comprehensive test here that covers write, read, and persistence.

use agb::save::{SaveSlotManager, Slot};
use serde::{Deserialize, Serialize};

const NUM_SLOTS: usize = 1;
const MAGIC: [u8; 32] = *b"agb-test-eeprom512______________";
const MIN_SECTOR_SIZE: usize = 128;

/// Small metadata to fit in limited space
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SmallMetadata {
    pub level: u8,
}

/// Small save data to fit in limited space
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct SmallSaveData {
    pub score: u32,
}

#[test_case]
fn test_write_read_and_persistence(gba: &mut agb::Gba) {
    let timers = gba.timers.timers();

    // Initialize
    let mut manager: SaveSlotManager<SmallMetadata> = gba
        .save
        .init_eeprom_512b(NUM_SLOTS, MAGIC, MIN_SECTOR_SIZE, Some(timers.timer2))
        .expect("Failed to init EEPROM 512B");

    let data = SmallSaveData { score: 12345 };
    let metadata = SmallMetadata { level: 5 };

    // Write to slot 0
    manager
        .write(0, &data, &metadata)
        .expect("Failed to write save data");

    // Verify slot status
    assert_eq!(manager.slot(0), Slot::Valid(&metadata));

    // Read back and verify
    let loaded: SmallSaveData = manager.read(0).expect("Failed to read save data");
    assert_eq!(loaded, data);

    // Drop the manager and reopen to simulate game restart
    drop(manager);

    let timers = gba.timers.timers();
    let mut manager2: SaveSlotManager<SmallMetadata> = gba
        .save
        .reopen(Some(timers.timer2))
        .expect("Failed to reopen EEPROM 512B");

    // Verify data persisted after reopen
    assert_eq!(manager2.slot(0), Slot::Valid(&metadata));

    let loaded2: SmallSaveData = manager2.read(0).expect("Failed to read after reopen");
    assert_eq!(loaded2, data);
}

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::halt();
    }
}
