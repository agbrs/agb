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
//! Each `init_*` method returns a [`SaveSlotManager`] which provides a
//! high-level interface for managing multiple save slots with corruption
//! detection and recovery.
//!
//! ## Performance and Other Details
//!
//! The performance characteristics of the media types are as follows:
//!
//! * SRAM is simply a form of battery backed memory, and has no particular
//!   performance characteristics. Reads and writes at any alignment are
//!   efficient. Furthermore, no timer is needed for accesses to this type of
//!   media.
//! * Non-Atmel flash chips have a sector size of 4096 bytes. Reads and writes
//!   to any alignment are efficient, however, writes will erase all data in an
//!   entire sector before writing.
//! * Atmel flash chips have a sector size of 128 bytes. Reads to any alignment
//!   are efficient, however, unaligned writes are extremely slow.
//! * EEPROM has a sector size of 8 bytes. Unaligned reads and writes are slower
//!   than aligned writes, however, this is easily mitigated by the small sector
//!   size.

use crate::save::utils::Timeout;
use crate::sync::{Lock, RawLockGuard};
use crate::timer::Timer;
use core::num::NonZeroUsize;
use core::ops::Range;

mod asm_utils;
mod eeprom;
mod flash;
mod sram;
mod utils;

#[doc(inline)]
pub use agb_save::{SerializationError, Slot};

/// An error that can happen while saving or loading.
pub type SaveError = agb_save::SaveError<StorageError>;

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
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum StorageError {
    /// There is no save media attached to this game cart.
    NoMedia,
    /// Failed to write the data to save media.
    WriteError,
    /// An operation on save media timed out.
    OperationTimedOut,
    /// An attempt was made to access save media at an invalid offset.
    OutOfBounds,
    /// The media is already in use.
    ///
    /// This can generally only happen in an IRQ that happens during an ongoing
    /// save media operation.
    MediaInUse,
    /// This command cannot be used with the save media in use.
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
    fn info(&self) -> Result<&'static MediaInfo, StorageError>;
    fn read(
        &self,
        offset: usize,
        buffer: &mut [u8],
        timeout: &mut Timeout,
    ) -> Result<(), StorageError>;
    fn verify(
        &self,
        offset: usize,
        buffer: &[u8],
        timeout: &mut Timeout,
    ) -> Result<bool, StorageError>;
    fn prepare_write(
        &self,
        sector: usize,
        count: usize,
        timeout: &mut Timeout,
    ) -> Result<(), StorageError>;
    fn write(
        &self,
        offset: usize,
        buffer: &[u8],
        timeout: &mut Timeout,
    ) -> Result<(), StorageError>;
}

static CURRENT_SAVE_ACCESS: Lock<Option<&'static dyn RawSaveAccess>> = Lock::new(None);

/// Configuration stored when save is initialized, used for reopen
struct SaveConfig {
    num_slots: usize,
    magic: [u8; 32],
}

static SAVE_CONFIG: Lock<Option<SaveConfig>> = Lock::new(None);

fn set_save_implementation(
    access_impl: &'static dyn RawSaveAccess,
    num_slots: usize,
    magic: [u8; 32],
) {
    let mut access = CURRENT_SAVE_ACCESS.lock();
    assert!(
        access.is_none(),
        "Cannot initialize the save media engine more than once."
    );
    *access = Some(access_impl);

    let mut config = SAVE_CONFIG.lock();
    *config = Some(SaveConfig { num_slots, magic });
}

fn get_save_implementation() -> Option<&'static dyn RawSaveAccess> {
    *CURRENT_SAVE_ACCESS.lock()
}

/// Low-level save media accessor.
pub(crate) struct SaveData {
    _lock: RawLockGuard<'static>,
    access: &'static dyn RawSaveAccess,
    info: &'static MediaInfo,
    timeout: utils::Timeout,
}

