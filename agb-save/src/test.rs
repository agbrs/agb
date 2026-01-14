extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use quickcheck::{Arbitrary, Gen, quickcheck};

use crate::test_storage::TestStorage;
use crate::{MIN_SECTOR_SIZE, SaveSlotManager, SlotStatus};

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct TestSaveData {
    level: u32,
    health: u32,
    position: (i32, i32),
    inventory: Vec<u8>,
}

#[test]
fn write_and_read_data_roundtrip() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"Hero____________",
    };
    let save_data = TestSaveData {
        level: 42,
        health: 100,
        position: (123, -456),
        inventory: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    };

    manager.write(0, &save_data, &metadata).unwrap();

    // Read back
    let loaded: TestSaveData = manager.read(0).unwrap();
    assert_eq!(loaded, save_data);
}

#[test]
fn write_and_read_persists_across_reinit() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"Persistent______",
    };
    let save_data = TestSaveData {
        level: 99,
        health: 255,
        position: (1000, 2000),
        inventory: vec![0xFF; 64],
    };

    manager.write(1, &save_data, &metadata).unwrap();

    // Reinitialize from storage
    let storage = manager.into_storage();
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Data should persist
    assert_eq!(manager.slot_status(1), SlotStatus::Valid);
    let loaded: TestSaveData = manager.read(1).unwrap();
    assert_eq!(loaded, save_data);
}

#[test]
fn write_large_data_spans_multiple_blocks() {
    // Use larger storage to fit multi-block data
    let storage = TestStorage::new_sram(8192);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"BigSave_________",
    };

    // Create data larger than one block's payload (128 - 8 = 120 bytes per block)
    let large_data: Vec<u8> = (0..500).map(|i| (i % 256) as u8).collect();

    manager.write(0, &large_data, &metadata).unwrap();

    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, large_data);
}

#[test]
fn multiple_writes_to_same_slot() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata1 = TestMetadata {
        name: *b"First___________",
    };
    let data1 = TestSaveData {
        level: 1,
        health: 50,
        position: (0, 0),
        inventory: vec![],
    };

    manager.write(0, &data1, &metadata1).unwrap();

    // Write again to the same slot
    let metadata2 = TestMetadata {
        name: *b"Second__________",
    };
    let data2 = TestSaveData {
        level: 10,
        health: 100,
        position: (100, 200),
        inventory: vec![1, 2, 3],
    };

    manager.write(0, &data2, &metadata2).unwrap();

    // Should have the second version
    assert_eq!(manager.metadata(0).unwrap().name, *b"Second__________");
    let loaded: TestSaveData = manager.read(0).unwrap();
    assert_eq!(loaded, data2);
}

#[test]
fn erase_slot_makes_slot_empty() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"ToBeErased______",
    };
    let save_data = TestSaveData {
        level: 50,
        health: 100,
        position: (0, 0),
        inventory: vec![1, 2, 3, 4, 5],
    };

    manager.write(0, &save_data, &metadata).unwrap();
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);

    // Erase the slot
    manager.erase(0).unwrap();

    // Slot should now be empty
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
    assert!(manager.metadata(0).is_none());
}

#[test]
fn erase_slot_persists_across_reinit() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"ToBeErased______",
    };
    let save_data = TestSaveData {
        level: 50,
        health: 100,
        position: (0, 0),
        inventory: vec![1, 2, 3],
    };

    manager.write(0, &save_data, &metadata).unwrap();
    manager.erase(0).unwrap();

    // Reinitialize
    let storage = manager.into_storage();
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Slot should still be empty
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
}

#[test]
fn erase_slot_frees_space_for_new_write() {
    // Small storage to test space reclamation
    let storage = TestStorage::new_sram(2048);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"BigData_________",
    };
    // Write data that takes up several blocks
    let large_data: Vec<u8> = (0..300).map(|i| (i % 256) as u8).collect();

    manager.write(0, &large_data, &metadata).unwrap();

    // Erase the slot to free up space
    manager.erase(0).unwrap();

    // Should be able to write again using the freed space
    let new_data: Vec<u8> = (0..300).map(|i| (255 - i % 256) as u8).collect();
    manager.write(0, &new_data, &metadata).unwrap();

    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, new_data);
}

#[test]
fn crash_during_first_write_leaves_slot_empty() {
    let storage = TestStorage::new_sram(4096);

    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Configure failure after initialization but before user write completes
    let mut storage = manager.into_storage();
    storage.fail_after_writes(Some(1)); // Fail after first write (during data block writing)

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"CrashTest_______",
    };
    let save_data: Vec<u8> = (0..200).map(|i| i as u8).collect();

    // This write should fail partway through
    let result = manager.write(0, &save_data, &metadata);
    assert!(result.is_err());

    // Get the storage back and reinitialize
    let storage = manager.into_storage();
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Slot should still be empty since the write didn't complete
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
}

#[test]
fn crash_during_overwrite_preserves_old_data() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // First, write some valid data
    let metadata1 = TestMetadata {
        name: *b"Original________",
    };
    let data1: Vec<u8> = vec![1, 2, 3, 4, 5];
    manager.write(0, &data1, &metadata1).unwrap();

    // Verify it's there
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);

    // Now get storage and configure to fail during the next write
    let mut storage = manager.into_storage();
    storage.fail_after_writes(Some(1)); // Fail early in the write process

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Verify old data is still there after reinit
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, data1);

    // Try to overwrite with new data - this should fail
    let metadata2 = TestMetadata {
        name: *b"NewData_________",
    };
    let data2: Vec<u8> = vec![10, 20, 30, 40, 50];
    let result = manager.write(0, &data2, &metadata2);
    assert!(result.is_err());

    // Reinitialize and verify old data is still intact
    let storage = manager.into_storage();
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    assert_eq!(manager.metadata(0).unwrap().name, *b"Original________");
    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, data1);
}

