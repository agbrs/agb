//! Module for reading and writing to save media.
//!
//! ## Save media types
//!
//! There are, broadly speaking, three different kinds of save media that can be
//! found in official Game Carts:
//!
//! * Battery-Backed SRAM: The simplest kind of save media, which can be
//!   accessed like normal memory. You can have SRAM up to 32KiB, and while
//!   there exist a few variants this does not matter much for a game developer.
//! * EEPROM: A kind of save media based on very cheap chips and slow chips.
//!   These are accessed using a serial interface based on reading/writing bit
//!   streams into IO registers. This memory comes in 8KiB and 512 byte
//!   versions, which unfortunately cannot be distinguished at runtime.
//! * Flash: A kind of save media based on flash memory. Flash memory can be
//!   read like ordinary memory, but writing requires sending commands using
//!   multiple IO register spread across the address space. This memory comes in
//!   64KiB and 128KiB variants, which can thankfully be distinguished using a
//!   chip ID.
//!
//! As these various types of save media cannot be easily distinguished at
//! runtime, the kind of media in use should be set manually.
//!
//! ## Setting save media type
//!
//! To use save media in your game, you must set which type to use. This is done
//! by calling one of the following functions at startup:
//!
//! * For 32 KiB battery-backed SRAM, call [`init_sram`].
//! * For 64 KiB flash memory, call [`init_flash_64k`].
//! * For 128 KiB flash memory, call [`init_flash_128k`].
//! * For 512 byte EEPROM, call [`init_eeprom_512b`].
//! * For 8 KiB EEPROM, call [`init_eeprom_8k`].
//!
//! [`init_sram`]: SaveManager::init_sram
//! [`init_flash_64k`]: SaveManager::init_flash_64k
//! [`init_flash_128k`]: SaveManager::init_flash_128k
//! [`init_eeprom_512b`]: SaveManager::init_eeprom_512b
//! [`init_eeprom_8k`]: SaveManager::init_eeprom_8k
//!
//! ## Using save media
//!
//! To access save media, use the [`SaveManager::access`] or
//! [`SaveManager::access_with_timer`] methods to create a new [`SaveData`]
//! object. Its methods are used to read or write save media.
//!
//! Reading data from the save media is simple. Use [`read`] to copy data from an
//! offset in the save media into a buffer in memory.
//!
//! Writing to save media requires you to prepare the area for writing by
//! calling the [`prepare_write`] method to return a [`SavePreparedBlock`],
//! which contains the actual [`write`] method.
//!
//! The `prepare_write` method leaves everything in a sector that overlaps the
//! range passed to it in an implementation defined state. On some devices it
//! may do nothing, and on others, it may clear the entire range to `0xFF`.
//!
//! Because writes can only be prepared on a per-sector basis, a clear on a
//! range of `4000..5000` on a device with 4096 byte sectors will actually clear
//! a range of `0..8192`. Use [`sector_size`] to find the sector size, or
//! [`align_range`] to directly calculate the range of memory that will be
//! affected by the clear.
//!
//! [`read`]: SaveData::read
//! [`prepare_write`]: SaveData::prepare_write
//! [`write`]: SavePreparedBlock::write
//! [`sector_size`]: SaveData::sector_size
//! [`align_range`]: SaveData::align_range
//!
//! ## Performance and Other Details
//!
//! The performance characteristics of the media types are as follows:
//!
//! * SRAM is simply a form of battery backed memory, and has no particular
//!   performance characteristics.  Reads and writes at any alignment are
//!   efficient. Furthermore, no timer is needed for accesses to this type of
//!   media. `prepare_write` does not immediately erase any data.
//! * Non-Atmel flash chips have a sector size of 4096 bytes. Reads and writes
//!   to any alignment are efficient, however, `prepare_write` will erase all
//!   data in an entire sector before writing.
//! * Atmel flash chips have a sector size of 128 bytes. Reads to any alignment
//!   are efficient, however, unaligned writes are extremely slow.
//!   `prepare_write` does not immediately erase any data.
//! * EEPROM has a sector size of 8 bytes. Unaligned reads and writes are slower
//!   than aligned writes, however, this is easily mitigated by the small sector
//!   size.

use crate::save::utils::Timeout;
use crate::sync::{Lock, RawLockGuard};
use crate::timer::Timer;
use core::ops::Range;

