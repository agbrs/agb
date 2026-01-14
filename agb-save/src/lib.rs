//! This crate provides a storage agnostic way to save any serde serialize / deserialize
//! data into save slots like you would find in a classic RPG.
//!
//! It is opinionated on how the data should be stored and accessed, and expects you to be
//! able to load an entire save slot into RAM at once.
//!
//! # Example
//!
//! ```ignore
//! use agb_save::{SaveSlotManager, StorageMedium};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct SaveData {
//!     level: u32,
//!     health: u32,
//!     inventory: Vec<Item>,
//! }
//!
//! #[derive(Serialize, Deserialize)]
//! struct SaveMetadata {
//!     player_name: String,
//!     playtime_seconds: u32,
//! }
//!
//! let mut manager = SaveSlotManager::<_, SaveMetadata>::new(
//!     storage,
//!     3,  // 3 save slots
//!     *b"my-game-v1.0____________________",
//! )?;
//!
//! // Check slot status before loading
//! match manager.slot_status(0) {
//!     SlotStatus::Empty => println!("Slot 0 is empty"),
//!     SlotStatus::Valid => {
//!         let metadata = manager.metadata(0).unwrap();
//!         println!("Player: {}", metadata.player_name);
//!     }
//!     SlotStatus::Corrupted => println!("Slot 0 is corrupted"),
//! }
//!
//! // Save game
//! manager.write(0, &save_data, &metadata)?;
//!
//! // Load game
//! let save_data: SaveData = manager.read(0)?;
//! ```
#![no_std]
#![warn(clippy::all)]
#![warn(missing_docs)]

extern crate alloc;

use alloc::vec::Vec;
use core::num::NonZeroUsize;

#[cfg(test)]
pub(crate) mod test_storage;

mod block;
mod sector_storage;

/// Data about how the [`StorageMedium`] should be used.
#[derive(Debug, Clone, Copy)]
pub struct StorageInfo {
    /// Total size in bytes
    pub size: usize,
    /// Minimum erase size in bytes, None if no explicit erase is required. All erases
    /// must happen at the same alignment as erase_size as well.
    pub erase_size: Option<NonZeroUsize>,
    /// Minimum write size, 1 if byte-addressable. All writes must happen at alignment equal
    /// to write_size as well.
    pub write_size: NonZeroUsize,
}

/// Core trait for save storage access.
pub trait StorageMedium {
    /// The error kind for this.
    type Error;

    /// Get information about this storage.
    fn info(&self) -> StorageInfo;

    /// Read bytes from storage into the buffer
    ///
    /// Returns an error if `offset + buf.len() > self.info().size`
    fn read(&mut self, offset: usize, buf: &mut [u8]) -> Result<(), Self::Error>;

    /// Erase a region before writing.
    ///
    /// `offset` and `len` must be aligned to `info().erase_size`. For storage that
    /// doesn't require erase (`erase_size` is `None`), this is a no-op
    fn erase(&mut self, offset: usize, len: usize) -> Result<(), Self::Error>;

    /// Write bytes to storage.
    ///
    /// The region must have been erased first for media that require it. They should
    /// be aligned to `info().write_size`.
    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Self::Error>;
}

/// The status of a save slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotStatus {
    /// Slot has never been written to or has been erased.
    Empty,
    /// Slot contains valid, verified save data.
    Valid,
    /// Slot data is corrupted and could not be recovered.
    Corrupted,
}

/// Errors that can occur during save operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveError<StorageError> {
    /// The underlying storage returned an error.
    Storage(StorageError),
    /// The slot is empty (no data to read).
    SlotEmpty,
    /// The slot data is corrupted.
    SlotCorrupted,
    /// Not enough free space to write the data.
    OutOfSpace,
    /// Failed to serialize the data.
    SerializationFailed,
    /// Failed to deserialize the data.
    DeserializationFailed,
}

/// A save slot manager which gives you some level of write safety and confirmation
/// that everything worked correctly.
///
/// This manager handles:
/// - Multiple save slots (like classic RPGs)
/// - Corruption detection and recovery via ghost slots
/// - Metadata for each slot (e.g., player name, playtime) that can be read without
///   loading the full save data
///
/// # Type Parameters
///
/// - `Storage`: The underlying storage medium (e.g., flash, SRAM)
/// - `Metadata`: A serde-serializable type for slot metadata shown in save menus
pub struct SaveSlotManager<Storage: StorageMedium, Metadata> {
    num_slots: usize,
    storage: Storage,
    magic: [u8; 32],
    slot_info: Vec<SlotInfo<Metadata>>,
    block_size: usize,
}

/// Internal info about a slot's current state
struct SlotInfo<Metadata> {
    status: SlotStatus,
    metadata: Option<Metadata>,
    generation: u32,
    first_data_block: u16,
    data_length: u32,
    data_crc32: u32,
}

