use std::{fs, path::Path};

use crate::state;

pub fn save(state: &state::State, filepath: &Path) {
    let persisted_state = state::persistance::PersistedState::new_from_state(state);

    let output = ron::ser::to_string_pretty(&persisted_state, ron::ser::PrettyConfig::default())
        .expect("Failed to serialize");
    fs::write(filepath, output).expect("failed to write to file");
}

pub fn load(filepath: &Path, block_factory: &state::BlockFactory) -> state::State {
    let content = fs::read_to_string(filepath).expect("Failed to load");
    let deserialized: state::persistance::PersistedState =
        ron::from_str(&content).expect("Failed to deserialize");

    deserialized.to_state(block_factory)
}
