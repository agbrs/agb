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

use alloc::vec;
use alloc::vec::Vec;
use core::num::NonZeroUsize;

use block::{
    Block, GlobalBlock, GlobalHeader, SlotHeader, SlotHeaderBlock, SlotState, deserialize_block,
    serialize_block,
};

#[cfg(test)]
mod test;
#[cfg(test)]
pub(crate) mod test_storage;

mod block;
mod sector_storage;

pub use sector_storage::MIN_SECTOR_SIZE;
use sector_storage::SectorStorage;

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
    storage: SectorStorage<Storage>,
    magic: [u8; 32],
    slot_info: Vec<SlotInfo<Metadata>>,
    free_sector_list: Vec<u16>,
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
    Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
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
    /// * `min_sector_size` - Minimum sector size in bytes. Must be at least
    ///   [`MIN_SECTOR_SIZE`]. Larger values allow more metadata per slot
    ///   (metadata size = sector_size - 24 bytes for the slot header).
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying storage operations fail.
    pub fn new(
        storage: Storage,
        num_slots: usize,
        magic: [u8; 32],
        min_sector_size: usize,
    ) -> Result<Self, SaveError<Storage::Error>> {
        let mut manager = Self {
            num_slots,
            storage: SectorStorage::new(storage, min_sector_size),
            magic,
            slot_info: Vec::new(),
            free_sector_list: Vec::new(),
        };
        manager.initialise()?;
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

    /// Consume the manager and return the underlying storage.
    ///
    /// Useful for testing or if you need to reclaim the storage.
    #[cfg(test)]
    pub(crate) fn into_storage(self) -> Storage {
        self.storage.into_inner()
    }

    // --- Private implementation methods ---

    fn initialise(&mut self) -> Result<(), SaveError<Storage::Error>> {
        let sector_size = self.storage.sector_size();
        let mut buffer = vec![0u8; sector_size];

        // Try to read the global header (sector 0)
        self.storage
            .read_sector(0, &mut buffer)
            .map_err(SaveError::Storage)?;

        let needs_format = match deserialize_block(&buffer) {
            Ok(Block::Global(global)) => {
                // Check if magic matches and slot count is correct
                global.game_identifier[..32] != self.magic
                    || global.header.slot_count as usize != self.num_slots
            }
            _ => true, // CRC mismatch, wrong block type, or other error
        };

        if needs_format {
            self.format_storage()?;
        } else {
            self.load_slot_headers()?;
        }

        Ok(())
    }

    fn format_storage(&mut self) -> Result<(), SaveError<Storage::Error>> {
        let sector_size = self.storage.sector_size();
        let mut buffer = vec![0u8; sector_size];

        // Write global header (sector 0)
        serialize_block(
            Block::Global(GlobalBlock {
                header: GlobalHeader {
                    slot_count: self.num_slots as u16,
                },
                game_identifier: &self.magic,
            }),
            &mut buffer,
        );
        self.storage
            .write_sector(0, &buffer)
            .map_err(SaveError::Storage)?;

        // Write empty slot headers (sectors 1..num_slots+1)
        let metadata_size = sector_size - SlotHeaderBlock::header_size();
        let empty_metadata = vec![0u8; metadata_size];

        for slot in 0..self.num_slots {
            buffer.fill(0);
            serialize_block(
                Block::SlotHeader(SlotHeaderBlock {
                    header: SlotHeader::empty(slot as u8),
                    metadata: &empty_metadata,
                }),
                &mut buffer,
            );
            self.storage
                .write_sector(slot + 1, &buffer)
                .map_err(SaveError::Storage)?;
        }

        // Initialise slot_info with empty slots
        self.slot_info.clear();
        for _ in 0..self.num_slots {
            self.slot_info.push(SlotInfo {
                status: SlotStatus::Empty,
                metadata: None,
                generation: 0,
                first_data_block: 0xFFFF,
                data_length: 0,
                data_crc32: 0,
            });
        }

        // All sectors after slot headers are free
        // Sector layout: [0: global] [1..num_slots+1: slot headers] [num_slots+1: ghost] [num_slots+2..: data]
        let first_data_sector = self.num_slots + 2;
        let sector_count = self.storage.sector_count();
        self.free_sector_list.clear();
        for sector in first_data_sector..sector_count {
            self.free_sector_list.push(sector as u16);
        }

        Ok(())
    }

    fn load_slot_headers(&mut self) -> Result<(), SaveError<Storage::Error>> {
        let sector_size = self.storage.sector_size();
        let mut buffer = vec![0u8; sector_size];

        // Pre-initialize all slots as corrupted (will be filled in as we find valid headers)
        self.slot_info.clear();
        for _ in 0..self.num_slots {
            self.slot_info.push(SlotInfo {
                status: SlotStatus::Corrupted,
                metadata: None,
                generation: 0,
                first_data_block: 0xFFFF,
                data_length: 0,
                data_crc32: 0,
            });
        }

        // Scan slot header sectors and populate slot_info based on logical_slot_id
        // We scan num_slots + 1 sectors to include the ghost slot sector
        let num_slot_header_sectors = self.num_slots + 1;
        for sector in 0..num_slot_header_sectors {
            self.storage
                .read_sector(sector + 1, &mut buffer)
                .map_err(SaveError::Storage)?;

            if let Ok(Block::SlotHeader(slot_block)) = deserialize_block(&buffer) {
                let logical_id = slot_block.header.logical_slot_id as usize;

                // Skip if logical_id is out of bounds
                if logical_id >= self.num_slots {
                    continue;
                }

                let existing = &self.slot_info[logical_id];

                // Only update if this is a newer generation or the existing slot is corrupted
                if existing.status == SlotStatus::Corrupted
                    || slot_block.header.generation > existing.generation
                {
                    let (status, metadata) = match slot_block.header.state {
                        SlotState::Empty => (SlotStatus::Empty, None),
                        SlotState::Valid => {
                            // Deserialize metadata - if it fails, slot is corrupted
                            match postcard::from_bytes(slot_block.metadata) {
                                Ok(m) => (SlotStatus::Valid, Some(m)),
                                Err(_) => (SlotStatus::Corrupted, None),
                            }
                        }
                        SlotState::Ghost => (SlotStatus::Empty, None), // Treat ghost as empty for now
                    };

                    self.slot_info[logical_id] = SlotInfo {
                        status,
                        metadata,
                        generation: slot_block.header.generation,
                        first_data_block: slot_block.header.first_data_block,
                        data_length: slot_block.header.length,
                        data_crc32: slot_block.header.crc32,
                    };
                }
            }
            // If deserialization fails, we leave the slot as Corrupted (already initialized)
        }

        // Build free sector list
        match self.build_free_sector_list() {
            Some(free_list) => {
                self.free_sector_list = free_list;
                Ok(())
            }
            None => {
                // Corruption detected (loop in chain), reformat
                self.format_storage()
            }
        }
    }

    /// Build the free sector list by scanning data chains from valid slots.
    ///
    /// Returns `None` if corruption is detected (e.g., a loop in a data chain),
    /// indicating the storage needs to be reformatted.
    fn build_free_sector_list(&mut self) -> Option<Vec<u16>> {
        // Sector layout: [0: global] [1..num_slots+1: slot headers] [num_slots+1: ghost] [num_slots+2..: data]
        let first_data_sector = self.num_slots + 2;
        let sector_count = self.storage.sector_count();
        let mut used_sectors = vec![false; sector_count];

        // Mark header sectors as used
        for used in used_sectors.iter_mut().take(first_data_sector) {
            *used = true;
        }

        // Follow data chains from valid slots and mark those sectors as used
        let max_data_sectors = sector_count - first_data_sector;
        let mut total_sectors_visited = 0usize;
        let sector_size = self.storage.sector_size();
        let mut buffer = vec![0u8; sector_size];

        for slot_info in &self.slot_info {
            if slot_info.status != SlotStatus::Valid {
                continue;
            }

            let mut current_sector = slot_info.first_data_block;
            while current_sector != 0xFFFF {
                let sector_idx = current_sector as usize;
                if sector_idx >= sector_count {
                    break; // Invalid sector reference, stop following chain
                }
                used_sectors[sector_idx] = true;
                total_sectors_visited += 1;

                // If we've visited more sectors than possible, there's a loop
                if total_sectors_visited > max_data_sectors {
                    return None;
                }

                // Read the sector to find the next block in the chain
                if self.storage.read_sector(sector_idx, &mut buffer).is_err() {
                    break;
                }
                match deserialize_block(&buffer) {
                    Ok(Block::Data(data_block)) => {
                        current_sector = data_block.header.next_block;
                    }
                    _ => break, // Not a valid data block, stop following chain
                }
            }
        }

        // Collect free sectors
        let mut free_list = Vec::new();
        for (sector, &used) in used_sectors.iter().enumerate() {
            if !used {
                free_list.push(sector as u16);
            }
        }

        Some(free_list)
    }

    fn read_slot_data<T>(&mut self, _slot: usize) -> Result<T, SaveError<Storage::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        // TODO: Implement
        // Uses self.storage.read_sector() and block::deserialize_block
        // 1. Follow data block chain
        // 2. Concatenate payloads
        // 3. Verify CRC32
        // 4. Deserialize with serde
        todo!()
    }

    fn write_slot_data<T>(
        &mut self,
        slot: usize,
        _data: &T,
        metadata: &Metadata,
    ) -> Result<(), SaveError<Storage::Error>>
    where
        T: serde::Serialize,
    {
        let sector_size = self.storage.sector_size();
        let metadata_size = sector_size - SlotHeaderBlock::header_size();

        // Serialize metadata
        let mut metadata_bytes = vec![0u8; metadata_size];
        postcard::to_slice(metadata, &mut metadata_bytes)
            .map_err(|_| SaveError::SerializationFailed)?;

        // TODO: Serialize data and write data blocks
        // For now, we just write the slot header with no data
        let first_data_block = 0xFFFF;
        let data_length = 0u32;
        let data_crc32 = 0u32;

        // Increment generation
        let new_generation = self.slot_info[slot].generation.wrapping_add(1);

        // Create the slot header
        let slot_header = SlotHeader {
            state: SlotState::Valid,
            logical_slot_id: slot as u8,
            first_data_block,
            generation: new_generation,
            crc32: data_crc32,
            length: data_length,
        };

        // Serialize and write the block
        let mut buffer = vec![0u8; sector_size];
        serialize_block(
            Block::SlotHeader(SlotHeaderBlock {
                header: slot_header,
                metadata: &metadata_bytes,
            }),
            &mut buffer,
        );

        self.storage
            .write_sector(slot + 1, &buffer)
            .map_err(SaveError::Storage)?;

        // Update in-memory state
        self.slot_info[slot] = SlotInfo {
            status: SlotStatus::Valid,
            metadata: Some(metadata.clone()),
            generation: new_generation,
            first_data_block,
            data_length,
            data_crc32,
        };

        Ok(())
    }

    fn erase_slot(&mut self, _slot: usize) -> Result<(), SaveError<Storage::Error>> {
        // TODO: Implement
        // Uses self.storage.write_sector() and block::serialize_block
        // 1. Write slot header with state = Empty (Block::SlotHeader)
        // 2. Return data sectors to free list
        // 3. Update in-memory state
        todo!()
    }
}

fn calc_crc32(bytes: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}
