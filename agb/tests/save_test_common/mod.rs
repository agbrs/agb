use agb::save::{Slot, SlotStatus};
use serde::{Deserialize, Serialize};

/// Test metadata stored with each save slot
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TestMetadata {
    pub name: [u8; 8],
    pub level: u32,
}

/// Test save data
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TestSaveData {
    pub score: u32,
    pub items: [u16; 10],
    pub checksum: u32,
}

impl TestSaveData {
    pub fn new(seed: u32) -> Self {
        let mut data = TestSaveData {
            score: seed.wrapping_mul(12345),
            items: [0; 10],
            checksum: 0,
        };
        for (i, item) in data.items.iter_mut().enumerate() {
            *item = (seed.wrapping_add(i as u32).wrapping_mul(7919)) as u16;
        }
        data.checksum = data.compute_checksum();
        data
    }

    fn compute_checksum(&self) -> u32 {
        let mut sum = self.score;
        for &item in &self.items {
            sum = sum.wrapping_add(item as u32);
        }
        sum
    }

    pub fn verify(&self) -> bool {
        self.checksum == self.compute_checksum()
    }
}

impl TestMetadata {
    pub fn new(name: &[u8], level: u32) -> Self {
        let mut n = [0u8; 8];
        let len = name.len().min(8);
        n[..len].copy_from_slice(&name[..len]);
        TestMetadata { name: n, level }
    }
}

#[test_case]
fn test_write_and_read(gba: &mut agb::Gba) {
    let mut manager = crate::save_setup(gba);

    let data = TestSaveData::new(42);
    let metadata = TestMetadata::new(b"Player1", 5);

    // Write to slot 0
    manager
        .write(0, &data, &metadata)
        .expect("Failed to write save data");

    // Verify slot status
    assert_eq!(manager.slot_status(0), SlotStatus::Valid);

    // Read back and verify
    let loaded: TestSaveData = manager.read(0).expect("Failed to read save data");
    assert_eq!(loaded, data);
    assert!(loaded.verify());

    // Verify metadata
    let loaded_meta = manager.metadata(0).expect("Metadata should exist");
    assert_eq!(loaded_meta, &metadata);
}

#[test_case]
fn test_multiple_slots(gba: &mut agb::Gba) {
    let mut manager = crate::save_setup(gba);
    let num_slots = manager.num_slots();

    // Write different data to each slot
    for slot in 0..num_slots {
        let data = TestSaveData::new(slot as u32 * 1000);
        let metadata = TestMetadata::new(b"Slot", slot as u32);

        manager
            .write(slot, &data, &metadata)
            .expect("Failed to write");
    }

    // Verify all slots
    for slot in 0..num_slots {
        assert_eq!(manager.slot_status(slot), SlotStatus::Valid);

        let expected = TestSaveData::new(slot as u32 * 1000);
        let loaded: TestSaveData = manager.read(slot).expect("Failed to read");
        assert_eq!(loaded, expected);
    }
}

#[test_case]
fn test_erase_slot(gba: &mut agb::Gba) {
    let mut manager = crate::save_setup(gba);

    // Write to slot 0
    let data = TestSaveData::new(123);
    let metadata = TestMetadata::new(b"Test", 1);
    manager.write(0, &data, &metadata).expect("Failed to write");

    assert_eq!(manager.slot_status(0), SlotStatus::Valid);

    // Erase slot 0
    manager.erase(0).expect("Failed to erase");

    assert_eq!(manager.slot_status(0), SlotStatus::Empty);
    assert!(manager.metadata(0).is_none());
}

#[test_case]
fn test_overwrite_slot(gba: &mut agb::Gba) {
    let mut manager = crate::save_setup(gba);

    // Write initial data
    let data1 = TestSaveData::new(100);
    let metadata1 = TestMetadata::new(b"First", 1);
    manager
        .write(0, &data1, &metadata1)
        .expect("Failed to write first");

    // Overwrite with new data
    let data2 = TestSaveData::new(200);
    let metadata2 = TestMetadata::new(b"Second", 2);
    manager
        .write(0, &data2, &metadata2)
        .expect("Failed to write second");

    // Verify new data
    let loaded: TestSaveData = manager.read(0).expect("Failed to read");
    assert_eq!(loaded, data2);

    let loaded_meta = manager.metadata(0).expect("Metadata should exist");
    assert_eq!(loaded_meta, &metadata2);
}

#[test_case]
fn test_slots_iterator(gba: &mut agb::Gba) {
    let mut manager = crate::save_setup(gba);

    // Write to slot 0 only (within this test)
    let data = TestSaveData::new(999);
    let metadata = TestMetadata::new(b"Only", 42);
    manager.write(0, &data, &metadata).expect("Failed to write");

    // Check iterator
    let slots: alloc::vec::Vec<_> = manager.slots().collect();
    assert_eq!(slots.len(), manager.num_slots());

    // First slot should be valid with metadata
    match &slots[0] {
        Slot::Valid(meta) => {
            assert_eq!(meta.name, metadata.name);
            assert_eq!(meta.level, metadata.level);
        }
        other => panic!("Expected Slot::Valid, got {:?}", other),
    }

    // Note: Other slots may have data from previous tests since tests share state,
    // so we don't assert they're empty. Just verify the iterator returns all slots
    // and each slot is either Valid or Empty (not corrupted).
    for (idx, slot) in slots.iter().enumerate() {
        assert!(
            matches!(slot, Slot::Valid(_) | Slot::Empty),
            "Slot {} should be Valid or Empty, got {:?}",
            idx,
            slot
        );
    }
}

#[test_case]
fn test_persistence(gba: &mut agb::Gba) {
    let mut manager = crate::save_setup(gba);

    let data = TestSaveData::new(54321);
    let metadata = TestMetadata::new(b"Persist", 99);

    // Write data
    manager
        .write(0, &data, &metadata)
        .expect("Failed to write save data");

    // Drop the manager and reopen to simulate game restart
    drop(manager);
    let mut manager2 = crate::save_reopen(gba);

    // Verify data persisted
    assert_eq!(manager2.slot_status(0), SlotStatus::Valid);

    let loaded: TestSaveData = manager2.read(0).expect("Failed to read after reopen");
    assert_eq!(loaded, data);
    assert!(loaded.verify());

    let loaded_meta = manager2
        .metadata(0)
        .expect("Metadata should exist after reopen");
    assert_eq!(loaded_meta, &metadata);
}

extern crate alloc;