mod asm_utils;
mod eeprom;
mod flash;
#[cfg(feature = "serde")]
mod serde;
mod sram;
mod utils;

#[cfg(feature = "serde")]
pub use serde::Save;

/// A list of save media types.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
#[non_exhaustive]
pub enum MediaType {
    /// 32KiB Battery-Backed SRAM or FRAM
    Sram32K,
    /// 8KiB EEPROM
    Eeprom8K,
    /// 512B EEPROM
    Eeprom512B,
    /// 64KiB flash chip
    Flash64K,
    /// 128KiB flash chip
    Flash128K,
}

/// The type used for errors encountered while reading or writing save media.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// There is no save media attached to this game cart.
    #[error("There is no save media attached to this game cart")]
    NoMedia,
    /// Failed to write the data to save media.
    #[error("Failed to write the data to save media")]
    WriteError,
    /// An operation on save media timed out.
    #[error("An operation on save media timed out")]
    OperationTimedOut,
    /// An attempt was made to access save media at an invalid offset.
    #[error("An attempt was made to access save media at an invalid offset")]
    OutOfBounds,
    /// The media is already in use.
    ///
    /// This can generally only happen in an IRQ that happens during an ongoing
    /// save media operation.
    #[error("This media is already in use.")]
    MediaInUse,
    /// This command cannot be used with the save media in use.
    #[error("This command cannot be used with the save media in use.")]
    IncompatibleCommand,
}

/// Information about the save media used.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct MediaInfo {
    /// The type of save media installed.
    pub media_type: MediaType,
    /// The power-of-two size of each sector. Zero represents a sector size of
    /// 0, implying sectors are not in use.
    ///
    /// (For example, 512 byte sectors would return 9 here.)
    pub sector_shift: usize,
    /// The size of the save media, in sectors.
    pub sector_count: usize,
    /// Whether the save media type requires media be prepared before writing.
    pub uses_prepare_write: bool,
}
impl MediaInfo {
    /// Returns the sector size of the save media. It is generally optimal to
    /// write data in blocks that are aligned to the sector size.
    #[must_use]
    pub fn sector_size(&self) -> usize {
        1 << self.sector_shift
    }

    /// Returns the total length of this save media.
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // is_empty() would always be false
    pub fn len(&self) -> usize {
        self.sector_count << self.sector_shift
    }
}

/// A trait allowing low-level saving and writing to save media.
trait RawSaveAccess: Sync {
    fn info(&self) -> Result<&'static MediaInfo, Error>;
    fn read(&self, offset: usize, buffer: &mut [u8], timeout: &mut Timeout) -> Result<(), Error>;
    fn verify(&self, offset: usize, buffer: &[u8], timeout: &mut Timeout) -> Result<bool, Error>;
    fn prepare_write(
        &self,
        sector: usize,
        count: usize,
        timeout: &mut Timeout,
    ) -> Result<(), Error>;
    fn write(&self, offset: usize, buffer: &[u8], timeout: &mut Timeout) -> Result<(), Error>;
}

static CURRENT_SAVE_ACCESS: Lock<Option<&'static dyn RawSaveAccess>> = Lock::new(None);

fn set_save_implementation(access_impl: &'static dyn RawSaveAccess) {
    let mut access = CURRENT_SAVE_ACCESS.lock();
    assert!(
        access.is_none(),
        "Cannot initialize the save media engine more than once."
    );
    *access = Some(access_impl);
}

fn get_save_implementation() -> Option<&'static dyn RawSaveAccess> {
    *CURRENT_SAVE_ACCESS.lock()
}

/// Allows reading and writing of save media.
pub struct SaveData {
    _lock: RawLockGuard<'static>,
    access: &'static dyn RawSaveAccess,
    info: &'static MediaInfo,
    timeout: utils::Timeout,
}
impl SaveData {
    /// Creates a new save accessor around the current save implementation.
    fn new(timer: Option<Timer>) -> Result<SaveData, Error> {
        match get_save_implementation() {
            Some(access) => Ok(SaveData {
                _lock: utils::lock_media_access()?,
                access,
                info: access.info()?,
                timeout: utils::Timeout::new(timer),
            }),
            None => Err(Error::NoMedia),
        }
    }

