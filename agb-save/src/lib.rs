//! This crate provides a storage agnostic way to save any serde serialize / deserialize
//! data into save slots like you would find in a classic RPG.
//!
//! It is opinionated on how the data should be stored and accessed, and expects you to be
//! able to load an entire save slot into RAM at once.
#![no_std]
#![warn(clippy::all)]
#![warn(missing_docs)]

use core::num::NonZeroUsize;

#[cfg(test)]
mod test;

mod block;

#[derive(Debug, Clone, Copy)]
/// Data about how the [`StorageMedium`] should be used.
pub struct StorageInfo {
    /// Total size in bytes
    pub size: usize,
    /// Minimum erase size in bytes, None if no explicit erase is required
    pub erase_size: Option<NonZeroUsize>,
    /// Minimum write size, 1 if byte-addressable
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

/// A save slot manager which gives you some level of write safety and confirmation
/// that everything worked correctly.
pub struct SaveSlotManager<Storage: StorageMedium> {
    num_slots: usize,
    storage: Storage,
    magic: [u8; 32],
}

impl<Storage: StorageMedium> SaveSlotManager<Storage> {
    /// Create a new instance of the SaveSlotManager.
    ///
    /// Will read from `storage` to initialise itself. If the top header is detected as corrupted
    /// and the backup one is too you'll get a blank initialised SaveSlotManager because it'll
    /// assume that we've got a brand new file. Writes will happen to the Storage as some initialisation
    /// has to happen.
    ///
    /// If the save data was corrupted in some way, but it was recoverable, this will be handled
    /// without any errors.
    ///
    /// This fully verifies the save data before returning, and you can retrieve the status of
    /// the various save slots by querying them with the methods on this at a later point.
    ///
    /// * `num_slots` is the number of save slots in this save file
    /// * `magic` is a magic number you can use to check if this is a save file for your game.
    ///   If the stored magic number doesn't match the one provided here, we assume full corruption
    ///   and reinitialise the save file.
    pub fn new(storage: Storage, num_slots: usize, magic: [u8; 32]) -> Self {
        Self {
            num_slots,
            storage,
            magic,
        }
    }
}
