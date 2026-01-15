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
//! match manager.slot(0) {
//!     Slot::Empty => println!("Slot 0 is empty"),
//!     Slot::Valid(metadata) => {
//!         println!("Player: {}", metadata.player_name);
//!     }
//!     Slot::Corrupted => println!("Slot 0 is corrupted"),
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
    Block, DataBlock, GlobalBlock, SlotHeaderBlock, SlotState, deserialize_block, serialize_block,
};

#[cfg(test)]
mod test;
#[cfg(test)]
pub(crate) mod test_storage;

mod block;
mod sector_storage;

pub use sector_storage::MIN_SECTOR_SIZE;
use sector_storage::{SectorError, SectorStorage};

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

    /// Verify that data at the given offset matches the expected bytes.
    ///
    /// Returns `Ok(true)` if the data matches, `Ok(false)` if it doesn't match,
    /// or `Err` if there was a storage error during the read.
    ///
    /// The default implementation reads back the data and compares byte-by-byte.
    fn verify(&mut self, offset: usize, expected: &[u8]) -> Result<bool, Self::Error> {
        let mut buf = vec![0u8; expected.len()];
        self.read(offset, &mut buf)?;
        Ok(buf == expected)
    }
}

/// The status of a save slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SlotStatus {
    /// Slot has never been written to or has been erased.
    Empty,
    /// Slot contains valid, verified save data.
    Valid,
    /// Slot data is corrupted and could not be recovered.
    Corrupted,
}

/// A save slot with its current state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot<'a, Metadata> {
    /// Slot has never been written to or has been erased.
    Empty,
    /// Slot contains valid, verified save data with the given metadata.
    Valid(&'a Metadata),
    /// Slot data is corrupted and could not be recovered.
    Corrupted,
}

/// Errors that can occur during save operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError<StorageError> {
    /// The underlying storage returned an error.
    Storage(StorageError),
    /// The slot is empty (no data to read).
    SlotEmpty,
    /// The slot data is corrupted.
    SlotCorrupted,
    /// Not enough free space to write the data.
    OutOfSpace,
    /// Failed to serialize / deserialize the data.
    Serialization(SerializationError),
    /// Write verification failed - data read back didn't match what was written.
    VerificationFailed,
}

/// Further details about the serialization error.
///
/// This only provides a Debug implementation you can use to debug the issues.
/// The output of this isn't stable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializationError(postcard::Error);

impl<StorageError> SaveError<StorageError> {
    fn from_postcard_serialization(e: postcard::Error) -> Self {
        Self::Serialization(SerializationError(e))
    }

    fn from_data_verify_error(e: DataVerifyError) -> Self {
        match e {
            DataVerifyError::Deserialization(e) => Self::Serialization(SerializationError(e)),
            DataVerifyError::LengthMismatch | DataVerifyError::CrcMismatch => Self::SlotCorrupted,
        }
    }
}

/// Errors that can occur when verifying and deserializing data.
#[derive(Debug)]
enum DataVerifyError {
    /// The data length doesn't match the expected length.
    LengthMismatch,
    /// The CRC32 checksum doesn't match.
    CrcMismatch,
    /// Deserialization failed.
    Deserialization(postcard::Error),
}

/// Verify data CRC32 and deserialize if valid.
///
/// Checks that:
/// 1. The data slice has exactly `expected_length` bytes
/// 2. The CRC32 of the data matches `expected_crc32`
/// 3. The data can be deserialized into type T
fn verify_and_deserialize_data<T>(
    data: &[u8],
    expected_length: u32,
    expected_crc32: u32,
) -> Result<T, DataVerifyError>
where
    T: serde::de::DeserializeOwned,
{
    let expected_len = expected_length as usize;
    if data.len() != expected_len {
        return Err(DataVerifyError::LengthMismatch);
    }

    let actual_crc = calc_crc32(data);
    if actual_crc != expected_crc32 {
        return Err(DataVerifyError::CrcMismatch);
    }

    postcard::from_bytes(data).map_err(DataVerifyError::Deserialization)
}