impl<Storage, Metadata> SaveSlotManager<Storage, Metadata>
where
    Storage: StorageMedium,
    Metadata: serde::Serialize + serde::de::DeserializeOwned,
{
    /// Create a new instance of the SaveSlotManager.
    ///
    /// This will read from `storage` to initialise itself:
    /// - If the storage is uninitialised or has mismatched magic, it will be formatted
    /// - If any slots are corrupted but recoverable from ghost, they will be recovered
    /// - Slot statuses and metadata are loaded into memory for fast access
    ///
    /// # Arguments
    ///
    /// * `storage` - The underlying storage medium
    /// * `num_slots` - The number of save slots (typically 1-4)
    /// * `magic` - A 32-byte game identifier. If this doesn't match what's stored,
    ///   the save file is considered incompatible and will be reformatted.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying storage operations fail.
    pub fn new(
        storage: Storage,
        num_slots: usize,
        magic: [u8; 32],
    ) -> Result<Self, SaveError<Storage::Error>> {
        let mut manager = Self {
            num_slots,
            storage,
            magic,
            slot_info: Vec::new(),
            block_size: 0,
        };
        manager.initialize()?;
        Ok(manager)
    }

    /// Returns the number of save slots.
    pub fn num_slots(&self) -> usize {
        self.num_slots
    }

    /// Returns the status of the given slot.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    pub fn slot_status(&self, slot: usize) -> SlotStatus {
        self.slot_info[slot].status
    }

    /// Returns the metadata for the given slot, if it exists and is valid.
    ///
    /// This is useful for displaying save slot information (player name, playtime, etc.)
    /// without loading the full save data.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    pub fn metadata(&self, slot: usize) -> Option<&Metadata> {
        self.slot_info[slot].metadata.as_ref()
    }

    /// Read the full save data from a slot.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    ///
    /// # Errors
    ///
    /// - [`SaveError::SlotEmpty`] if the slot has no data
    /// - [`SaveError::SlotCorrupted`] if the slot data is corrupted
    /// - [`SaveError::DeserializationFailed`] if the data cannot be deserialized
    /// - [`SaveError::Storage`] if the underlying storage fails
    pub fn read<T>(&mut self, slot: usize) -> Result<T, SaveError<Storage::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        match self.slot_info[slot].status {
            SlotStatus::Empty => Err(SaveError::SlotEmpty),
            SlotStatus::Corrupted => Err(SaveError::SlotCorrupted),
            SlotStatus::Valid => self.read_slot_data(slot),
        }
    }

    /// Write save data and metadata to a slot.
    ///
    /// This operation is designed to be crash-safe:
    /// 1. Data is written to new blocks
    /// 2. A new slot header is written
    /// 3. The old slot header is marked as ghost (backup)
    ///
    /// If a crash occurs during writing, the next `new()` call will recover
    /// from the ghost slot if the new data is incomplete.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    ///
    /// # Errors
    ///
    /// - [`SaveError::OutOfSpace`] if there's not enough free space
    /// - [`SaveError::SerializationFailed`] if the data cannot be serialized
    /// - [`SaveError::Storage`] if the underlying storage fails
    pub fn write<T>(
        &mut self,
        slot: usize,
        data: &T,
        metadata: &Metadata,
    ) -> Result<(), SaveError<Storage::Error>>
    where
        T: serde::Serialize,
    {
        assert!(slot < self.num_slots, "slot index out of bounds");
        self.write_slot_data(slot, data, metadata)
    }

    /// Erase a save slot, marking it as empty.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    ///
    /// # Errors
    ///
    /// - [`SaveError::Storage`] if the underlying storage fails
    pub fn erase(&mut self, slot: usize) -> Result<(), SaveError<Storage::Error>> {
        assert!(slot < self.num_slots, "slot index out of bounds");
        self.erase_slot(slot)
    }

    /// Returns an iterator over all slots with their status and metadata.
    ///
    /// Useful for displaying a save slot selection screen.
    pub fn slots(&self) -> impl Iterator<Item = (usize, SlotStatus, Option<&Metadata>)> {
        self.slot_info
            .iter()
            .enumerate()
            .map(|(i, info)| (i, info.status, info.metadata.as_ref()))
    }

    // --- Private implementation methods ---

    fn initialize(&mut self) -> Result<(), SaveError<Storage::Error>> {
        // TODO: Implement initialization logic
        // 1. Calculate block size from storage info
        // 2. Load and verify global header (including slot count)
        // 3. Load slot headers and metadata
        // 4. Attempt recovery from ghost slots if needed
        // 5. Rebuild free list
        todo!()
    }

    fn read_slot_data<T>(&mut self, _slot: usize) -> Result<T, SaveError<Storage::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        // TODO: Implement
        // 1. Follow data block chain
        // 2. Concatenate payloads
        // 3. Verify CRC32
        // 4. Deserialize
        todo!()
    }

    fn write_slot_data<T>(
        &mut self,
        _slot: usize,
        _data: &T,
        _metadata: &Metadata,
    ) -> Result<(), SaveError<Storage::Error>>
    where
        T: serde::Serialize,
    {
        // TODO: Implement
        // 1. Serialize data
        // 2. Allocate blocks from free list
        // 3. Write data chain
        // 4. Calculate CRC32
        // 5. Serialize metadata
        // 6. Write new slot header
        // 7. Mark old slot as ghost
        // 8. Update in-memory state
        todo!()
    }

    fn erase_slot(&mut self, _slot: usize) -> Result<(), SaveError<Storage::Error>> {
        // TODO: Implement
        // 1. Mark slot header as empty
        // 2. Return data blocks to free list
        // 3. Update in-memory state
        todo!()
    }
}
