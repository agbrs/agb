//! Module for battery backed SRAM save media support.
//!
//! SRAM acts as ordinary memory mapped into the memory space, and as such
//! is accessed using normal memory read/write commands.

use crate::save::asm_utils::*;
use crate::save::utils::Timeout;
use crate::save::{MediaInfo, MediaType, RawSaveAccess, StorageError};

const SRAM_SIZE: usize = 32 * 1024; // 32 KiB

/// Checks whether an offset is contained within the bounds of the SRAM.
fn check_bounds(offset: usize, len: usize) -> Result<(), StorageError> {
    if offset.checked_add(len).is_none() || offset + len > SRAM_SIZE {
        return Err(StorageError::OutOfBounds);
    }
    Ok(())
}

/// The [`RawSaveAccess`] used for battery backed SRAM.
pub struct BatteryBackedAccess;
impl RawSaveAccess for BatteryBackedAccess {
    fn info(&self) -> Result<&'static MediaInfo, StorageError> {
        Ok(&MediaInfo {
            media_type: MediaType::Sram32K,
            sector_shift: 0,
            sector_count: SRAM_SIZE,
            uses_prepare_write: false,
        })
    }

    fn read(&self, offset: usize, buffer: &mut [u8], _: &mut Timeout) -> Result<(), StorageError> {
        check_bounds(offset, buffer.len())?;
        unsafe {
            read_raw_buf(buffer, 0x0E000000 + offset);
        }
        Ok(())
    }

    fn verify(&self, offset: usize, buffer: &[u8], _: &mut Timeout) -> Result<bool, StorageError> {
        check_bounds(offset, buffer.len())?;
        let val = unsafe { verify_raw_buf(buffer, 0x0E000000 + offset) };
        Ok(val)
    }

    fn prepare_write(&self, _: usize, _: usize, _: &mut Timeout) -> Result<(), StorageError> {
        Ok(())
    }

    fn write(&self, offset: usize, buffer: &[u8], _: &mut Timeout) -> Result<(), StorageError> {
        check_bounds(offset, buffer.len())?;
        unsafe {
            write_raw_buf(0x0E000000 + offset, buffer);
        }
        Ok(())
    }
}
