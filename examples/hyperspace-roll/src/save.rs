use agb::Gba;
use agb::external::portable_atomic::{AtomicU32, Ordering};
use agb::save::{SaveError, SaveSlotManager};

use serde::{Deserialize, Serialize};

static HIGH_SCORE: AtomicU32 = AtomicU32::new(0);

#[derive(Serialize, Deserialize, Clone)]
pub struct HighScoreSaveMetadata(u32);

pub fn init_save(gba: &mut Gba) -> Result<SaveSlotManager<HighScoreSaveMetadata>, SaveError> {
    let save_mager = gba.save.init_sram::<HighScoreSaveMetadata>(1, [0; _])?;

    let score = save_mager.metadata(0).map(|hs| hs.0).unwrap_or_default();
    HIGH_SCORE.store(score, Ordering::SeqCst);

    Ok(save_mager)
}

pub fn load_high_score() -> u32 {
    HIGH_SCORE.load(Ordering::SeqCst)
}

pub fn save_high_score(
    save: &mut SaveSlotManager<HighScoreSaveMetadata>,
    score: u32,
) -> Result<(), SaveError> {
    HIGH_SCORE.store(score, Ordering::SeqCst);
    save.write(0, &(), &HighScoreSaveMetadata(score))?;
    Ok(())
}