#[test]
fn crash_during_large_write_preserves_old_data() {
    let storage = TestStorage::new_sram(8192);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write initial large data
    let metadata1 = TestMetadata {
        name: *b"LargeOriginal___",
    };
    let data1: Vec<u8> = (0..500).map(|i| (i % 256) as u8).collect();
    manager.write(0, &data1, &metadata1).unwrap();

    // Get storage and configure to fail midway through the large write
    let mut storage = manager.into_storage();
    storage.fail_after_writes(Some(3)); // Fail after a few data blocks

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Try to overwrite with different large data
    let metadata2 = TestMetadata {
        name: *b"LargeNew________",
    };
    let data2: Vec<u8> = (0..500).map(|i| (255 - i % 256) as u8).collect();
    let result = manager.write(0, &data2, &metadata2);
    assert!(result.is_err());

    // Reinitialize and verify old data is preserved
    let storage = manager.into_storage();
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    assert_eq!(manager.metadata(0).unwrap().name, *b"LargeOriginal___");
    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, data1);
}

#[test]
fn corrupted_header_recovers_from_ghost() {
    // This test simulates a crash that happens AFTER writing the new header
    // but BEFORE marking the old header as ghost and freeing old data.
    // In this case, we have two VALID headers and should pick the newest one.
    // If the newest one is corrupted, we should fall back to the ghost.

    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write initial data
    let metadata1 = TestMetadata {
        name: *b"FirstVersion____",
    };
    let data1: Vec<u8> = vec![1, 2, 3, 4, 5];
    manager.write(0, &data1, &metadata1).unwrap();

    // Get storage and set up to crash during second write
    // We want to crash AFTER writing new header but BEFORE marking old as ghost
    let mut storage = manager.into_storage();

    // After first write:
    // - Slot 0 header is at sector 4 (was ghost), state=VALID
    // - Ghost sector is now sector 1
    // - Data blocks at sectors 5+

    // Configure to fail after several writes (data blocks + new header, but before ghost marking)
    // Data is small so 1 data block + 1 header = 2 writes, fail on 3rd (marking ghost)
    storage.fail_after_writes(Some(2));

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Try second write - will fail after writing new header
    let metadata2 = TestMetadata {
        name: *b"SecondVersion___",
    };
    let data2: Vec<u8> = vec![10, 20, 30, 40, 50];
    let result = manager.write(0, &data2, &metadata2);
    assert!(result.is_err());

    // Get storage and corrupt the NEW header that was written before crash
    // The new header was written to the ghost sector (sector 1)
    let mut storage = manager.into_storage();
    let new_header_sector = 1; // The ghost sector where new header was written
    let corrupt_offset = new_header_sector * MIN_SECTOR_SIZE + 4;
    storage.data_mut()[corrupt_offset] ^= 0xFF;

    // Reinitialize - should recover from the old valid header (first version)
    // since the new header is corrupted
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Should have recovered with the first version's data
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    assert_eq!(manager.metadata(0).unwrap().name, *b"FirstVersion____");
    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, data1);
}

#[test]
fn corrupted_valid_header_recovers_from_ghost_state() {
    // This test verifies recovery from a GHOST state header when the VALID header is corrupted.
    // Scenario: crash happens AFTER marking old header as GHOST but BEFORE freeing old data.

    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write initial data
    let metadata1 = TestMetadata {
        name: *b"FirstVersion____",
    };
    let data1: Vec<u8> = vec![1, 2, 3, 4, 5];
    manager.write(0, &data1, &metadata1).unwrap();

    // Set up to crash after marking ghost but before freeing data
    // Write order: data blocks, new header, mark old as ghost, free old data
    // For small data: 1 data block + 1 new header + 1 ghost marking = 3 writes
    let mut storage = manager.into_storage();
    storage.fail_after_writes(Some(3)); // Fail on freeing data (4th write would be next op)

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Second write - will complete marking ghost but fail on freeing
    let metadata2 = TestMetadata {
        name: *b"SecondVersion___",
    };
    let data2: Vec<u8> = vec![10, 20, 30, 40, 50];
    let _result = manager.write(0, &data2, &metadata2);
    // This might succeed or fail depending on when exactly freeing happens

    // Now corrupt the NEW (VALID) header
    // After first write, header was at sector 4 (old ghost)
    // After second write attempt, new header is at sector 1
    let mut storage = manager.into_storage();
    let new_header_sector = 1;
    let corrupt_offset = new_header_sector * MIN_SECTOR_SIZE + 4;
    storage.data_mut()[corrupt_offset] ^= 0xFF;

    // Reinitialize - the VALID header at sector 1 is corrupted
    // The GHOST header at sector 4 should be used for recovery
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Should recover from GHOST header with first version's data
    assert_eq!(
        manager.slot_status(0),
        SlotStatus::Valid,
        "slot should be valid after ghost recovery"
    );
    assert_eq!(
        manager.metadata(0).unwrap().name,
        *b"FirstVersion____",
        "should have first version's metadata from ghost"
    );
    let loaded: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(loaded, data1, "should have first version's data from ghost");
}

// --- Property-based tests ---

/// Arbitrary metadata for property tests
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct ArbitraryMetadata {
    values: [u8; 8],
}

impl Arbitrary for ArbitraryMetadata {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut values = [0u8; 8];
        for v in &mut values {
            *v = u8::arbitrary(g);
        }
        Self { values }
    }
}