impl SaveData {
    /// Creates a new save accessor around the current save implementation.
    fn new(timer: Option<Timer>) -> Result<SaveData, StorageError> {
        match get_save_implementation() {
            Some(access) => Ok(SaveData {
                _lock: utils::lock_media_access()?,
                access,
                info: access.info()?,
                timeout: utils::Timeout::new(timer),
            }),
            None => Err(StorageError::NoMedia),
        }
    }

    fn check_bounds(&self, range: Range<usize>) -> Result<(), StorageError> {
        let len = self.info.len();
        if range.start >= len || range.end > len {
            Err(StorageError::OutOfBounds)
        } else {
            Ok(())
        }
    }

    fn check_bounds_len(&self, offset: usize, len: usize) -> Result<(), StorageError> {
        self.check_bounds(offset..(offset + len))
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

/// Allows access to the cartridge's save data.
#[non_exhaustive]
pub struct SaveManager {}

impl SaveManager {
    pub(crate) const fn new() -> Self {
        SaveManager {}
    }

    /// Declares that the ROM uses battery backed SRAM/FRAM and initializes the
    /// save slot manager.
    ///
    /// Battery Backed SRAM is generally very fast, but limited in size compared
    /// to flash chips.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses.
    ///
    /// # Arguments
    ///
    /// * `num_slots` - Number of save slots (typically 1-4)
    /// * `magic` - A 32-byte game identifier. If this doesn't match what's stored,
    ///   the save data is considered incompatible and will be reformatted.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_sram<Metadata>(
        &mut self,
        num_slots: usize,
        magic: [u8; 32],
    ) -> Result<SaveSlotManager<Metadata>, SaveError>
    where
        Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        marker::emit_sram_marker();
        set_save_implementation(&sram::BatteryBackedAccess, num_slots, magic);
        let save_data = SaveData::new(None).map_err(SaveError::Storage)?;
        Ok(SaveSlotManager {
            inner: agb_save::SaveSlotManager::new(save_data, num_slots, magic)?,
        })
    }

