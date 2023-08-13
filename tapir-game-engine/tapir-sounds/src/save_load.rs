use std::path::Path;

use crate::state;

pub fn save(state: &state::State, filepath: &Path) {}

pub fn load(filepath: &Path, block_factory: &state::BlockFactory) -> state::State {
    todo!()
}