/// Constrained data size for property tests to avoid running out of storage space.
/// Max ~400 bytes ensures we can fit data + headers in 4KB storage with 3 slots.
#[derive(Clone, Debug)]
struct BoundedData(Vec<u8>);

impl Arbitrary for BoundedData {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = usize::arbitrary(g) % 400;
        let data: Vec<u8> = (0..len).map(|_| u8::arbitrary(g)).collect();
        Self(data)
    }
}

quickcheck! {
    /// Any data written to a slot should read back identically.
    fn data_roundtrip(data: BoundedData, metadata: ArbitraryMetadata, slot: u8) -> bool {
        let slot = (slot % 3) as usize;
        let storage = TestStorage::new_sram(4096);

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        manager.write(slot, &data.0, &metadata).unwrap();

        let loaded: Vec<u8> = manager.read(slot).unwrap();
        loaded == data.0
    }

    /// Metadata written to a slot should be retrievable and match.
    fn metadata_roundtrip(data: BoundedData, metadata: ArbitraryMetadata, slot: u8) -> bool {
        let slot = (slot % 3) as usize;
        let storage = TestStorage::new_sram(4096);

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        manager.write(slot, &data.0, &metadata).unwrap();

        manager.metadata(slot) == Some(&metadata)
    }

    /// Data and metadata should survive storage reinitialisation.
    fn persistence_roundtrip(data: BoundedData, metadata: ArbitraryMetadata, slot: u8) -> bool {
        let slot = (slot % 3) as usize;
        let storage = TestStorage::new_sram(4096);

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        manager.write(slot, &data.0, &metadata).unwrap();

        // Reinitialise from storage
        let storage = manager.into_storage();
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        let status_ok = manager.slot_status(slot) == SlotStatus::Valid;
        let metadata_ok = manager.metadata(slot) == Some(&metadata);
        let data_ok = manager.read::<Vec<u8>>(slot).unwrap() == data.0;

        status_ok && metadata_ok && data_ok
    }

    /// Empty data should roundtrip correctly.
    fn empty_data_roundtrip(metadata: ArbitraryMetadata, slot: u8) -> bool {
        let slot = (slot % 3) as usize;
        let storage = TestStorage::new_sram(4096);

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        let empty: Vec<u8> = vec![];
        manager.write(slot, &empty, &metadata).unwrap();

        let loaded: Vec<u8> = manager.read(slot).unwrap();
        loaded.is_empty()
    }

    /// Writing to one slot should not affect other slots.
    fn slot_isolation(
        data0: BoundedData, meta0: ArbitraryMetadata,
        data1: BoundedData, meta1: ArbitraryMetadata,
        data2: BoundedData, meta2: ArbitraryMetadata
    ) -> bool {
        let storage = TestStorage::new_sram(8192); // Larger storage for 3 slots of data

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        manager.write(0, &data0.0, &meta0).unwrap();
        manager.write(1, &data1.0, &meta1).unwrap();
        manager.write(2, &data2.0, &meta2).unwrap();

        let read0: Vec<u8> = manager.read(0).unwrap();
        let read1: Vec<u8> = manager.read(1).unwrap();
        let read2: Vec<u8> = manager.read(2).unwrap();

        read0 == data0.0 && read1 == data1.0 && read2 == data2.0
            && manager.metadata(0) == Some(&meta0)
            && manager.metadata(1) == Some(&meta1)
            && manager.metadata(2) == Some(&meta2)
    }

    /// Overwriting a slot multiple times should always return the last written data.
    fn overwrite_returns_latest(writes: Vec<(BoundedData, ArbitraryMetadata)>) -> bool {
        if writes.is_empty() {
            return true;
        }

        let storage = TestStorage::new_sram(4096);
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        let mut last_data = vec![];
        let mut last_meta = None;

        for (data, meta) in &writes {
            if manager.write(0, &data.0, meta).is_ok() {
                last_data = data.0.clone();
                last_meta = Some(meta.clone());
            }
        }

        if last_meta.is_none() {
            // No successful writes, slot should still be empty
            return manager.slot_status(0) == SlotStatus::Empty;
        }

        let read: Vec<u8> = manager.read(0).unwrap();
        read == last_data && manager.metadata(0) == last_meta.as_ref()
    }
}

// --- Crash recovery property tests ---

/// Test that repeated crash-recovery cycles don't corrupt data.
///
/// Previously a bug existed where after multiple crash cycles, the free sector
/// list could become incorrect, causing new writes to overwrite data blocks
/// that were still in use. This was fixed by properly selecting the ghost sector
/// after recovery when no explicit Ghost state header was found.
#[test]
fn repeated_crash_does_not_corrupt_data() {
    let initial_data: Vec<u8> = vec![
        0, 64, 104, 108, 94, 201, 231, 63, 150, 56, 87, 38, 129, 101, 60, 0, 238, 224, 157, 53,
        134, 179, 162, 150, 108, 98, 19,
    ];
    let initial_meta = ArbitraryMetadata {
        values: [246, 169, 155, 0, 65, 36, 1, 100],
    };
    let crash_points: Vec<u8> = vec![2, 92, 3];

    let storage = TestStorage::new_sram(4096);
    let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    manager.write(0, &initial_data, &initial_meta).unwrap();

    let mut valid_data_versions: Vec<(Vec<u8>, ArbitraryMetadata)> = vec![];
    valid_data_versions.push((initial_data.clone(), initial_meta.clone()));

    for (i, &fail_point) in crash_points.iter().enumerate() {
        let fail_after = (fail_point % 15) as usize;

        let mut storage = manager.into_storage();
        storage.fail_after_writes(Some(fail_after));

        manager = SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        let new_data: Vec<u8> = (0..((i + 1) * 10).min(300))
            .map(|j| ((i + j) % 256) as u8)
            .collect();
        let new_meta = ArbitraryMetadata {
            values: [(i as u8).wrapping_add(1); 8],
        };
        valid_data_versions.push((new_data.clone(), new_meta.clone()));

        let _ = manager.write(0, &new_data, &new_meta);

        let storage = manager.into_storage();
        manager = SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        assert_eq!(manager.slot_status(0), SlotStatus::Valid);

        let read_data: Vec<u8> = manager.read(0).expect("data should be readable");
        let read_meta = manager.metadata(0).unwrap();

        let is_valid_version = valid_data_versions
            .iter()
            .any(|(d, m)| d == &read_data && m == read_meta);

        assert!(is_valid_version);
    }
}

