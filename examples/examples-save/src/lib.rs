#![no_std]

use agb::save::{SaveError, SaveSlotManager, Slot};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameWithSave {
    HyperspaceRoll = 0,
    DungeonPuzzler = 1,
}

pub const NUM_SAVE_SLOTS: usize = 2;
pub const SAVE_ID: &[u8; 32] = b"agb_examples____________________";

pub fn load<T: serde::de::DeserializeOwned + serde::Serialize + Clone>(
    save_manager: &mut SaveSlotManager,
    game_id: GameWithSave,
) -> Result<Option<T>, SaveError> {
    let slot = game_id as usize;
    match save_manager.slot(slot) {
        Slot::Empty => return Ok(None),
        Slot::Valid(_) => {}
        Slot::Corrupted => return Err(SaveError::SlotCorrupted),
    }

    Ok(Some(save_manager.read(slot)?))
}

pub fn save<T: serde::de::DeserializeOwned + serde::Serialize + Clone>(
    save_manager: &mut SaveSlotManager,
    game_id: GameWithSave,
    value: T,
) -> Result<(), SaveError> {
    save_manager.write(game_id as usize, &value, &())
}
