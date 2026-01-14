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