quickcheck! {
    /// Crash during first write should leave slot empty (never corrupted).
    fn crash_during_first_write_leaves_empty(
        data: BoundedData,
        metadata: ArbitraryMetadata,
        fail_point: u8
    ) -> bool {
        // fail_point determines when to crash (0-255 maps to different write counts)
        let fail_after = (fail_point % 20) as usize; // Reasonable range for write count

        let storage = TestStorage::new_sram(4096);
        let manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        let mut storage = manager.into_storage();
        storage.fail_after_writes(Some(fail_after));

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Attempt write - may fail
        let _ = manager.write(0, &data.0, &metadata);

        // Reinitialise and check state
        let storage = manager.into_storage();
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Slot must be either Empty (write didn't complete) or Valid (write completed)
        // It must NEVER be Corrupted
        match manager.slot_status(0) {
            SlotStatus::Empty => true,
            SlotStatus::Valid => {
                // If valid, data must be readable and correct
                manager.read::<Vec<u8>>(0).ok() == Some(data.0.clone())
            }
            SlotStatus::Corrupted => false, // This is the failure case
        }
    }

    /// Crash during overwrite should preserve old data or complete new data (never corrupt).
    fn crash_during_overwrite_preserves_integrity(
        old_data: BoundedData,
        old_meta: ArbitraryMetadata,
        new_data: BoundedData,
        new_meta: ArbitraryMetadata,
        fail_point: u8
    ) -> bool {
        let fail_after = (fail_point % 20) as usize;

        let storage = TestStorage::new_sram(4096);
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // First write should succeed
        manager.write(0, &old_data.0, &old_meta).unwrap();

        // Set up crash for second write
        let mut storage = manager.into_storage();
        storage.fail_after_writes(Some(fail_after));

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Attempt second write - may fail
        let _ = manager.write(0, &new_data.0, &new_meta);

        // Reinitialise and check state
        let storage = manager.into_storage();
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Slot must be Valid (either old or new data) - never Corrupted or Empty
        match manager.slot_status(0) {
            SlotStatus::Valid => {
                // Data must be either old or new - never partial/mixed
                let read_data: Vec<u8> = match manager.read(0) {
                    Ok(d) => d,
                    Err(_) => return false,
                };
                let read_meta = manager.metadata(0);

                let is_old = read_data == old_data.0 && read_meta == Some(&old_meta);
                let is_new = read_data == new_data.0 && read_meta == Some(&new_meta);

                is_old || is_new
            }
            SlotStatus::Corrupted | SlotStatus::Empty => false,
        }
    }

    /// Crash during write to one slot should not affect other slots.
    fn crash_does_not_affect_other_slots(
        data0: BoundedData, meta0: ArbitraryMetadata,
        data1: BoundedData, meta1: ArbitraryMetadata,
        new_data1: BoundedData, new_meta1: ArbitraryMetadata,
        fail_point: u8
    ) -> bool {
        let fail_after = (fail_point % 20) as usize;

        let storage = TestStorage::new_sram(8192);
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Write to both slots successfully
        manager.write(0, &data0.0, &meta0).unwrap();
        manager.write(1, &data1.0, &meta1).unwrap();

        // Set up crash for write to slot 1
        let mut storage = manager.into_storage();
        storage.fail_after_writes(Some(fail_after));

        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Attempt write to slot 1 - may fail
        let _ = manager.write(1, &new_data1.0, &new_meta1);

        // Reinitialise and check slot 0 is unaffected
        let storage = manager.into_storage();
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Slot 0 must be completely unaffected
        if manager.slot_status(0) != SlotStatus::Valid {
            return false;
        }
        let read0: Vec<u8> = match manager.read(0) {
            Ok(d) => d,
            Err(_) => return false,
        };
        if read0 != data0.0 || manager.metadata(0) != Some(&meta0) {
            return false;
        }

        // Slot 1 must be valid (old or new data)
        match manager.slot_status(1) {
            SlotStatus::Valid => {
                let read1: Vec<u8> = match manager.read(1) {
                    Ok(d) => d,
                    Err(_) => return false,
                };
                let is_old = read1 == data1.0 && manager.metadata(1) == Some(&meta1);
                let is_new = read1 == new_data1.0 && manager.metadata(1) == Some(&new_meta1);
                is_old || is_new
            }
            _ => false,
        }
    }

    /// Repeated crashes during writes should never corrupt data.
    /// After any sequence of crashes, the slot should contain one of the
    /// successfully written versions.
    fn repeated_crash_recovery_never_corrupts(
        initial_data: BoundedData,
        initial_meta: ArbitraryMetadata,
        crash_points: Vec<u8>
    ) -> bool {
        // Limit crash iterations to avoid very long tests
        let crash_points: Vec<_> = crash_points.into_iter().take(5).collect();

        let storage = TestStorage::new_sram(4096);
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Initial successful write
        if manager.write(0, &initial_data.0, &initial_meta).is_err() {
            return true; // Data too large for storage, skip test
        }

        // Track all valid versions of data that could exist
        let mut valid_versions: Vec<(Vec<u8>, ArbitraryMetadata)> = vec![];
        valid_versions.push((initial_data.0.clone(), initial_meta.clone()));

        for (i, &fail_point) in crash_points.iter().enumerate() {
            let fail_after = (fail_point % 15) as usize;

            // Set up crash
            let mut storage = manager.into_storage();
            storage.fail_after_writes(Some(fail_after));

            manager = SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

            // Generate new data for this write attempt
            let new_data: Vec<u8> = (0..((i + 1) * 10).min(300))
                .map(|j| ((i + j) % 256) as u8)
                .collect();
            let new_meta = ArbitraryMetadata {
                values: [(i as u8).wrapping_add(1); 8],
            };

            // Track this as a potential valid version (if write completes)
            valid_versions.push((new_data.clone(), new_meta.clone()));

            // Attempt write - may fail at crash point
            let _ = manager.write(0, &new_data, &new_meta);

            // Reinitialise (simulates power cycle after crash)
            let storage = manager.into_storage();
            manager = SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

            // Slot must be Valid - never Corrupted or Empty after crash
            if manager.slot_status(0) != SlotStatus::Valid {
                return false;
            }

            // Data must match one of the valid versions
            let read_data: Vec<u8> = match manager.read(0) {
                Ok(d) => d,
                Err(_) => return false,
            };
            let read_meta = manager.metadata(0).unwrap();

            let is_valid_version = valid_versions
                .iter()
                .any(|(d, m)| d == &read_data && m == read_meta);

            if !is_valid_version {
                return false;
            }
        }

        true
    }

}

/// Extensive stress test: random saves to random slots with random sizes,
/// reloading manager after each batch and verifying all data.
#[test]
fn stress_test_random_saves_and_reloads() {
    use quickcheck::{Gen, QuickCheck, TestResult};

    fn prop(seed: u64) -> TestResult {
        let mut rng = Gen::new(256);
        // Use seed to make the test deterministic for a given input
        for _ in 0..(seed % 100) {
            let _: u8 = Arbitrary::arbitrary(&mut rng);
        }

        const NUM_SLOTS: usize = 3;
        // 16KB storage with max 500 byte saves ensures we never hit capacity
        let storage = TestStorage::new_sram(16384);
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, NUM_SLOTS, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Track what data each slot should contain
        let mut expected: Vec<Option<(Vec<u8>, ArbitraryMetadata)>> = vec![None; NUM_SLOTS];

        // Run many cycles
        for _cycle in 0..50 {
            // Random number of saves this cycle (1-10)
            let num_saves: usize = (u8::arbitrary(&mut rng) % 10) as usize + 1;

            for _ in 0..num_saves {
                // Pick random slot
                let slot = (u8::arbitrary(&mut rng) % NUM_SLOTS as u8) as usize;

                // Generate random-sized data (0 to 500 bytes to vary block count)
                let data_size = (u16::arbitrary(&mut rng) % 501) as usize;
                let data: Vec<u8> = (0..data_size).map(|_| u8::arbitrary(&mut rng)).collect();

                let metadata = ArbitraryMetadata::arbitrary(&mut rng);

                // Write must succeed - storage is sized to never run out of space
                if manager.write(slot, &data, &metadata).is_err() {
                    return TestResult::failed();
                }
                expected[slot] = Some((data, metadata));
            }

            // Reload the manager
            let storage = manager.into_storage();
            manager =
                SaveSlotManager::new(storage, NUM_SLOTS, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

            // Verify all slots match expected state
            for (slot, expected_data) in expected.iter().enumerate() {
                match expected_data {
                    None => {
                        if manager.slot_status(slot) != SlotStatus::Empty {
                            return TestResult::failed();
                        }
                    }
                    Some((data, meta)) => {
                        if manager.slot_status(slot) != SlotStatus::Valid {
                            return TestResult::failed();
                        }
                        let read_data: Vec<u8> = match manager.read(slot) {
                            Ok(d) => d,
                            Err(_) => return TestResult::failed(),
                        };
                        if &read_data != data {
                            return TestResult::failed();
                        }
                        if manager.metadata(slot) != Some(meta) {
                            return TestResult::failed();
                        }
                    }
                }
            }
        }

        TestResult::passed()
    }

    // Run with many different seeds for thorough coverage
    QuickCheck::new()
        .tests(1000)
        .quickcheck(prop as fn(u64) -> TestResult);
}

/// Stress test with random write failures to exercise crash recovery.
/// After each failure, verifies slots contain either old or new data (never corrupted).
#[test]
fn stress_test_with_random_failures() {
    use quickcheck::{Gen, QuickCheck, TestResult};

    fn prop(seed: u64) -> TestResult {
        let mut rng = Gen::new(256);
        for _ in 0..(seed % 100) {
            let _: u8 = Arbitrary::arbitrary(&mut rng);
        }

        const NUM_SLOTS: usize = 3;
        let storage = TestStorage::new_sram(16384);
        let mut manager: SaveSlotManager<_, ArbitraryMetadata> =
            SaveSlotManager::new(storage, NUM_SLOTS, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

        // Track current confirmed data and pending (possibly committed) data for each slot
        let mut current: Vec<Option<(Vec<u8>, ArbitraryMetadata)>> = vec![None; NUM_SLOTS];

        for _cycle in 0..50 {
            let num_saves: usize = (u8::arbitrary(&mut rng) % 10) as usize + 1;

            // Set up random failure point for this cycle
            let fail_after: usize = (u8::arbitrary(&mut rng) % 100) as usize + 1;
            let mut storage = manager.into_storage();
            storage.fail_after_writes(Some(fail_after));
            manager =
                SaveSlotManager::new(storage, NUM_SLOTS, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

            // Pending tracks writes that might have partially committed
            let mut pending = current.clone();

            for _ in 0..num_saves {
                let slot = (u8::arbitrary(&mut rng) % NUM_SLOTS as u8) as usize;
                let data_size = (u16::arbitrary(&mut rng) % 501) as usize;
                let data: Vec<u8> = (0..data_size).map(|_| u8::arbitrary(&mut rng)).collect();
                let metadata = ArbitraryMetadata::arbitrary(&mut rng);

                let new_data = (data.clone(), metadata.clone());

                match manager.write(slot, &data, &metadata) {
                    Ok(()) => {
                        // Write succeeded, update both current and pending
                        current[slot] = Some(new_data.clone());
                        pending[slot] = Some(new_data);
                    }
                    Err(_) => {
                        // Write failed - but it might have partially committed
                        // Record as pending (might be valid after reload)
                        pending[slot] = Some(new_data);
                        break; // Stop this cycle, writes will keep failing
                    }
                }
            }

            // Reload without failure injection
            let mut storage = manager.into_storage();
            storage.fail_after_writes(None);
            manager =
                SaveSlotManager::new(storage, NUM_SLOTS, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

            // Verify each slot has valid data (either current or pending, never corrupted)
            for slot in 0..NUM_SLOTS {
                match manager.slot_status(slot) {
                    SlotStatus::Empty => {
                        // Valid if both current and pending are None
                        if current[slot].is_some() || pending[slot].is_some() {
                            // Could still be okay if pending was a failed first write
                            if current[slot].is_some() {
                                return TestResult::failed();
                            }
                            // pending was set but slot is empty - write failed before commit
                            // This is fine, clear pending
                            pending[slot] = None;
                        }
                    }
                    SlotStatus::Valid => {
                        let read_data: Vec<u8> = match manager.read(slot) {
                            Ok(d) => d,
                            Err(_) => return TestResult::failed(),
                        };
                        let read_meta = manager.metadata(slot).unwrap();

                        // Check if it matches current
                        let matches_current = current[slot]
                            .as_ref()
                            .map(|(d, m)| d == &read_data && m == read_meta)
                            .unwrap_or(false);

                        // Check if it matches pending
                        let matches_pending = pending[slot]
                            .as_ref()
                            .map(|(d, m)| d == &read_data && m == read_meta)
                            .unwrap_or(false);

                        if !matches_current && !matches_pending {
                            return TestResult::failed();
                        }

                        // Update current to reflect actual state
                        current[slot] = Some((read_data, read_meta.clone()));
                    }
                    SlotStatus::Corrupted => {
                        // Never acceptable
                        return TestResult::failed();
                    }
                }
            }
        }

        TestResult::passed()
    }

    QuickCheck::new()
        .tests(1000)
        .quickcheck(prop as fn(u64) -> TestResult);
}

// --- Flash storage tests ---
// GBA flash is 64KB with 4KB erase blocks

const GBA_FLASH_SIZE: usize = 64 * 1024; // 64KB
const GBA_FLASH_ERASE_SIZE: usize = 4096; // 4KB erase blocks
const GBA_FLASH_WRITE_SIZE: usize = 1; // Byte-addressable writes

#[test]
fn flash_storage_basic_roundtrip() {
    let storage =
        TestStorage::new_flash(GBA_FLASH_SIZE, GBA_FLASH_ERASE_SIZE, GBA_FLASH_WRITE_SIZE);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    // Write and read back
    let metadata = TestMetadata {
        name: *b"FlashTest_______",
    };
    let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6, 7, 8];

    manager.write(0, &data, &metadata).unwrap();

    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn flash_storage_persists_across_reinit() {
    let storage =
        TestStorage::new_flash(GBA_FLASH_SIZE, GBA_FLASH_ERASE_SIZE, GBA_FLASH_WRITE_SIZE);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"FlashPersist____",
    };
    let data: Vec<u8> = vec![10, 20, 30, 40];

    manager.write(0, &data, &metadata).unwrap();

    // Reinitialize
    let storage = manager.into_storage();
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn flash_storage_multiple_writes_same_slot() {
    let storage =
        TestStorage::new_flash(GBA_FLASH_SIZE, GBA_FLASH_ERASE_SIZE, GBA_FLASH_WRITE_SIZE);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    // Multiple overwrites to same slot on flash
    for i in 0..5 {
        let metadata = TestMetadata {
            name: *b"FlashOverwrite__",
        };
        let data: Vec<u8> = vec![i as u8; 20];

        manager.write(0, &data, &metadata).unwrap();

        let read_data: Vec<u8> = manager.read(0).unwrap();
        assert_eq!(read_data, data);
    }
}

#[test]
fn flash_storage_crash_recovery() {
    let storage =
        TestStorage::new_flash(GBA_FLASH_SIZE, GBA_FLASH_ERASE_SIZE, GBA_FLASH_WRITE_SIZE);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    // Write initial data
    let metadata1 = TestMetadata {
        name: *b"FlashCrash1_____",
    };
    let data1: Vec<u8> = vec![1, 2, 3, 4];
    manager.write(0, &data1, &metadata1).unwrap();

    // Set up to crash during second write
    let mut storage = manager.into_storage();
    storage.fail_after_writes(Some(1)); // Crash after first write (data block)

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    let metadata2 = TestMetadata {
        name: *b"FlashCrash2_____",
    };
    let data2: Vec<u8> = vec![5, 6, 7, 8];
    let _ = manager.write(0, &data2, &metadata2);

    // Reinitialize and verify recovery
    let storage = manager.into_storage();
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    // Should have original data (crash during second write)
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    let read_data: Vec<u8> = manager.read(0).unwrap();
    // Either old or new data is acceptable
    assert!(read_data == data1 || read_data == data2);
}

#[test]
fn flash_storage_large_save_data() {
    let storage =
        TestStorage::new_flash(GBA_FLASH_SIZE, GBA_FLASH_ERASE_SIZE, GBA_FLASH_WRITE_SIZE);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, GBA_FLASH_ERASE_SIZE).unwrap();

    // Write a larger save that spans multiple 4KB sectors
    let metadata = TestMetadata {
        name: *b"LargeSave_______",
    };
    let data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

    manager.write(0, &data, &metadata).unwrap();

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

// --- Storage capacity limits tests ---

#[test]
fn out_of_space_returns_error() {
    // Small storage: 1KB total
    // With 3 slots + global + ghost = 5 header sectors
    // Minimal space for data
    let storage = TestStorage::new_sram(1024);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let metadata = TestMetadata {
        name: *b"TooBig__________",
    };
    // Try to write more data than storage can hold
    let large_data: Vec<u8> = vec![0xAB; 2000];

    let result = manager.write(0, &large_data, &metadata);
    assert!(
        matches!(result, Err(crate::SaveError::OutOfSpace)),
        "expected OutOfSpace error, got {:?}",
        result
    );
}

#[test]
fn out_of_space_does_not_corrupt_existing_data() {
    let storage = TestStorage::new_sram(2048);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write some data first
    let metadata1 = TestMetadata {
        name: *b"ExistingData____",
    };
    let data1: Vec<u8> = vec![1, 2, 3, 4, 5];
    manager.write(0, &data1, &metadata1).unwrap();

    // Try to write data that's too large for remaining space
    let metadata2 = TestMetadata {
        name: *b"TooBigData______",
    };
    let large_data: Vec<u8> = vec![0xFF; 2000];
    let result = manager.write(1, &large_data, &metadata2);
    assert!(matches!(result, Err(crate::SaveError::OutOfSpace)));

    // Original data should still be valid
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data1);
}

#[test]
fn fill_storage_then_overwrite() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 1, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Calculate approximate max data size
    // 4096 bytes total, ~3 sectors for headers (global + slot + ghost)
    // Remaining sectors for data, each sector is MIN_SECTOR_SIZE
    let metadata = TestMetadata {
        name: *b"FillStorage_____",
    };

    // Fill with moderately large data
    let data1: Vec<u8> = vec![0x11; 500];
    manager.write(0, &data1, &metadata).unwrap();

    // Overwrite with different data (should reuse freed sectors)
    let data2: Vec<u8> = vec![0x22; 500];
    manager.write(0, &data2, &metadata).unwrap();

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data2);
}

#[test]
fn multiple_slots_approaching_capacity() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write to all 3 slots
    for slot in 0..3 {
        let metadata = TestMetadata {
            name: *b"MultiSlotCap____",
        };
        let data: Vec<u8> = vec![slot as u8; 100];
        manager.write(slot, &data, &metadata).unwrap();
    }

    // Verify all slots are valid
    for slot in 0..3 {
        assert_eq!(manager.slot_status(slot), SlotStatus::Valid);
        let read_data: Vec<u8> = manager.read(slot).unwrap();
        assert_eq!(read_data, vec![slot as u8; 100]);
    }
}

// --- Global header corruption tests ---

#[test]
fn corrupted_global_header_causes_reformat() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write some data
    let metadata = TestMetadata {
        name: *b"WillBeLost______",
    };
    let data: Vec<u8> = vec![1, 2, 3, 4];
    manager.write(0, &data, &metadata).unwrap();

    // Corrupt the global header (sector 0)
    let mut storage = manager.into_storage();
    storage.data_mut()[0] ^= 0xFF; // Corrupt CRC

    // Reinitialize - should detect corruption and reformat
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // All slots should be empty after reformat
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
    assert_eq!(manager.slot_status(1), SlotStatus::Empty);
    assert_eq!(manager.slot_status(2), SlotStatus::Empty);
}

