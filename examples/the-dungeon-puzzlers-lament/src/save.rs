use agb::external::portable_atomic::{AtomicU32, Ordering};
use agb::{
    Gba,
    save::{SaveError, SaveSlotManager, Slot},
};
use serde::{Deserialize, Serialize};

static MAXIMUM_LEVEL: AtomicU32 = AtomicU32::new(0);

const SAVE_MAGIC: [u8; 32] = *b"dungeon-puzzlers-lament-v1______";

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SaveMetadata {
    max_level: u32,
}

pub fn init_save(gba: &mut Gba) -> Result<SaveSlotManager<SaveMetadata>, SaveError> {
    let manager = gba.save.init_sram::<SaveMetadata>(1, SAVE_MAGIC)?;

    match manager.slot(0) {
        Slot::Valid(metadata) => {
            MAXIMUM_LEVEL.store(metadata.max_level, Ordering::SeqCst);
        }
        Slot::Empty | Slot::Corrupted => {
            MAXIMUM_LEVEL.store(0, Ordering::SeqCst);
        }
    }

    Ok(manager)
}

pub fn load_max_level() -> u32 {
    MAXIMUM_LEVEL.load(Ordering::SeqCst)
}

pub fn save_max_level(
    manager: &mut SaveSlotManager<SaveMetadata>,
    level: u32,
) -> Result<(), SaveError> {
    let metadata = SaveMetadata { max_level: level };
    manager.write(0, &(), &metadata)?;
    MAXIMUM_LEVEL.store(level, Ordering::SeqCst);
    Ok(())
}
