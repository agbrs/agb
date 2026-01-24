use agb::Gba;
use agb::external::portable_atomic::{AtomicU32, Ordering};
use agb::save::{SaveError, SaveSlotManager};

use examples_save::*;

use serde::{Deserialize, Serialize};

static MAXIMUM_LEVEL: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Clone)]
pub struct MaxLevelSaveData(u32);

pub fn init_save(gba: &mut Gba) -> Result<SaveSlotManager, SaveError> {
    let mut save_manager = gba.save.init_sram(NUM_SAVE_SLOTS, *SAVE_ID)?;

    let level: Option<MaxLevelSaveData> = load(&mut save_manager, GameWithSave::DungeonPuzzler)?;
    let level = level.map(|save_data| save_data.0).unwrap_or(0);
    MAXIMUM_LEVEL.store(level, Ordering::SeqCst);

    Ok(save_manager)
}

pub fn load_max_level() -> u32 {
    MAXIMUM_LEVEL.load(Ordering::SeqCst)
}

pub fn save_max_level(save_manager: &mut SaveSlotManager, level: u32) -> Result<(), SaveError> {
    MAXIMUM_LEVEL.store(level, Ordering::SeqCst);
    save(
        save_manager,
        GameWithSave::DungeonPuzzler,
        MaxLevelSaveData(level),
    )?;
    Ok(())
}