#[test]
fn mismatched_magic_causes_reformat() {
    let storage = TestStorage::new_sram(4096);
    let magic1 = *b"game-one________________________";
    let magic2 = *b"game-two________________________";

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, magic1, MIN_SECTOR_SIZE).unwrap();

    // Write data with magic1
    let metadata = TestMetadata {
        name: *b"MagicMismatch___",
    };
    let data: Vec<u8> = vec![1, 2, 3];
    manager.write(0, &data, &metadata).unwrap();

    // Reinitialize with different magic
    let storage = manager.into_storage();
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, magic2, MIN_SECTOR_SIZE).unwrap();

    // Should have reformatted due to magic mismatch
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
}

#[test]
fn mismatched_slot_count_causes_reformat() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Write data with 2 slots
    let metadata = TestMetadata {
        name: *b"SlotCountChange_",
    };
    let data: Vec<u8> = vec![1, 2, 3];
    manager.write(0, &data, &metadata).unwrap();

    // Reinitialize with different slot count
    let storage = manager.into_storage();
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Should have reformatted due to slot count mismatch
    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
}

// --- Interleaved multi-slot operation tests ---

#[test]
fn interleaved_writes_to_multiple_slots() {
    let storage = TestStorage::new_sram(8192);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Interleaved writes: slot 0, slot 1, slot 0, slot 2, slot 1
    let meta0 = TestMetadata {
        name: *b"InterleavedS0___",
    };
    let meta1 = TestMetadata {
        name: *b"InterleavedS1___",
    };
    let meta2 = TestMetadata {
        name: *b"InterleavedS2___",
    };

    let data0_v1: Vec<u8> = vec![0; 50];
    manager.write(0, &data0_v1, &meta0).unwrap();

    let data1_v1: Vec<u8> = vec![1; 50];
    manager.write(1, &data1_v1, &meta1).unwrap();

    let data0_v2: Vec<u8> = vec![10; 50];
    manager.write(0, &data0_v2, &meta0).unwrap();

    let data2_v1: Vec<u8> = vec![2; 50];
    manager.write(2, &data2_v1, &meta2).unwrap();

    let data1_v2: Vec<u8> = vec![11; 50];
    manager.write(1, &data1_v2, &meta1).unwrap();

    // Verify all slots have their latest data
    let read0: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read0, data0_v2);

    let read1: Vec<u8> = manager.read(1).unwrap();
    assert_eq!(read1, data1_v2);

    let read2: Vec<u8> = manager.read(2).unwrap();
    assert_eq!(read2, data2_v1);
}