    /// Declares that the ROM uses 64KiB flash memory and initializes the save
    /// slot manager.
    ///
    /// Flash save media is generally very slow to write to and relatively fast
    /// to read from. It is the only real option if you need larger save data.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses.
    ///
    /// # Arguments
    ///
    /// * `num_slots` - Number of save slots (typically 1-4)
    /// * `magic` - A 32-byte game identifier. If this doesn't match what's stored,
    ///   the save data is considered incompatible and will be reformatted.
    /// * `timer` - Optional timer for timeout handling during flash operations.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_flash_64k<Metadata>(
        &mut self,
        num_slots: usize,
        magic: [u8; 32],
        timer: Option<Timer>,
    ) -> Result<SaveSlotManager<Metadata>, SaveError>
    where
        Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        marker::emit_flash_512k_marker();
        set_save_implementation(&flash::FlashAccess, num_slots, magic);
        let save_data = SaveData::new(timer).map_err(SaveError::Storage)?;
        Ok(SaveSlotManager {
            inner: agb_save::SaveSlotManager::new(save_data, num_slots, magic)?,
        })
    }

    /// Declares that the ROM uses 128KiB flash memory and initializes the save
    /// slot manager.
    ///
    /// Flash save media is generally very slow to write to and relatively fast
    /// to read from. It is the only real option if you need larger save data.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses.
    ///
    /// # Arguments
    ///
    /// * `num_slots` - Number of save slots (typically 1-4)
    /// * `magic` - A 32-byte game identifier. If this doesn't match what's stored,
    ///   the save data is considered incompatible and will be reformatted.
    /// * `timer` - Optional timer for timeout handling during flash operations.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_flash_128k<Metadata>(
        &mut self,
        num_slots: usize,
        magic: [u8; 32],
        timer: Option<Timer>,
    ) -> Result<SaveSlotManager<Metadata>, SaveError>
    where
        Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        marker::emit_flash_1m_marker();
        set_save_implementation(&flash::FlashAccess, num_slots, magic);
        let save_data = SaveData::new(timer).map_err(SaveError::Storage)?;
        Ok(SaveSlotManager {
            inner: agb_save::SaveSlotManager::new(save_data, num_slots, magic)?,
        })
    }

    /// Declares that the ROM uses 512 bytes EEPROM memory and initializes the
    /// save slot manager.
    ///
    /// EEPROM is generally pretty slow and also very small. It's mainly used in
    /// Game Paks because it's cheap.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses.
    ///
    /// # Arguments
    ///
    /// * `num_slots` - Number of save slots (typically 1-4)
    /// * `magic` - A 32-byte game identifier. If this doesn't match what's stored,
    ///   the save data is considered incompatible and will be reformatted.
    /// * `timer` - Optional timer for timeout handling during EEPROM operations.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_eeprom_512b<Metadata>(
        &mut self,
        num_slots: usize,
        magic: [u8; 32],
        timer: Option<Timer>,
    ) -> Result<SaveSlotManager<Metadata>, SaveError>
    where
        Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        marker::emit_eeprom_marker();
        set_save_implementation(&eeprom::Eeprom512B, num_slots, magic);
        let save_data = SaveData::new(timer).map_err(SaveError::Storage)?;
        Ok(SaveSlotManager {
            inner: agb_save::SaveSlotManager::new(save_data, num_slots, magic)?,
        })
    }

    /// Declares that the ROM uses 8 KiB EEPROM memory and initializes the save
    /// slot manager.
    ///
    /// EEPROM is generally pretty slow and also very small. It's mainly used in
    /// Game Paks because it's cheap.
    ///
    /// This creates a marker in the ROM that allows emulators to understand what
    /// save type the Game Pak uses.
    ///
    /// # Arguments
    ///
    /// * `num_slots` - Number of save slots (typically 1-4)
    /// * `magic` - A 32-byte game identifier. If this doesn't match what's stored,
    ///   the save data is considered incompatible and will be reformatted.
    /// * `timer` - Optional timer for timeout handling during EEPROM operations.
    ///
    /// Only one `init_*` function may be called in the lifetime of the program.
    pub fn init_eeprom_8k<Metadata>(
        &mut self,
        num_slots: usize,
        magic: [u8; 32],
        timer: Option<Timer>,
    ) -> Result<SaveSlotManager<Metadata>, SaveError>
    where
        Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        marker::emit_eeprom_marker();
        set_save_implementation(&eeprom::Eeprom8K, num_slots, magic);
        let save_data = SaveData::new(timer).map_err(SaveError::Storage)?;
        Ok(SaveSlotManager {
            inner: agb_save::SaveSlotManager::new(save_data, num_slots, magic)?,
        })
    }

    /// Reopens an already-initialized save media with a new [`SaveSlotManager`].
    ///
    /// This is useful for verifying save persistence or recovering from errors.
    /// The save media must have been previously initialized with one of the
    /// `init_*` methods. The configuration (num_slots, magic) is automatically
    /// reused from the original initialization.
    ///
    /// # Arguments
    ///
    /// * `timer` - Optional timer for timeout handling (for EEPROM/Flash operations)
    ///
    /// # Errors
    ///
    /// Returns [`SaveError::Storage`] with [`StorageError::NoMedia`] if the save media
    /// has not been initialized.
    pub fn reopen<Metadata>(
        &mut self,
        timer: Option<Timer>,
    ) -> Result<SaveSlotManager<Metadata>, SaveError>
    where
        Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
    {
        let config = SAVE_CONFIG.lock();
        let config = config
            .as_ref()
            .ok_or(SaveError::Storage(StorageError::NoMedia))?;

        let save_data = SaveData::new(timer).map_err(SaveError::Storage)?;
        Ok(SaveSlotManager {
            inner: agb_save::SaveSlotManager::new(save_data, config.num_slots, config.magic)?,
        })
    }
}

