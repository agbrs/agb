use serde::{Deserialize, Serialize};

use super::blocks::BlockName;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PersistedBlock {
    id: uuid::Uuid,
    name: BlockName,
    inputs: Vec<super::Input>,
    x: f32,
    y: f32,
}

impl PersistedBlock {
    fn new_from_block(block: &super::Block) -> Self {
        let block_pos = block.pos();

        Self {
            id: block.id().0,
            name: block.name(),
            inputs: block.inputs().into_iter().map(|(_, input)| input).collect(),
            x: block_pos.0,
            y: block_pos.1,
        }
    }

    fn into_block(self, block_factory: &super::BlockFactory) -> super::Block {
        block_factory.make_block_with_id(&self.name, (self.x, self.y), super::Id(self.id))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PersistedState {
    blocks: Vec<PersistedBlock>,
    connections: Vec<(uuid::Uuid, uuid::Uuid, usize)>,
    frequency: f64,
    selected_block: Option<uuid::Uuid>,
}

impl PersistedState {
    pub fn new_from_state(state: &super::State) -> Self {
        Self {
            blocks: state.blocks().map(PersistedBlock::new_from_block).collect(),
            connections: state
                .connections()
                .map(|(output, (input, index))| (output.0, input.0, index))
                .collect(),
            frequency: state.frequency,
            selected_block: state.selected_block().map(|id| id.0),
        }
    }

    pub fn to_state(self, block_factory: &super::BlockFactory) -> super::State {
        let mut result = super::State::default();

        for block in self.blocks {
            result.add_block(block.into_block(block_factory));
        }

        for (output_id, input_id, index) in self.connections {
            result.add_connection((super::Id(output_id), (super::Id(input_id), index)));
        }

        if let Some(selected_block) = self.selected_block {
            result.set_selected_block(super::Id(selected_block));
        }

        result
    }
}
