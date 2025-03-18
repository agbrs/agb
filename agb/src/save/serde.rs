use core::marker::PhantomData;

use crate::{
    Gba,
    save::{Error, SaveData},
};
use alloc::vec;
use serde::{Serialize, de::DeserializeOwned};

use super::InitialisedSaveEngine;

pub struct Save<T> {
    access: SaveData,
    unique_id: &'static [u8],
    phantom: PhantomData<T>,
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum LoadError {
    #[error("The Id in the save file indicates this save is incompatible")]
    IdMismatch,
    #[error("The save is uninitialised")]
    UninitialisedSave,
    #[error("Problem with serialization: {0}")]
    SerializationError(#[from] postcard::Error),
    #[error("Problem interacting with save media: {0}")]
    SaveError(#[from] Error),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Clone)]
pub enum SaveError {
    #[error("Problem with serialization: {0}")]
    SerializationError(#[from] postcard::Error),
    #[error("Problem interacting with save media: {0}")]
    SaveError(#[from] Error),
}

impl<T> Save<T>
where
    T: Serialize + DeserializeOwned,
{
    /// Creates an easier to use save system. The unique_id is an id identifying
    /// your game, the id is stored as part of saving the game and when loading
    /// a game it will reject the save if the id does not match. This is useful
    /// to make sure you don't try load other save data from other games as your
    /// own. Previously we've made it be the commit hash, this meant that we
    /// didn't need to worry about compatibility between different versions of
    /// the game during development.
    pub fn new(
        gba: &mut Gba,
        _save_engine: InitialisedSaveEngine,
        unique_id: &'static [u8],
    ) -> Result<Self, Error> {
        Ok(Self {
            access: Self::create_access(gba)?,
            unique_id,
            phantom: PhantomData,
        })
    }

    fn create_access(gba: &mut Gba) -> Result<SaveData, Error> {
        let timers = gba.timers.timers();
        let access = gba.save.access_with_timer(timers.timer2)?;

        Ok(access)
    }

    /// Loads the save data and deserialises it. Depending on how the save data
    /// is corrupted this could cause a panic due to allocation failures as the
    /// corrupted bytes may refer to the length of a dynamicly sized collection.
    pub fn load(&mut self) -> Result<T, LoadError> {
        let mut buffer_size_buffer = [0u8; 4];
        self.access.read(0, &mut buffer_size_buffer)?;
        let buffer_size = usize::from_le_bytes(buffer_size_buffer);

        if buffer_size == 0xffff_ffff {
            return Err(LoadError::UninitialisedSave);
        }

        let mut unique_id_length_buffer = [0u8; 4];
        self.access.read(4, &mut unique_id_length_buffer)?;
        let id_length = usize::from_le_bytes(unique_id_length_buffer);

        if id_length != self.unique_id.len() {
            return Err(LoadError::IdMismatch);
        }

        let mut save_id_buffer = vec![0u8; id_length];

        self.access.read(8, &mut save_id_buffer)?;
        if save_id_buffer != self.unique_id {
            return Err(LoadError::IdMismatch);
        }

        let save_length = buffer_size - id_length - 4;

        let mut buffer = vec![0u8; save_length];
        self.access.read(8 + id_length, &mut buffer)?;

        Ok(postcard::from_bytes(&buffer)?)
    }

    /// Serialises and saves the save game.
    pub fn save(&mut self, state: &T) -> Result<(), SaveError> {
        let mut buffer = vec![0, 0, 0, 0];

        buffer.extend(usize::to_le_bytes(self.unique_id.len()));
        buffer.extend(self.unique_id);

        let mut buffer = postcard::to_extend(state, buffer)?;

        for (i, &b) in buffer.len().to_le_bytes().iter().enumerate() {
            buffer[i] = b;
        }

        // extend the buffer to the nearest multiple of 4
        buffer.resize((buffer.len() + 3) & !3, 0);

        let mut block = self.access.prepare_write(0..buffer.len())?;
        block.write(0, &buffer)?;

        Ok(())
    }
}