/// Manages multiple save slots with corruption detection and recovery.
///
/// This manager handles:
/// - Multiple save slots (like classic RPGs)
/// - Corruption detection and recovery via ghost slots
/// - Metadata for each slot (e.g., player name, playtime) that can be read without
///   loading the full save data
///
/// # Type Parameters
///
/// - `Metadata`: A serde-serializable type for slot metadata shown in save menus.
///   Defaults to `()` if you don't need metadata.
pub struct SaveSlotManager<Metadata = ()> {
    inner: agb_save::SaveSlotManager<SaveData, Metadata>,
}

impl<Metadata> SaveSlotManager<Metadata>
where
    Metadata: serde::Serialize + serde::de::DeserializeOwned + Clone,
{
    /// Returns the number of save slots.
    #[must_use]
    pub fn num_slots(&self) -> usize {
        self.inner.num_slots()
    }

    /// Returns the state of the given slot.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    #[must_use]
    pub fn slot(&self, slot: usize) -> Slot<'_, Metadata> {
        self.inner.slot(slot)
    }

    /// Returns the metadata for the given slot, if it exists and is valid.
    ///
    /// This is useful for displaying save slot information (player name, playtime, etc.)
    /// without loading the full save data.
    ///
    /// # Panics
    ///
    /// Panics if `slot >= num_slots()`.
    #[must_use]
    pub fn metadata(&self, slot: usize) -> Option<&Metadata> {
        self.inner.metadata(slot)
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
    pub fn read<T>(&mut self, slot: usize) -> Result<T, SaveError>
    where
        T: serde::de::DeserializeOwned,
    {
        self.inner.read(slot)
    }

    /// Write save data and metadata to a slot.
    ///
    /// This operation is designed to be crash-safe:
    /// 1. Data is written to new blocks
    /// 2. A new slot header is written
    /// 3. The old slot header is marked as ghost (backup)
    ///
    /// If a crash occurs during writing, the next initialization will recover
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
    pub fn write<T>(&mut self, slot: usize, data: &T, metadata: &Metadata) -> Result<(), SaveError>
    where
        T: serde::Serialize,
    {
        self.inner.write(slot, data, metadata)
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
    pub fn erase(&mut self, slot: usize) -> Result<(), SaveError> {
        self.inner.erase(slot)
    }

    /// Returns an iterator over all slots with their current state.
    ///
    /// Useful for displaying a save slot selection screen.
    pub fn slots(&self) -> impl Iterator<Item = Slot<'_, Metadata>> {
        self.inner.slots()
    }
}

impl agb_save::StorageMedium for SaveData {
    type Error = StorageError;

    fn info(&self) -> agb_save::StorageInfo {
        let sector_size = self.info.sector_size();
        agb_save::StorageInfo {
            size: self.info.len(),
            erase_size: if self.info.uses_prepare_write {
                NonZeroUsize::new(sector_size)
            } else {
                None
            },
            write_size: NonZeroUsize::new(if self.info.uses_prepare_write {
                1
            } else {
                sector_size
            })
            .unwrap(),
        }
    }

    fn read(&mut self, offset: usize, buf: &mut [u8]) -> Result<(), Self::Error> {
        self.check_bounds_len(offset, buf.len())?;
        self.access.read(offset, buf, &mut self.timeout)
    }

    fn erase(&mut self, offset: usize, len: usize) -> Result<(), Self::Error> {
        if self.info.uses_prepare_write {
            let shift = self.info.sector_shift;
            self.access
                .prepare_write(offset >> shift, len >> shift, &mut self.timeout)
        } else {
            Ok(())
        }
    }

    fn write(&mut self, offset: usize, data: &[u8]) -> Result<(), Self::Error> {
        self.check_bounds_len(offset, data.len())?;
        self.access.write(offset, data, &mut self.timeout)
    }

    fn verify(&mut self, offset: usize, expected: &[u8]) -> Result<bool, Self::Error> {
        self.check_bounds_len(offset, expected.len())?;
        self.access.verify(offset, expected, &mut self.timeout)
    }
}
