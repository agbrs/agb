//! Sector-level storage abstraction.
//!
//! This module provides [`SectorStorage`], which wraps a [`StorageMedium`] and provides
//! a simpler sector-oriented API. This handles:
//! - Automatic erase-before-write for flash media
//! - Sector size calculation based on storage constraints
//! - Alignment requirements

use crate::StorageMedium;

/// Minimum sector size to ensure there's enough space for headers and useful data.
pub const MIN_SECTOR_SIZE: usize = 128;

/// A sector-oriented wrapper around [`StorageMedium`].
///
/// This abstraction simplifies storage access by:
/// - Calculating an appropriate sector size based on storage constraints
/// - Handling erase operations automatically before writes
/// - Providing a simple read/write sector API
///
/// # Sector Size Calculation
///
/// The sector size is calculated as:
/// ```text
/// sector_size = max(MIN_SECTOR_SIZE, erase_size, write_size)
/// ```
/// rounded up to be a multiple of both `erase_size` and `write_size`.
///
/// This ensures:
/// - Sectors are large enough for the save format (at least 128 bytes)
/// - Each sector can be erased independently (aligned to erase_size)
/// - Each sector can be written in one operation (aligned to write_size)
pub struct SectorStorage<S: StorageMedium> {
    storage: S,
    sector_size: usize,
    sector_count: usize,
}

impl<S: StorageMedium> SectorStorage<S> {
    /// Create a new sector storage wrapper.
    ///
    /// Calculates the sector size based on the storage's constraints and
    /// determines how many sectors fit in the available space.
    pub fn new(storage: S) -> Self {
        let info = storage.info();

        // Start with minimum sector size
        let mut sector_size = MIN_SECTOR_SIZE;

        // Sector size must be at least erase_size (if erase is required)
        if let Some(erase_size) = info.erase_size {
            sector_size = sector_size.max(erase_size.get());
        }

        // Sector size must be at least write_size
        sector_size = sector_size.max(info.write_size.get());

        // Round up to be a multiple of write_size
        let write_size = info.write_size.get();
        sector_size = sector_size.div_ceil(write_size) * write_size;

        // Round up to be a multiple of erase_size (if required)
        if let Some(erase_size) = info.erase_size {
            let erase_size = erase_size.get();
            sector_size = sector_size.div_ceil(erase_size) * erase_size;
        }

        let sector_count = info.size / sector_size;

        Self {
            storage,
            sector_size,
            sector_count,
        }
    }

    /// Returns the sector size in bytes.
    pub fn sector_size(&self) -> usize {
        self.sector_size
    }

    /// Returns the total number of sectors available.
    pub fn sector_count(&self) -> usize {
        self.sector_count
    }

    /// Read a sector into the provided buffer.
    ///
    /// # Panics
    ///
    /// Panics if `sector_index >= sector_count()` or if `buf.len() != sector_size()`.
    pub fn read_sector(&mut self, sector_index: usize, buf: &mut [u8]) -> Result<(), S::Error> {
        assert!(
            sector_index < self.sector_count,
            "sector index {sector_index} out of bounds (sector_count = {})",
            self.sector_count
        );
        assert_eq!(
            buf.len(),
            self.sector_size,
            "buffer length {} does not match sector size {}",
            buf.len(),
            self.sector_size
        );

        let offset = sector_index * self.sector_size;
        self.storage.read(offset, buf)
    }

