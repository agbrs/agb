use agb::Gba;
use agb::external::portable_atomic::{AtomicU32, Ordering};
use agb::save::{SaveError, SaveSlotManager};

use examples_save::*;

use serde::{Deserialize, Serialize};

static HIGH_SCORE: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreSaveData(u32);

pub fn init_save(gba: &mut Gba) -> Result<SaveSlotManager, SaveError> {
    let mut save_manager = gba.save.init_sram(NUM_SAVE_SLOTS, *SAVE_ID)?;

    let score: Option<HighScoreSaveData> = load(&mut save_manager, GameWithSave::HyperspaceRoll)?;
    let score = score.map(|save_data| save_data.0).unwrap_or(0);
    HIGH_SCORE.store(score, Ordering::SeqCst);

    Ok(save_manager)
}

pub fn load_high_score() -> u32 {
    HIGH_SCORE.load(Ordering::SeqCst)
}

pub fn save_high_score(save_manager: &mut SaveSlotManager, score: u32) -> Result<(), SaveError> {
    HIGH_SCORE.store(score, Ordering::SeqCst);
    save(
        save_manager,
        GameWithSave::HyperspaceRoll,
        HighScoreSaveData(score),
    )?;
    Ok(())
}
