extern crate std;

use std::vec::Vec;

use crate::{StorageInfo, StorageMedium};

/// A test implementation of [`StorageMedium`] backed by an in-memory buffer.
///
/// This implementation includes assertions to validate that all operations
/// respect the alignment and size requirements specified in [`StorageInfo`].
pub struct TestStorage {
    data: Vec<u8>,
    info: StorageInfo,
    /// Tracks which regions have been erased (for media that require erase before write).
    /// Each bit represents one erase_size block: 1 = erased, 0 = not erased.
    erased_blocks: Vec<bool>,
    /// Number of writes performed so far.
    write_count: usize,
    /// If set, writes will fail after this many successful writes.
    fail_after_writes: Option<usize>,
}

/// Errors that can occur in [`TestStorage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStorageError {
    /// Attempted to read/write/erase out of bounds.
    OutOfBounds,
    /// Attempted to write to a region that hasn't been erased.
    NotErased,
    /// Simulated write failure for testing crash scenarios.
    SimulatedFailure,
}

impl TestStorage {
    /// Create a new test storage with the given configuration.
    ///
    /// The storage is initialized to all 0xFF bytes (typical for flash memory).
    pub fn new(info: StorageInfo) -> Self {
        let data = std::vec![0xFF; info.size];

        // Calculate number of erase blocks (if erase is required)
        let num_erase_blocks = if let Some(erase_size) = info.erase_size {
            info.size.div_ceil(erase_size.get())
        } else {
            0
        };

        Self {
            data,
            info,
            // Start with all blocks "erased" for convenience in simple tests
            erased_blocks: std::vec![true; num_erase_blocks],
            write_count: 0,
            fail_after_writes: None,
        }
    }

    /// Create a new test storage that simulates byte-addressable SRAM.
    ///
    /// No erase required, single-byte writes allowed.
    pub fn new_sram(size: usize) -> Self {
        use core::num::NonZeroUsize;
        Self::new(StorageInfo {
            size,
            erase_size: None,
            write_size: NonZeroUsize::new(1).unwrap(),
        })
    }

    /// Create a new test storage that simulates flash memory.
    ///
    /// Requires erase before write, with configurable block sizes.
    pub fn new_flash(size: usize, erase_size: usize, write_size: usize) -> Self {
        use core::num::NonZeroUsize;
        Self::new(StorageInfo {
            size,
            erase_size: NonZeroUsize::new(erase_size),
            write_size: NonZeroUsize::new(write_size).unwrap(),
        })
    }

    /// Reset all erase tracking, marking all blocks as not erased.
    ///
    /// Useful for testing that code properly erases before writing.
    pub fn reset_erase_state(&mut self) {
        for block in &mut self.erased_blocks {
            *block = false;
        }
    }

    /// Configure the storage to fail writes after a given number of successful writes.
    ///
    /// This is useful for testing crash/failure scenarios during save operations.
    /// Pass `None` to disable simulated failures.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut storage = TestStorage::new_sram(1024);
    /// storage.fail_after_writes(2); // Fail on the 3rd write
    /// storage.write(0, &[1]).unwrap(); // OK (write 1)
    /// storage.write(1, &[2]).unwrap(); // OK (write 2)
    /// storage.write(2, &[3]).unwrap_err(); // Fails (write 3)
    /// ```
    pub fn fail_after_writes(&mut self, count: Option<usize>) {
        self.fail_after_writes = count;
        self.write_count = 0;
    }

    /// Returns the number of writes performed since the last reset or creation.
    pub fn write_count(&self) -> usize {
        self.write_count
    }

    /// Get direct access to the underlying data for test verification.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable access to the underlying data for test setup.
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Check if a specific erase block has been erased.
    pub fn is_block_erased(&self, block_index: usize) -> bool {
        self.erased_blocks.get(block_index).copied().unwrap_or(true)
    }

    fn check_erase_alignment(&self, offset: usize, len: usize) {
        if let Some(erase_size) = self.info.erase_size {
            let erase_size = erase_size.get();
            assert!(
                offset.is_multiple_of(erase_size),
                "erase offset {offset} is not aligned to erase_size {erase_size}"
            );
            assert!(
                len.is_multiple_of(erase_size),
                "erase length {len} is not aligned to erase_size {erase_size}"
            );
        }
    }

    fn check_write_alignment(&self, offset: usize, len: usize) {
        let write_size = self.info.write_size.get();
        assert!(
            offset.is_multiple_of(write_size),
            "write offset {offset} is not aligned to write_size {write_size}"
        );
        assert!(
            len.is_multiple_of(write_size),
            "write length {len} is not aligned to write_size {write_size}"
        );
    }

    fn check_bounds(&self, offset: usize, len: usize) -> Result<(), TestStorageError> {
        if offset.saturating_add(len) > self.info.size {
            return Err(TestStorageError::OutOfBounds);
        }
        Ok(())
    }

    fn check_erased(&self, offset: usize, len: usize) -> Result<(), TestStorageError> {
        if let Some(erase_size) = self.info.erase_size {
            let erase_size = erase_size.get();
            let start_block = offset / erase_size;
            let end_block = (offset + len).div_ceil(erase_size);

            for block in start_block..end_block {
                if !self.erased_blocks.get(block).copied().unwrap_or(false) {
                    return Err(TestStorageError::NotErased);
                }
            }
        }
        Ok(())
    }
}

impl StorageMedium for TestStorage {
    type Error = TestStorageError;

    fn info(&self) -> StorageInfo {
        self.info
    }

    fn read(&mut self, offset: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.check_bounds(offset, buf.len())?;
        buf.copy_from_slice(&self.data[offset..offset + buf.len()]);
        Ok(())
    }

