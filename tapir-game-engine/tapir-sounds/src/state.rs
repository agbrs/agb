use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Id(uuid::Uuid);

mod blocks;
pub mod persistance;

pub use blocks::{Block, BlockFactory, Input};

impl Id {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct State {
    blocks: im::HashMap<Id, Block>,

    // Maps inputs to outputs to make lookup faster
    connections: im::HashMap<(Id, usize), Id>,
    frequency: f64,

    selected_block: Option<Id>,

    dirty: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            connections: Default::default(),
            frequency: 18157.0,
            selected_block: None,
            dirty: false,
        }
    }
}

impl State {
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.blocks.iter().any(|(_, block)| block.is_dirty())
    }

    pub fn frequency(&self) -> f64 {
        self.frequency
    }

    pub fn add_connection(
        &mut self,
        (output_block, (input_block, input_block_index)): (Id, (Id, usize)),
    ) {
        if output_block == input_block {
            return;
        }

        // check if adding this connection would produce a cycle
        let mut graph = self.graph();
        graph.add_edge(output_block, input_block, ());

        if petgraph::algo::is_cyclic_directed(&graph) {
            return;
        }

        let input_key = (input_block, input_block_index);

        if self.connections.get(&input_key) == Some(&output_block) {
            self.connections.remove(&input_key);
        } else {
            self.connections.insert(input_key, output_block);
        }

        self.dirty = true;
    }

    pub fn blocks(&self) -> impl Iterator<Item = &Block> {
        self.blocks.values()
    }

    pub fn get_block_mut(&mut self, id: Id) -> Option<&mut Block> {
        self.blocks.get_mut(&id)
    }

    pub fn add_block(&mut self, block: Block) {
        self.blocks.insert(block.id(), block);
    }

    pub fn connections(&self) -> impl Iterator<Item = (Id, (Id, usize))> + '_ {
        self.connections
            .iter()
            .map(|(input, output)| (*output, *input))
    }

    pub fn clean(&mut self) {
        for (_, block) in self.blocks.iter_mut() {
            block.clean();
        }

        self.dirty = false;
    }

    pub fn calculate(&self) -> HashMap<Id, Vec<f64>> {
        let mut calculation: HashMap<Id, Vec<f64>> = HashMap::with_capacity(self.blocks.len());

        let sorted_blocks = petgraph::algo::toposort(&self.graph(), None)
            .expect("There shouldn't be a cycle because we check on addition");

        let sorted_blocks = sorted_blocks.iter().map(|id| self.blocks.get(id).unwrap());

        for block in sorted_blocks {
            let n_inputs = block.inputs().len();
            let input_data = (0..n_inputs)
                .map(|i| {
                    self.connections
                        .get(&(block.id(), i))
                        .and_then(|connection| calculation.get(connection))
                        .map(|data| data.as_slice())
                })
                .collect::<Vec<_>>();

            calculation.insert(block.id(), block.calculate(self.frequency, &input_data));
        }

        calculation
    }

    pub fn set_selected_block(&mut self, id: Id) {
        self.selected_block = Some(id);
    }

    pub fn selected_block(&self) -> Option<Id> {
        self.selected_block
    }

    fn graph(&self) -> petgraph::graphmap::GraphMap<Id, (), petgraph::Directed> {
        let mut graph =
            petgraph::graphmap::GraphMap::with_capacity(self.blocks.len(), self.connections.len());

        for id in self.blocks.keys() {
            graph.add_node(*id);
        }

        for ((input, _), output) in &self.connections {
            graph.add_edge(*output, *input, ());
        }

        graph
    }

    pub fn average_location(&self) -> (f32, f32) {
        let total_pos = self.blocks.values().fold((0.0, 0.0), |acc, curr| {
            (acc.0 + curr.pos().0, acc.1 + curr.pos().1)
        });

        let count = self.blocks.len() as f32;

        (total_pos.0 / count, total_pos.1 / count)
    }
}