    /// Returns the media info underlying this accessor.
    #[must_use]
    pub fn media_info(&self) -> &'static MediaInfo {
        self.info
    }

    /// Returns the save media type being used.
    #[must_use]
    pub fn media_type(&self) -> MediaType {
        self.info.media_type
    }

    /// Returns the sector size of the save media. It is generally optimal to
    /// write data in blocks that are aligned to the sector size.
    #[must_use]
    pub fn sector_size(&self) -> usize {
        self.info.sector_size()
    }

    /// Returns the total length of this save media.
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // is_empty() would always be false
    pub fn len(&self) -> usize {
        self.info.len()
    }

    fn check_bounds(&self, range: Range<usize>) -> Result<(), Error> {
        if range.start >= self.len() || range.end > self.len() {
            Err(Error::OutOfBounds)
        } else {
            Ok(())
        }
    }
    fn check_bounds_len(&self, offset: usize, len: usize) -> Result<(), Error> {
        self.check_bounds(offset..(offset + len))
    }

    /// Copies data from the save media to a buffer.
    ///
    /// If an error is returned, the contents of the buffer are unpredictable.
    pub fn read(&mut self, offset: usize, buffer: &mut [u8]) -> Result<(), Error> {
        self.check_bounds_len(offset, buffer.len())?;
        self.access.read(offset, buffer, &mut self.timeout)
    }

    /// Verifies that a given block of memory matches the save media.
    pub fn verify(&mut self, offset: usize, buffer: &[u8]) -> Result<bool, Error> {
        self.check_bounds_len(offset, buffer.len())?;
        self.access.verify(offset, buffer, &mut self.timeout)
    }

    /// Returns a range that contains all sectors the input range overlaps.
    ///
    /// This can be used to calculate which blocks would be erased by a call
    /// to [`prepare_write`](`SaveData::prepare_write`)
    #[must_use]
    pub fn align_range(&self, range: Range<usize>) -> Range<usize> {
        let shift = self.info.sector_shift;
        let mask = (1 << shift) - 1;
        (range.start & !mask)..((range.end + mask) & !mask)
    }

    /// Prepares a given span of offsets for writing.
    ///
    /// This will erase any data in any sector overlapping the input range. To
    /// calculate which offset ranges would be affected, use the
    /// [`align_range`](`SaveData::align_range`) function.
    pub fn prepare_write(&mut self, range: Range<usize>) -> Result<SavePreparedBlock, Error> {
        self.check_bounds(range.clone())?;
        if self.info.uses_prepare_write {
            let range = self.align_range(range.clone());
            let shift = self.info.sector_shift;
            self.access.prepare_write(
                range.start >> shift,
                range.len() >> shift,
                &mut self.timeout,
            )?;
        }
        Ok(SavePreparedBlock {
            parent: self,
            range,
        })
    }
}

/// A block of save memory that has been prepared for writing.
pub struct SavePreparedBlock<'a> {
    parent: &'a mut SaveData,
    range: Range<usize>,
}
impl SavePreparedBlock<'_> {
    /// Writes a given buffer into the save media.
    ///
    /// Multiple overlapping writes to the same memory range without a separate
    /// call to `prepare_write` will leave the save data in an unpredictable
    /// state. If an error is returned, the contents of the save media is
    /// unpredictable.
    pub fn write(&mut self, offset: usize, buffer: &[u8]) -> Result<(), Error> {
        if buffer.is_empty() {
            Ok(())
        } else if !self.range.contains(&offset)
            || !self.range.contains(&(offset + buffer.len() - 1))
        {
            Err(Error::OutOfBounds)
        } else {
            self.parent
                .access
                .write(offset, buffer, &mut self.parent.timeout)
        }
    }

    /// Writes and validates a given buffer into the save media.
    ///
    /// This function will verify that the write has completed successfully, and
    /// return an error if it has not done so.
    ///
    /// Multiple overlapping writes to the same memory range without a separate
    /// call to `prepare_write` will leave the save data in an unpredictable
    /// state. If an error is returned, the contents of the save media is
    /// unpredictable.
    pub fn write_and_verify(&mut self, offset: usize, buffer: &[u8]) -> Result<(), Error> {
        self.write(offset, buffer)?;
        if !self.parent.verify(offset, buffer)? {
            Err(Error::WriteError)
        } else {
            Ok(())
        }
    }
}

mod marker {
    #[repr(align(4))]
    struct Align<T>(T);