    fn erase(&mut self, offset: usize, len: usize) -> Result<(), Self::Error> {
        // No-op if erase is not required
        if self.info.erase_size.is_none() {
            return Ok(());
        }

        self.check_bounds(offset, len)?;
        self.check_erase_alignment(offset, len);

        // Fill with 0xFF (typical erased state for flash)
        self.data[offset..offset + len].fill(0xFF);

        // Mark blocks as erased
        let erase_size = self.info.erase_size.unwrap().get();
        let start_block = offset / erase_size;
        let end_block = (offset + len) / erase_size;
        for block in start_block..end_block {
            if let Some(erased) = self.erased_blocks.get_mut(block) {
                *erased = true;
            }
        }

        Ok(())
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
        // Check for simulated failure
        if let Some(limit) = self.fail_after_writes {
            if self.write_count >= limit {
                return Err(TestStorageError::SimulatedFailure);
            }
        }

        self.check_bounds(offset, data.len())?;
        self.check_write_alignment(offset, data.len());
        self.check_erased(offset, data.len())?;

        self.data[offset..offset + data.len()].copy_from_slice(data);

        // Mark affected blocks as no longer erased (written to)
        if let Some(erase_size) = self.info.erase_size {
            let erase_size = erase_size.get();
            let start_block = offset / erase_size;
            let end_block = (offset + data.len()).div_ceil(erase_size);
            for block in start_block..end_block {
                if let Some(erased) = self.erased_blocks.get_mut(block) {
                    *erased = false;
                }
            }
        }

        self.write_count += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sram_read_write() {
        let mut storage = TestStorage::new_sram(1024);

        // Write some data
        storage.write(0, &[1, 2, 3, 4]).unwrap();
        storage.write(100, &[5, 6, 7, 8]).unwrap();

        // Read it back
        let mut buf = [0u8; 4];
        storage.read(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3, 4]);

        storage.read(100, &mut buf).unwrap();
        assert_eq!(buf, [5, 6, 7, 8]);
    }

    #[test]
    fn sram_out_of_bounds() {
        let mut storage = TestStorage::new_sram(100);

        // Write at the end - should succeed
        storage.write(96, &[1, 2, 3, 4]).unwrap();

        // Write past the end - should fail
        let result = storage.write(97, &[1, 2, 3, 4]);
        assert_eq!(result, Err(TestStorageError::OutOfBounds));

        // Read past the end - should fail
        let mut buf = [0u8; 4];
        let result = storage.read(97, &mut buf);
        assert_eq!(result, Err(TestStorageError::OutOfBounds));
    }

    #[test]
    fn flash_requires_erase() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);
        storage.reset_erase_state();

        // Writing without erase should fail
        let result = storage.write(0, &[1, 2, 3, 4]);
        assert_eq!(result, Err(TestStorageError::NotErased));

        // Erase first, then write should succeed
        storage.erase(0, 256).unwrap();
        storage.write(0, &[1, 2, 3, 4]).unwrap();

        // Read it back
        let mut buf = [0u8; 4];
        storage.read(0, &mut buf).unwrap();
        assert_eq!(buf, [1, 2, 3, 4]);
    }

    #[test]
    #[should_panic(expected = "erase offset")]
    fn flash_erase_alignment_offset() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);
        // Misaligned erase offset should panic
        let _ = storage.erase(100, 256);
    }

    #[test]
    #[should_panic(expected = "erase length")]
    fn flash_erase_alignment_length() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);
        // Misaligned erase length should panic
        let _ = storage.erase(0, 100);
    }

    #[test]
    #[should_panic(expected = "write offset")]
    fn flash_write_alignment_offset() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);
        storage.erase(0, 256).unwrap();
        // Misaligned write offset should panic
        let _ = storage.write(1, &[1, 2, 3, 4]);
    }

    #[test]
    #[should_panic(expected = "write length")]
    fn flash_write_alignment_length() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);
        storage.erase(0, 256).unwrap();
        // Misaligned write length should panic
        let _ = storage.write(0, &[1, 2, 3]);
    }

    #[test]
    fn flash_erase_fills_with_ff() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);

        // Write some data first (storage starts erased)
        storage.write(0, &[1, 2, 3, 4]).unwrap();

        // Erase the block
        storage.erase(0, 256).unwrap();

        // Verify it's filled with 0xFF
        let mut buf = [0u8; 4];
        storage.read(0, &mut buf).unwrap();
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn flash_write_after_write_fails() {
        let mut storage = TestStorage::new_flash(1024, 256, 4);

        // First write succeeds (storage starts erased)
        storage.write(0, &[1, 2, 3, 4]).unwrap();

        // Second write to same block should fail (block no longer erased)
        let result = storage.write(4, &[5, 6, 7, 8]);
        assert_eq!(result, Err(TestStorageError::NotErased));

        // Re-erase and try again
        storage.erase(0, 256).unwrap();
        storage.write(4, &[5, 6, 7, 8]).unwrap();
    }

    #[test]
    fn simulated_write_failure() {
        let mut storage = TestStorage::new_sram(1024);

        // Configure to fail after 2 writes
        storage.fail_after_writes(Some(2));

        // First two writes succeed
        storage.write(0, &[1]).unwrap();
        assert_eq!(storage.write_count(), 1);

        storage.write(1, &[2]).unwrap();
        assert_eq!(storage.write_count(), 2);

        // Third write fails
        let result = storage.write(2, &[3]);
        assert_eq!(result, Err(TestStorageError::SimulatedFailure));
        assert_eq!(storage.write_count(), 2); // Count doesn't increase on failure

        // Disable failure and writes work again
        storage.fail_after_writes(None);
        storage.write(2, &[3]).unwrap();
    }
}