    /// Write a sector, automatically erasing first if required.
    ///
    /// # Panics
    ///
    /// Panics if `sector_index >= sector_count()` or if `data.len() != sector_size()`.
    pub fn write_sector(&mut self, sector_index: usize, data: &[u8]) -> Result<(), S::Error> {
        assert!(
            sector_index < self.sector_count,
            "sector index {sector_index} out of bounds (sector_count = {})",
            self.sector_count
        );
        assert_eq!(
            data.len(),
            self.sector_size,
            "data length {} does not match sector size {}",
            data.len(),
            self.sector_size
        );

        let offset = sector_index * self.sector_size;

        // Erase the sector first (no-op for SRAM-like storage)
        self.storage.erase(offset, self.sector_size)?;

        // Write the data
        self.storage.write(offset, data)
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use super::*;
    use crate::test_storage::TestStorage;

    #[test]
    fn sector_size_sram() {
        // SRAM: no erase, 1-byte writes
        let storage = TestStorage::new_sram(1024);
        let sector_storage = SectorStorage::new(storage);

        // Should use minimum sector size
        assert_eq!(sector_storage.sector_size(), 128);
        assert_eq!(sector_storage.sector_count(), 8); // 1024 / 128
    }

    #[test]
    fn sector_size_flash_small_erase() {
        // Flash with small erase size
        let storage = TestStorage::new_flash(1024, 64, 4);
        let sector_storage = SectorStorage::new(storage);

        // Should round up to 128 (minimum), which is a multiple of 64
        assert_eq!(sector_storage.sector_size(), 128);
        assert_eq!(sector_storage.sector_count(), 8);
    }

    #[test]
    fn sector_size_flash_large_erase() {
        // Flash with large erase size
        let storage = TestStorage::new_flash(4096, 512, 4);
        let sector_storage = SectorStorage::new(storage);

        // Should use erase size since it's larger than minimum
        assert_eq!(sector_storage.sector_size(), 512);
        assert_eq!(sector_storage.sector_count(), 8); // 4096 / 512
    }

    #[test]
    fn read_write_sector() {
        let storage = TestStorage::new_sram(1024);
        let mut sector_storage = SectorStorage::new(storage);

        let sector_size = sector_storage.sector_size();

        // Write a sector
        let mut write_data = alloc::vec![0u8; sector_size];
        write_data[0] = 0xAB;
        write_data[1] = 0xCD;
        write_data[sector_size - 1] = 0xEF;

        sector_storage.write_sector(0, &write_data).unwrap();

        // Read it back
        let mut read_data = alloc::vec![0u8; sector_size];
        sector_storage.read_sector(0, &mut read_data).unwrap();

        assert_eq!(read_data, write_data);
    }

    #[test]
    fn write_sector_erases_first() {
        let storage = TestStorage::new_flash(1024, 128, 4);
        let mut sector_storage = SectorStorage::new(storage);

        let sector_size = sector_storage.sector_size();

        // Write first sector
        let data1 = alloc::vec![0x11u8; sector_size];
        sector_storage.write_sector(0, &data1).unwrap();

        // Write again to same sector - should work because write_sector erases first
        let data2 = alloc::vec![0x22u8; sector_size];
        sector_storage.write_sector(0, &data2).unwrap();

        // Verify second write succeeded
        let mut read_data = alloc::vec![0u8; sector_size];
        sector_storage.read_sector(0, &mut read_data).unwrap();
        assert_eq!(read_data, data2);
    }

    #[test]
    fn multiple_sectors() {
        let storage = TestStorage::new_sram(1024);
        let mut sector_storage = SectorStorage::new(storage);

        let sector_size = sector_storage.sector_size();
        let sector_count = sector_storage.sector_count();

        // Write different data to each sector
        for i in 0..sector_count {
            let mut data = alloc::vec![i as u8; sector_size];
            data[0] = i as u8;
            sector_storage.write_sector(i, &data).unwrap();
        }

        // Read back and verify
        for i in 0..sector_count {
            let mut data = alloc::vec![0u8; sector_size];
            sector_storage.read_sector(i, &mut data).unwrap();
            assert_eq!(data[0], i as u8);
        }
    }

    #[test]
    #[should_panic(expected = "sector index")]
    fn read_sector_out_of_bounds() {
        let storage = TestStorage::new_sram(1024);
        let mut sector_storage = SectorStorage::new(storage);

        let mut buf = alloc::vec![0u8; sector_storage.sector_size()];
        let _ = sector_storage.read_sector(100, &mut buf);
    }

    #[test]
    #[should_panic(expected = "sector index")]
    fn write_sector_out_of_bounds() {
        let storage = TestStorage::new_sram(1024);
        let mut sector_storage = SectorStorage::new(storage);

        let data = alloc::vec![0u8; sector_storage.sector_size()];
        let _ = sector_storage.write_sector(100, &data);
    }

    #[test]
    #[should_panic(expected = "buffer length")]
    fn read_sector_wrong_buffer_size() {
        let storage = TestStorage::new_sram(1024);
        let mut sector_storage = SectorStorage::new(storage);

        let mut buf = alloc::vec![0u8; 64]; // Wrong size
        let _ = sector_storage.read_sector(0, &mut buf);
    }

    #[test]
    #[should_panic(expected = "data length")]
    fn write_sector_wrong_data_size() {
        let storage = TestStorage::new_sram(1024);
        let mut sector_storage = SectorStorage::new(storage);

        let data = alloc::vec![0u8; 64]; // Wrong size
        let _ = sector_storage.write_sector(0, &data);
    }
}