impl<T> From<T> for SaveError<T> {
    fn from(value: T) -> Self {
        Self::Storage(value)
    }
}

impl<T> From<SectorError<T>> for SaveError<T> {
    fn from(value: SectorError<T>) -> Self {
        match value {
            SectorError::Storage(e) => SaveError::Storage(e),
            SectorError::VerificationFailed => SaveError::VerificationFailed,
        }
    }
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
    /// The physical sector currently acting as the ghost/staging area
    ghost_sector: u16,
}

/// Internal info about a slot's current state
struct SlotInfo<Metadata> {
    status: SlotStatus,
    metadata: Option<Metadata>,
    generation: u32,
    first_data_block: Option<u16>,
    data_length: u32,
    data_crc32: u32,
    /// Physical sector where this slot's header is stored
    header_sector: u16,
}

impl<Metadata> SlotInfo<Metadata> {
    fn empty(generation: u32, header_sector: u16) -> Self {
        Self {
            status: SlotStatus::Empty,
            metadata: None,
            generation,
            first_data_block: None,
            data_length: 0,
            data_crc32: 0,
            header_sector,
        }
    }

    fn corrupted(header_sector: u16) -> Self {
        Self {
            status: SlotStatus::Corrupted,
            metadata: None,
            generation: 0,
            first_data_block: None,
            data_length: 0,
            data_crc32: 0,
            header_sector,
        }
    }

    fn valid(
        metadata: Metadata,
        generation: u32,
        first_data_block: Option<u16>,
        data_length: u32,
        data_crc32: u32,
        header_sector: u16,
    ) -> Self {
        Self {
            status: SlotStatus::Valid,
            metadata: Some(metadata),
            generation,
            first_data_block,
            data_length,
            data_crc32,
            header_sector,
        }
    }
}