#[test]
fn interleaved_writes_with_erase() {
    let storage = TestStorage::new_sram(8192);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let meta = TestMetadata {
        name: *b"InterleavedErase",
    };

    // Write to all slots
    for slot in 0..3 {
        let data: Vec<u8> = vec![slot as u8; 100];
        manager.write(slot, &data, &meta).unwrap();
    }

    // Erase slot 1
    manager.erase(1).unwrap();
    assert_eq!(manager.slot_status(1), SlotStatus::Empty);

    // Write to slot 0 again (should not affect empty slot 1)
    let new_data0: Vec<u8> = vec![0xFF; 100];
    manager.write(0, &new_data0, &meta).unwrap();

    // Verify states
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    assert_eq!(manager.slot_status(1), SlotStatus::Empty);
    assert_eq!(manager.slot_status(2), SlotStatus::Valid);

    let read0: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read0, new_data0);

    let read2: Vec<u8> = manager.read(2).unwrap();
    assert_eq!(read2, vec![2u8; 100]);
}

#[test]
fn ghost_sector_correct_after_interleaved_writes() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let meta = TestMetadata {
        name: *b"GhostInterleave_",
    };

    // Write multiple times to different slots
    for i in 0..5 {
        let slot = i % 2;
        let data: Vec<u8> = vec![i as u8; 50];
        manager.write(slot, &data, &meta).unwrap();
    }

    // Reinitialize and verify
    let storage = manager.into_storage();
    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 2, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Slot 0 had writes at i=0,2,4 -> latest is 4
    let read0: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read0, vec![4u8; 50]);

    // Slot 1 had writes at i=1,3 -> latest is 3
    let read1: Vec<u8> = manager.read(1).unwrap();
    assert_eq!(read1, vec![3u8; 50]);
}