    static EEPROM: Align<[u8; 12]> = Align(*b"EEPROM_Vnnn\0");
    static SRAM: Align<[u8; 12]> = Align(*b"SRAM_Vnnn\0\0\0");
    static FLASH512K: Align<[u8; 16]> = Align(*b"FLASH512_Vnnn\0\0\0");
    static FLASH1M: Align<[u8; 16]> = Align(*b"FLASH1M_Vnnn\0\0\0\0");

    #[inline(always)]
    pub fn emit_eeprom_marker() {
        core::hint::black_box(&EEPROM);
    }
    #[inline(always)]
    pub fn emit_sram_marker() {
        core::hint::black_box(&SRAM);
    }
    #[inline(always)]
    pub fn emit_flash_512k_marker() {
        core::hint::black_box(&FLASH512K);
    }
    #[inline(always)]
    pub fn emit_flash_1m_marker() {
        core::hint::black_box(&FLASH1M);
    }
}

#[derive(Clone, Copy)]
/// A type that indicates that you have initialised the save engine. It has no
/// impact on logic and is purely designed to help the user avoid bugs involved
/// in not initialising the save engine as a compile time check.
#[non_exhaustive]
pub struct InitialisedSaveEngine {}

/// Allows access to the cartridge's save data.
#[non_exhaustive]
pub struct SaveManager {}

impl SaveManager {
    pub(crate) const fn new() -> Self {
        SaveManager {}
    }

    /// Declares that the ROM uses battery backed SRAM/FRAM.
    ///
    /// Battery Backed SRAM is generally very fast, but limited in size compared
    /// to flash chips.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses, and configures the save manager to use the
    /// given save type.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_sram(&mut self) -> InitialisedSaveEngine {
        marker::emit_sram_marker();
        set_save_implementation(&sram::BatteryBackedAccess);
        InitialisedSaveEngine {}
    }

    /// Declares that the ROM uses 64KiB flash memory.
    ///
    /// Flash save media is generally very slow to write to and relatively fast
    /// to read from. It is the only real option if you need larger save data.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses, and configures the save manager to use the
    /// given save type.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_flash_64k(&mut self) -> InitialisedSaveEngine {
        marker::emit_flash_512k_marker();
        set_save_implementation(&flash::FlashAccess);
        InitialisedSaveEngine {}
    }

    /// Declares that the ROM uses 128KiB flash memory.
    ///
    /// Flash save media is generally very slow to write to and relatively fast
    /// to read from. It is the only real option if you need larger save data.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses, and configures the save manager to use the
    /// given save type.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_flash_128k(&mut self) -> InitialisedSaveEngine {
        marker::emit_flash_1m_marker();
        set_save_implementation(&flash::FlashAccess);
        InitialisedSaveEngine {}
    }

    /// Declares that the ROM uses 512 bytes EEPROM memory.
    ///
    /// EEPROM is generally pretty slow and also very small. It's mainly used in
    /// Game Paks because it's cheap.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses, and configures the save manager to use the
    /// given save type.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_eeprom_512b(&mut self) -> InitialisedSaveEngine {
        marker::emit_eeprom_marker();
        set_save_implementation(&eeprom::Eeprom512B);
        InitialisedSaveEngine {}
    }

    /// Declares that the ROM uses 8 KiB EEPROM memory.
    ///
    /// EEPROM is generally pretty slow and also very small. It's mainly used in
    /// Game Paks because it's cheap.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses, and configures the save manager to use the
    /// given save type.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_eeprom_8k(&mut self) -> InitialisedSaveEngine {
        marker::emit_eeprom_marker();
        set_save_implementation(&eeprom::Eeprom8K);
        InitialisedSaveEngine {}
    }

    /// Creates a new accessor to the save data.
    ///
    /// You must have initialized the save manager beforehand to use a specific
    /// type of media before calling this method.
    pub fn access(&mut self) -> Result<SaveData, Error> {
        SaveData::new(None)
    }

    /// Creates a new accessor to the save data that uses the given timer for timeouts.
    ///
    /// You must have initialized the save manager beforehand to use a specific
    /// type of media before calling this method.
    pub fn access_with_timer(&mut self, timer: Timer) -> Result<SaveData, Error> {
        SaveData::new(Some(timer))
    }
}
