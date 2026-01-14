use crate::test_storage::TestStorage;
use crate::{SaveSlotManager, SlotStatus, MIN_SECTOR_SIZE};

use serde::{Deserialize, Serialize};

const TEST_GAME_MAGIC: [u8; 32] = *b"test-game-______________________";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct TestMetadata {
    name: [u8; 16],
}

/// Different metadata type for testing deserialization failure.
/// This is an enum with only 3 variants, so postcard will fail when it
/// tries to decode TestMetadata bytes (which start with 'T' = 84) as a variant index.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum IncompatibleMetadata {
    A,
    B,
    C,
}

#[test]
fn new_storage_has_empty_slots() {
    // 4KB storage, enough for several sectors
    let storage = TestStorage::new_sram(4096);

    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    assert_eq!(manager.num_slots(), 3);

    for slot in 0..3 {
        assert_eq!(
            manager.slot_status(slot),
            SlotStatus::Empty,
            "slot {slot} should be empty on fresh storage"
        );
        assert!(
            manager.metadata(slot).is_none(),
            "slot {slot} should have no metadata on fresh storage"
        );
    }
}

#[test]
fn corrupted_slot_detected_as_corrupted() {
    let storage = TestStorage::new_sram(4096);

    // Initialize storage with empty slots
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Get the storage back and corrupt slot 1's header (sector 2)
    let mut storage = manager.into_storage();

    // Corrupt a byte after the CRC (byte 0-1 is CRC, so corrupt byte 4)
    // Sector 2 starts at offset 2 * MIN_SECTOR_SIZE
    let corrupt_offset = 2 * MIN_SECTOR_SIZE + 4;
    storage.data_mut()[corrupt_offset] ^= 0xFF;

    // Re-initialize from corrupted storage
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Slot 0 and 2 should still be empty
    assert_eq!(
        manager.slot_status(0),
        SlotStatus::Empty,
        "slot 0 should still be empty"
    );
    assert_eq!(
        manager.slot_status(2),
        SlotStatus::Empty,
        "slot 2 should still be empty"
    );

    // Slot 1 should be corrupted
    assert_eq!(
        manager.slot_status(1),
        SlotStatus::Corrupted,
        "slot 1 should be detected as corrupted"
    );
}

#[test]
fn write_slot_makes_slot_valid() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // All slots start empty
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);

    // Write to slot 0
    let metadata = TestMetadata {
        name: *b"Player One______",
    };
    manager.write(0, &(), &metadata).unwrap();

    // Slot 0 should now be valid
    assert_eq!(
        manager.slot_status(0),
        SlotStatus::Valid,
        "slot should be valid after write"
    );

    // Other slots should still be empty
    assert_eq!(manager.slot_status(1), SlotStatus::Empty);
    assert_eq!(manager.slot_status(2), SlotStatus::Empty);
}

#[test]
fn write_slot_stores_metadata() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"Hero____________",
    };
    manager.write(0, &(), &metadata).unwrap();

    // Should be able to retrieve the metadata
    let retrieved = manager.metadata(0).expect("metadata should exist");
    assert_eq!(retrieved.name, *b"Hero____________");
}

#[test]
fn write_multiple_slots() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata0 = TestMetadata {
        name: *b"Save One________",
    };
    let metadata1 = TestMetadata {
        name: *b"Save Two________",
    };
    let metadata2 = TestMetadata {
        name: *b"Save Three______",
    };

    manager.write(0, &(), &metadata0).unwrap();
    manager.write(1, &(), &metadata1).unwrap();
    manager.write(2, &(), &metadata2).unwrap();

    // All slots should be valid
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    assert_eq!(manager.slot_status(1), SlotStatus::Valid);
    assert_eq!(manager.slot_status(2), SlotStatus::Valid);

    // Each slot should have correct metadata
    assert_eq!(manager.metadata(0).unwrap().name, *b"Save One________");
    assert_eq!(manager.metadata(1).unwrap().name, *b"Save Two________");
    assert_eq!(manager.metadata(2).unwrap().name, *b"Save Three______");
}

#[test]
fn write_slot_persists_across_reinit() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"Persistent______",
    };
    manager.write(1, &(), &metadata).unwrap();

    // Get storage back and reinitialize
    let storage = manager.into_storage();
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Slot 1 should still be valid with correct metadata
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
    assert_eq!(manager.slot_status(1), SlotStatus::Valid);
    assert_eq!(manager.slot_status(2), SlotStatus::Empty);

    assert_eq!(manager.metadata(1).unwrap().name, *b"Persistent______");
}

#[test]
fn incompatible_metadata_detected_as_corrupted() {
    let storage = TestStorage::new_sram(4096);

    // Write with TestMetadata
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"Test Save_______",
    };
    manager.write(0, &(), &metadata).unwrap();

    // Get storage back
    let storage = manager.into_storage();

    // Reinitialize with incompatible metadata type
    let manager: SaveSlotManager<_, IncompatibleMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Slot 0 should be corrupted because metadata can't deserialize
    assert_eq!(
        manager.slot_status(0),
        SlotStatus::Corrupted,
        "slot with incompatible metadata should be detected as corrupted"
    );
    assert!(
        manager.metadata(0).is_none(),
        "corrupted slot should have no metadata"
    );
}
