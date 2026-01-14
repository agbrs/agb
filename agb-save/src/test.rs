use crate::sector_storage::MIN_SECTOR_SIZE;
use crate::test_storage::TestStorage;
use crate::{SaveSlotManager, SlotStatus};

use serde::{Deserialize, Serialize};

const TEST_GAME_MAGIC: [u8; 32] = *b"test-game-______________________";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
struct TestMetadata {
    name: [u8; 16],
}

#[test]
fn new_storage_has_empty_slots() {
    // 4KB storage, enough for several sectors
    let storage = TestStorage::new_sram(4096);

    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC).unwrap();

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
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC).unwrap();

    // Get the storage back and corrupt slot 1's header (sector 2)
    let mut storage = manager.into_storage();

    // Corrupt a byte after the CRC (byte 0-1 is CRC, so corrupt byte 4)
    // Sector 2 starts at offset 2 * MIN_SECTOR_SIZE
    let corrupt_offset = 2 * MIN_SECTOR_SIZE + 4;
    storage.data_mut()[corrupt_offset] ^= 0xFF;

    // Re-initialize from corrupted storage
    let manager: SaveSlotManager<_, TestMetadata> =
        SaveSlotManager::new(storage, 3, TEST_GAME_MAGIC).unwrap();

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