/// Information stored from a ghost header for potential slot recovery.
struct GhostRecoveryInfo {
    generation: u32,
    first_data_block: Option<u16>,
    data_length: u32,
    data_crc32: u32,
    metadata_bytes: Vec<u8>,
    metadata_length: u32,
    metadata_crc32: u32,
    physical_sector: u16,
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
            ghost_sector: 0, // Will be set by initialise
        };
        manager.initialise()?;
        Ok(manager)
    }

    /// Returns the number of save slots.
    pub fn num_slots(&self) -> usize {
        self.num_slots
    }

    /// Returns the state of the given slot.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    pub fn slot(&self, slot: usize) -> Slot<'_, Metadata> {
        let info = &self.slot_info[slot];
        match info.status {
            SlotStatus::Empty => Slot::Empty,
            SlotStatus::Valid => Slot::Valid(info.metadata.as_ref().unwrap()),
            SlotStatus::Corrupted => Slot::Corrupted,
        }
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
    /// - [`SaveError::Serialization`] if the data cannot be deserialized
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
    /// - [`SaveError::Serialization`] if the data cannot be serialized
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
        assert!(slot < self.num_slots, "slot index {slot} out of bounds");
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
        assert!(slot < self.num_slots, "slot index {slot} out of bounds");
        self.erase_slot(slot)
    }

    /// Returns an iterator over all slots with their current state.
    ///
    /// Useful for displaying a save slot selection screen.
    pub fn slots(&self) -> impl Iterator<Item = Slot<'_, Metadata>> {
        self.slot_info.iter().map(|info| match info.status {
            SlotStatus::Empty => Slot::Empty,
            SlotStatus::Valid => Slot::Valid(info.metadata.as_ref().unwrap()),
            SlotStatus::Corrupted => Slot::Corrupted,
        })
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
        self.storage.read_sector(0, &mut buffer)?;

        let needs_format = match deserialize_block(&buffer) {
            Ok(Block::Global(global)) => {
                // Check if magic matches and slot count is correct
                global.game_identifier[..32] != self.magic
                    || global.slot_count() as usize != self.num_slots
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
            Block::Global(GlobalBlock::new(self.num_slots as u16, &self.magic)),
            &mut buffer,
        );
        self.storage.write_sector(0, &buffer)?;

        // Write empty slot headers (sectors 1..num_slots+1)
        // Each logical slot gets its own physical sector initially
        let metadata_size = sector_size - SlotHeaderBlock::header_size();
        let empty_metadata = vec![0u8; metadata_size];

        for slot in 0..self.num_slots {
            buffer.fill(0);
            serialize_block(
                Block::SlotHeader(SlotHeaderBlock::empty(slot as u8, &empty_metadata)),
                &mut buffer,
            );
            self.storage.write_sector(slot + 1, &buffer)?;
        }

        // Write a GHOST state slot header to the ghost sector (sector num_slots + 1)
        // This is the extra physical slot sector used as a staging area
        // Use logical_slot_id = 0xFF to indicate it's not associated with any slot yet
        buffer.fill(0);
        serialize_block(
            Block::SlotHeader(SlotHeaderBlock::ghost(0xFF, &empty_metadata)),
            &mut buffer,
        );
        self.storage.write_sector(self.num_slots + 1, &buffer)?;

        // Initialise slot_info with empty slots
        self.slot_info.clear();
        for slot in 0..self.num_slots {
            self.slot_info.push(SlotInfo::empty(0, (slot + 1) as u16));
        }

        // The ghost sector starts at num_slots + 1 (the extra physical slot sector)
        self.ghost_sector = (self.num_slots + 1) as u16;

        // All sectors after the slot header sectors are free (data area)
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
        self.initialize_slots_as_corrupted();
        let ghost_recovery = self.scan_slot_headers()?;
        self.recover_from_ghosts(ghost_recovery);
        self.ghost_sector = self.find_unused_header_sector();
        self.build_free_sector_list()
    }

    /// Pre-initialize all slots as corrupted.
    fn initialize_slots_as_corrupted(&mut self) {
        self.slot_info.clear();
        for slot in 0..self.num_slots {
            self.slot_info.push(SlotInfo::corrupted((slot + 1) as u16));
        }
    }

    /// Scan slot header sectors and populate slot_info.
    ///
    /// Returns ghost recovery info for potential slot recovery.
    fn scan_slot_headers(
        &mut self,
    ) -> Result<Vec<Option<GhostRecoveryInfo>>, SaveError<Storage::Error>> {
        let sector_size = self.storage.sector_size();
        let mut buffer = vec![0u8; sector_size];
        let mut ghost_recovery: Vec<Option<GhostRecoveryInfo>> =
            (0..self.num_slots).map(|_| None).collect();

        // Default ghost sector to the last physical slot sector
        self.ghost_sector = (self.num_slots + 1) as u16;

        // Scan num_slots + 1 sectors (sectors 1 through num_slots+1 inclusive)
        let num_slot_header_sectors = self.num_slots + 1;
        for sector in 0..num_slot_header_sectors {
            let physical_sector = (sector + 1) as u16;
            self.storage
                .read_sector(physical_sector as usize, &mut buffer)?;

            if let Ok(Block::SlotHeader(slot_block)) = deserialize_block(&buffer) {
                self.process_slot_block(&slot_block, physical_sector, &mut ghost_recovery);
            }
            // If deserialization fails, slot remains Corrupted (already initialized)
        }

        Ok(ghost_recovery)
    }

    /// Process a single slot header block during scanning.
    fn process_slot_block(
        &mut self,
        slot_block: &SlotHeaderBlock,
        physical_sector: u16,
        ghost_recovery: &mut [Option<GhostRecoveryInfo>],
    ) {
        // Handle GHOST state - track for potential recovery
        if slot_block.state() == SlotState::Ghost {
            self.ghost_sector = physical_sector;

            let logical_id = slot_block.logical_slot_id() as usize;
            if logical_id < self.num_slots {
                ghost_recovery[logical_id] = Some(GhostRecoveryInfo {
                    generation: slot_block.generation(),
                    first_data_block: slot_block.first_data_block(),
                    data_length: slot_block.length(),
                    data_crc32: slot_block.crc32(),
                    metadata_bytes: slot_block.metadata().to_vec(),
                    metadata_length: slot_block.metadata_length(),
                    metadata_crc32: slot_block.metadata_crc32(),
                    physical_sector,
                });
            }
            return;
        }

        let logical_id = slot_block.logical_slot_id() as usize;

        // Skip if logical_id is out of bounds
        if logical_id >= self.num_slots {
            return;
        }

        let existing = &self.slot_info[logical_id];

        // Only update if this is a newer generation or the existing slot is corrupted
        if existing.status == SlotStatus::Corrupted
            || slot_block.generation() > existing.generation
        {
            let (status, metadata) = match slot_block.state() {
                SlotState::Empty => (SlotStatus::Empty, None),
                SlotState::Valid => {
                    let metadata_len = slot_block.metadata_length();
                    let metadata_bytes = slot_block.metadata();
                    if (metadata_len as usize) > metadata_bytes.len() {
                        (SlotStatus::Corrupted, None)
                    } else {
                        let metadata_slice = &metadata_bytes[..metadata_len as usize];
                        match verify_and_deserialize_data(
                            metadata_slice,
                            metadata_len,
                            slot_block.metadata_crc32(),
                        ) {
                            Ok(m) => (SlotStatus::Valid, Some(m)),
                            Err(_) => (SlotStatus::Corrupted, None),
                        }
                    }
                }
                SlotState::Ghost => unreachable!(), // Handled above
            };

            self.slot_info[logical_id] = SlotInfo {
                status,
                metadata,
                generation: slot_block.generation(),
                first_data_block: slot_block.first_data_block(),
                data_length: slot_block.length(),
                data_crc32: slot_block.crc32(),
                header_sector: physical_sector,
            };
        }
    }

    /// Try to recover corrupted slots from ghost headers.
    ///
    /// Only recovers if ghost has a generation >= the corrupted slot's generation
    /// (prevents recovering from an older ghost when new data is corrupted).
    fn recover_from_ghosts(&mut self, ghost_recovery: Vec<Option<GhostRecoveryInfo>>) {
        for (slot, ghost_info) in ghost_recovery.into_iter().enumerate() {
            if self.slot_info[slot].status == SlotStatus::Corrupted
                && let Some(ghost) = ghost_info
            {
                if ghost.generation >= self.slot_info[slot].generation {
                    let metadata_len = ghost.metadata_length as usize;
                    if metadata_len <= ghost.metadata_bytes.len() {
                        let metadata_slice = &ghost.metadata_bytes[..metadata_len];
                        if let Ok(metadata) = verify_and_deserialize_data(
                            metadata_slice,
                            ghost.metadata_length,
                            ghost.metadata_crc32,
                        ) {
                            self.slot_info[slot] = SlotInfo::valid(
                                metadata,
                                ghost.generation,
                                ghost.first_data_block,
                                ghost.data_length,
                                ghost.data_crc32,
                                ghost.physical_sector,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Find an unused header sector to use as the ghost sector.
    ///
    /// After crashes, both physical sectors might be Valid (no Ghost state found),
    /// so we must explicitly pick the unused one as ghost.
    fn find_unused_header_sector(&self) -> u16 {
        let used_header_sectors: Vec<u16> = self
            .slot_info
            .iter()
            .map(|info| info.header_sector)
            .collect();

        for sector in 1..=(self.num_slots + 1) as u16 {
            if !used_header_sectors.contains(&sector) {
                return sector;
            }
        }

        // Fallback to last sector (shouldn't happen in normal operation)
        (self.num_slots + 1) as u16
    }

    /// Build the free sector list by scanning data chains from valid slots.
    ///
    /// If a slot's data chain is corrupted (invalid reference, loop, or bad block),
    /// that slot is marked as corrupted and its sectors are added to the free list.
    fn build_free_sector_list(&mut self) -> Result<(), SaveError<Storage::Error>> {
        self.free_sector_list.clear();

        // Sector layout: [0: global] [1..num_slots+1: slot headers] [num_slots+1: ghost] [num_slots+2..: data]
        let first_data_sector = self.num_slots + 2;
        let sector_count = self.storage.sector_count();
        let mut used_sectors = vec![false; sector_count];

        // Mark header sectors as used
        for used in used_sectors.iter_mut().take(first_data_sector) {
            *used = true;
        }

        let sector_size = self.storage.sector_size();
        let mut buffer = vec![0u8; sector_size];

        // Track sectors claimed by this slot - only commit if chain is valid
        let mut slot_sectors: Vec<usize> = Vec::new();

        for slot_info in self.slot_info.iter_mut() {
            if slot_info.status != SlotStatus::Valid {
                continue;
            }

            let mut current_sector = slot_info.first_data_block;
            let mut slot_valid = true;
            slot_sectors.clear();

            while let Some(sector) = current_sector {
                let sector_idx = sector as usize;

                // Check for invalid reference or loop
                if sector_idx >= sector_count || slot_sectors.contains(&sector_idx) {
                    slot_valid = false;
                    break;
                }

                slot_sectors.push(sector_idx);

                // Read the sector to find the next block in the chain
                self.storage.read_sector(sector_idx, &mut buffer)?;

                match deserialize_block(&buffer) {
                    Ok(Block::Data(data_block)) => {
                        current_sector = data_block.next_block();
                    }
                    _ => {
                        slot_valid = false;
                        break;
                    }
                }
            }

            if slot_valid {
                // Chain is valid - mark its sectors as used
                for sector_idx in slot_sectors.drain(..) {
                    used_sectors[sector_idx] = true;
                }
            } else {
                // Chain is corrupted - mark slot as corrupted and let its sectors go on the
                // free list
                slot_info.status = SlotStatus::Corrupted;
            }
        }

        // Collect free sectors
        self.free_sector_list.clear();
        for (sector, &used) in used_sectors.iter().enumerate() {
            if !used {
                self.free_sector_list.push(sector as u16);
            }
        }

        Ok(())
    }

    /// Write data across multiple data blocks, allocating sectors from the free list.
    ///
    /// Returns the index of the first data block, or 0xFFFF if data is empty.
    /// Returns an error if there aren't enough free sectors.
    fn write_data_blocks(&mut self, data: &[u8]) -> Result<Option<u16>, SaveError<Storage::Error>> {
        if data.is_empty() {
            return Ok(None);
        }

        let sector_size = self.storage.sector_size();
        let payload_size = sector_size - DataBlock::header_size();

        // Calculate how many sectors we need
        let sectors_needed = data.len().div_ceil(payload_size);

        // Check if we have enough free sectors
        if self.free_sector_list.len() < sectors_needed {
            return Err(SaveError::OutOfSpace);
        }

        // Allocate sectors from the free list
        let allocated_sectors: Vec<u16> = self
            .free_sector_list
            .drain(self.free_sector_list.len() - sectors_needed..)
            .collect();

        // Write data blocks
        let mut buffer = vec![0u8; sector_size];
        let mut padded_data = vec![0u8; payload_size];
        let mut data_offset = 0;

        for (i, &sector) in allocated_sectors.iter().enumerate() {
            let is_last = i == allocated_sectors.len() - 1;
            let next_block = if is_last {
                None
            } else {
                Some(allocated_sectors[i + 1])
            };

            // Calculate how much data goes in this block
            let chunk_size = (data.len() - data_offset).min(payload_size);
            let chunk = &data[data_offset..data_offset + chunk_size];

            // Copy data and zero-pad the rest
            padded_data[..chunk_size].copy_from_slice(chunk);
            padded_data[chunk_size..].fill(0);

            // Serialize the data block
            buffer.fill(0);
            serialize_block(
                Block::Data(DataBlock::new(next_block, &padded_data)),
                &mut buffer,
            );

            // Write to storage
            self.storage.write_sector(sector as usize, &buffer)?;

            data_offset += chunk_size;
        }

        Ok(Some(allocated_sectors[0]))
    }

    /// Reads data from a chain of data blocks, appending to the provided buffer.
    ///
    /// Follows the block chain starting from `start_block`, reading up to `size` bytes
    /// total (including any data already in the buffer). Stops when either there's no
    /// next block or the target size has been reached.
    fn read_block_chain(
        &mut self,
        start_block: Option<u16>,
        data: &mut Vec<u8>,
        size: usize,
    ) -> Result<(), SaveError<Storage::Error>> {
        let sector_size = self.storage.sector_size();
        let payload_size = sector_size - DataBlock::header_size();
        let mut buffer = vec![0u8; sector_size];

        let mut current_block = start_block;
        let max_blocks = self.storage.sector_count();
        let mut blocks_read = 0;

        while let Some(block) = current_block {
            // Check if we've read enough
            if data.len() >= size {
                break;
            }

            // Prevent infinite loops
            blocks_read += 1;
            if blocks_read > max_blocks {
                return Err(SaveError::SlotCorrupted);
            }

            // Read the block
            self.storage.read_sector(block as usize, &mut buffer)?;

            // Deserialize and extract data
            match deserialize_block(&buffer) {
                Ok(Block::Data(data_block)) => {
                    // Append payload (up to what we need)
                    let remaining = size.saturating_sub(data.len());
                    let to_copy = remaining.min(payload_size);
                    data.extend_from_slice(&data_block.data[..to_copy]);

                    current_block = data_block.next_block();
                }
                _ => return Err(SaveError::SlotCorrupted),
            }
        }

        Ok(())
    }

    fn read_slot_data<T>(&mut self, slot: usize) -> Result<T, SaveError<Storage::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let slot_info = &self.slot_info[slot];
        let first_data_block = slot_info.first_data_block;
        let data_length = slot_info.data_length;
        let expected_crc32 = slot_info.data_crc32;

        // Handle empty data case
        if first_data_block.is_none() {
            if data_length == 0 {
                // Try to deserialize empty slice (works for unit type, empty structs, etc.)
                return postcard::from_bytes(&[]).map_err(SaveError::from_postcard_serialization);
            } else {
                // No data blocks but non-zero length - corrupted
                return Err(SaveError::SlotCorrupted);
            }
        }

        let mut data = Vec::with_capacity(data_length as usize);
        self.read_block_chain(first_data_block, &mut data, data_length as usize)?;

        verify_and_deserialize_data(&data, data_length, expected_crc32)
            .map_err(SaveError::from_data_verify_error)
    }

    fn write_slot_data<T>(
        &mut self,
        slot: usize,
        data: &T,
        metadata: &Metadata,
    ) -> Result<(), SaveError<Storage::Error>>
    where
        T: serde::Serialize,
    {
        // 1. Serialize data first (before we start modifying storage)
        let data_bytes =
            postcard::to_allocvec(data).map_err(SaveError::from_postcard_serialization)?;

        // 2. Compute checksum of the data
        let data_crc32 = calc_crc32(&data_bytes);
        let data_length = data_bytes.len() as u32;

        // 3. Write the data chain (this happens first for crash safety)
        let first_data_block = self.write_data_blocks(&data_bytes)?;

        // 4. Serialize metadata
        let sector_size = self.storage.sector_size();
        let metadata_size = sector_size - SlotHeaderBlock::header_size();
        let mut metadata_bytes = vec![0u8; metadata_size];
        let serialized_metadata = postcard::to_slice(metadata, &mut metadata_bytes)
            .map_err(SaveError::from_postcard_serialization)?;
        let metadata_length = serialized_metadata.len() as u32;
        let metadata_crc32 = calc_crc32(serialized_metadata);

        // Increment generation and save old slot info for later cleanup
        let new_generation = self.slot_info[slot].generation.wrapping_add(1);
        let old_header_sector = self.slot_info[slot].header_sector;
        let old_first_data_block = self.slot_info[slot].first_data_block;

        let mut buffer = vec![0u8; sector_size];

        // Write new header to the current ghost sector
        serialize_block(
            Block::SlotHeader(SlotHeaderBlock::valid(
                slot as u8,
                first_data_block,
                None, // first_metadata_block - no spillover for now
                new_generation,
                data_crc32,
                data_length,
                metadata_length,
                metadata_crc32,
                &metadata_bytes,
            )),
            &mut buffer,
        );

        self.storage
            .write_sector(self.ghost_sector as usize, &buffer)?;

        // Mark old header as ghost
        self.storage
            .read_sector(old_header_sector as usize, &mut buffer)?;

        if let Ok(Block::SlotHeader(old_block)) = deserialize_block(&buffer) {
            // Extract data we need before reusing buffer
            let old_metadata = old_block.metadata().to_vec();
            let ghost_block = SlotHeaderBlock::valid(
                old_block.logical_slot_id(),
                old_block.first_data_block(),
                old_block.first_metadata_block(),
                old_block.generation(),
                old_block.crc32(),
                old_block.length(),
                old_block.metadata_length(),
                old_block.metadata_crc32(),
                &old_metadata,
            )
            .with_state(SlotState::Ghost);

            serialize_block(Block::SlotHeader(ghost_block), &mut buffer);
            self.storage
                .write_sector(old_header_sector as usize, &buffer)?;
        }

        // The old header sector is now the ghost, the ghost sector now holds our new header
        let new_header_sector = self.ghost_sector;
        self.ghost_sector = old_header_sector;

        // 8. Return the old ghost's data sectors to the free list
        self.free_data_chain(old_first_data_block, &mut buffer)?;

        // Update in-memory state
        self.slot_info[slot] = SlotInfo::valid(
            metadata.clone(),
            new_generation,
            first_data_block,
            data_length,
            data_crc32,
            new_header_sector,
        );

        Ok(())
    }

    /// Traverse a data block chain and return all sectors to the free list.
    fn free_data_chain(
        &mut self,
        first_block: Option<u16>,
        buffer: &mut [u8],
    ) -> Result<(), SaveError<Storage::Error>> {
        let mut current_block = first_block;

        while let Some(block) = current_block {
            // Read the block to find the next one in the chain
            self.storage.read_sector(block as usize, buffer)?;

            let next_block = match deserialize_block(buffer) {
                Ok(Block::Data(data_block)) => data_block.next_block(),
                _ => None, // Stop if block is corrupted
            };

            // Return this sector to the free list
            self.free_sector_list.push(block);

            current_block = next_block;
        }

        Ok(())
    }

    fn erase_slot(&mut self, slot: usize) -> Result<(), SaveError<Storage::Error>> {
        let sector_size = self.storage.sector_size();
        let metadata_size = sector_size - SlotHeaderBlock::header_size();

        // Save old info for cleanup
        let old_first_data_block = self.slot_info[slot].first_data_block;
        let header_sector = self.slot_info[slot].header_sector;
        let new_generation = self.slot_info[slot].generation.wrapping_add(1);

        let mut buffer = vec![0u8; sector_size];
        let empty_metadata = vec![0u8; metadata_size];

        // 1. Write slot header with state = Empty
        serialize_block(
            Block::SlotHeader(SlotHeaderBlock::empty_with_generation(
                slot as u8,
                new_generation,
                &empty_metadata,
            )),
            &mut buffer,
        );

        self.storage.write_sector(header_sector as usize, &buffer)?;

        // 2. Return data sectors to free list
        self.free_data_chain(old_first_data_block, &mut buffer)?;

        // 3. Update in-memory state
        self.slot_info[slot] = SlotInfo::empty(new_generation, header_sector);

        Ok(())
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