// --- Exact block boundary data size tests ---

#[test]
fn data_exactly_fills_one_block() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 1, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // Calculate exact payload size for one data block
    // Data block header is 8 bytes, so payload = sector_size - 8
    let payload_size = MIN_SECTOR_SIZE - 8;
    let data: Vec<u8> = vec![0xAB; payload_size];

    let meta = TestMetadata {
        name: *b"ExactOneBlock___",
    };

    manager.write(0, &data, &meta).unwrap();

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn data_one_byte_over_block_boundary() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 1, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    // One byte more than one block can hold -> needs 2 blocks
    let payload_size = MIN_SECTOR_SIZE - 8;
    let data: Vec<u8> = vec![0xCD; payload_size + 1];

    let meta = TestMetadata {
        name: *b"OverBoundary____",
    };

    manager.write(0, &data, &meta).unwrap();

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn data_exactly_fills_two_blocks() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 1, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let payload_size = MIN_SECTOR_SIZE - 8;
    let data: Vec<u8> = vec![0xEF; payload_size * 2];

    let meta = TestMetadata {
        name: *b"ExactTwoBlocks__",
    };

    manager.write(0, &data, &meta).unwrap();

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn single_byte_data() {
    let storage = TestStorage::new_sram(4096);

    let mut manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 1, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let data: Vec<u8> = vec![0x42];
    let meta = TestMetadata {
        name: *b"SingleByte______",
    };

    manager.write(0, &data, &meta).unwrap();

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}

#[test]
fn metadata_near_max_size() {
    let storage = TestStorage::new_sram(4096);

    // Use a metadata type that's larger (close to slot header capacity)
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
    struct LargeMetadata {
        data: Vec<u8>,
    }

    let mut manager: SaveSlotManager<_, LargeMetadata> =
        SaveSlotManager::new(storage, 1, TEST_GAME_MAGIC, MIN_SECTOR_SIZE).unwrap();

    let meta = LargeMetadata {
        data: vec![0xAB; 64],
    };
    let data: Vec<u8> = vec![1, 2, 3];

    manager.write(0, &data, &meta).unwrap();

    assert_eq!(manager.slot_status(0), SlotStatus::Valid);
    let read_meta = manager.metadata(0).unwrap();
    assert_eq!(read_meta.data, vec![0xAB; 64]);

    let read_data: Vec<u8> = manager.read(0).unwrap();
    assert_eq!(read_data, data);
}
