use std::{borrow::Cow, collections::HashMap, f64::consts::PI};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Id(uuid::Uuid);

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

    dirty: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            connections: Default::default(),
            frequency: 18157.0,
            dirty: false,
        }
    }
}

impl State {
    pub fn is_dirty(&self) -> bool {
        self.dirty || self.blocks.iter().any(|(_, block)| block.is_dirty())
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
        self.blocks.insert(block.id, block);
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
                        .get(&(block.id, i))
                        .and_then(|connection| calculation.get(connection))
                        .map(|data| data.as_slice())
                })
                .collect::<Vec<_>>();

            calculation.insert(block.id, block.calculate(self.frequency, &input_data));
        }

        calculation
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
}

#[derive(Clone)]
pub struct Block {
    block_type: Box<dyn BlockType>,
    id: Id,
    dirty: bool,
}

#[derive(Clone, Debug)]
pub enum Input {
    Toggle(bool),
    Frequency(f64),
    Amplitude(f64),
    Periods(f64),
}

impl Block {
    pub fn new(block_type: Box<dyn BlockType>) -> Self {
        Self {
            block_type,
            id: Id::new(),
            dirty: true,
        }
    }

    pub fn name(&self) -> Cow<'static, str> {
        self.block_type.name()
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn inputs(&self) -> Vec<(Cow<'static, str>, Input)> {
        self.block_type.inputs()
    }

    pub fn set_input(&mut self, index: usize, value: &Input) {
        self.block_type.set_input(index, value);
        self.dirty = true;
    }

    pub fn calculate(&self, global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        self.block_type.calculate(global_frequency, inputs)
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn clean(&mut self) {
        self.dirty = false;
    }
}

pub trait BlockClone {
    fn clone_box(&self) -> Box<dyn BlockType>;
}

pub trait BlockType: BlockClone + Send + Sync {
    fn name(&self) -> Cow<'static, str>;
    fn inputs(&self) -> Vec<(Cow<'static, str>, Input)>;
    fn set_input(&mut self, index: usize, value: &Input);
    fn calculate(&self, global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64>;
}

impl Clone for Box<dyn BlockType> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl<T> BlockClone for T
where
    T: 'static + BlockType + Clone,
{
    fn clone_box(&self) -> Box<dyn BlockType> {
        Box::new(self.clone())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum FundamentalShapeType {
    Sine,
    Square,
    Triangle,
    Saw,
}

impl FundamentalShapeType {
    pub fn all() -> impl Iterator<Item = FundamentalShapeType> + 'static {
        [Self::Sine, Self::Square, Self::Triangle, Self::Saw].into_iter()
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Square => "Square",
            Self::Triangle => "Triangle",
            Self::Saw => "Saw",
        }
    }

    fn value(self, index: f64) -> f64 {
        match self {
            Self::Sine => (index * PI * 2.0).sin(),
            Self::Square => {
                if index < 0.5 {
                    -1.0
                } else {
                    1.0
                }
            }
            Self::Triangle => {
                if index < 0.25 {
                    index * 4.0
                } else if index < 0.75 {
                    (index - 0.5) * -4.0
                } else {
                    (index - 0.75) * 4.0 - 1.0
                }
            }
            Self::Saw => {
                if index < 0.5 {
                    index * 2.0
                } else {
                    index * 2.0 - 2.0
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct FundamentalShapeBlock {
    fundamental_shape_type: FundamentalShapeType,
    periods: f64,
    base_frequency: f64,
    base_amplitude: f64,
    offset: f64,
}

impl FundamentalShapeBlock {
    pub fn new(fundamental_shape_type: FundamentalShapeType) -> Self {
        Self {
            fundamental_shape_type,
            periods: 1.0,
            base_frequency: 256.0,
            base_amplitude: 0.5,
            offset: 0.0,
        }
    }
}

impl BlockType for FundamentalShapeBlock {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.fundamental_shape_type.name())
    }

    fn inputs(&self) -> Vec<(Cow<'static, str>, Input)> {
        vec![
            ("Frequency".into(), Input::Frequency(self.base_frequency)),
            ("Amplitude".into(), Input::Amplitude(self.base_amplitude)),
            ("Periods".into(), Input::Periods(self.periods)),
            ("Offset".into(), Input::Periods(self.offset)),
        ]
    }

    fn set_input(&mut self, index: usize, value: &Input) {
        match (index, value) {
            (0, Input::Frequency(new_frequency)) => {
                if *new_frequency != 0.0 {
                    self.base_frequency = *new_frequency;
                }
            }
            (1, Input::Amplitude(new_amplitude)) => {
                self.base_amplitude = *new_amplitude;
            }
            (2, Input::Periods(new_periods)) => {
                self.periods = *new_periods;
            }
            (3, Input::Periods(new_offset)) => {
                self.offset = new_offset.clamp(0.0, 1.0);
            }
            (name, value) => panic!("Invalid input {name} with value {value:?}"),
        }
    }

    fn calculate(&self, global_frequency: f64, inputs: &[Option<&[f64]>]) -> Vec<f64> {
        let periods = if self.periods == 0.0 {
            1.0
        } else {
            self.periods
        };

        let period_length = (global_frequency / self.base_frequency).ceil();
        let length = (period_length * periods) as usize;

        let mut ret = Vec::with_capacity(length);
        for i in 0..length {
            let frequency_at_i = self.base_frequency
                * stretch_frequency_shift(
                    inputs[0]
                        .map(|frequency_input| frequency_input[i % frequency_input.len()])
                        .unwrap_or(0.0),
                )
                .clamp(0.1, 10_000.0);

            let amplitude_at_i = (self.base_amplitude
                * inputs[1]
                    .map(|amplitude_input| amplitude_input[i % amplitude_input.len()])
                    .unwrap_or(1.0))
            .clamp(-1.0, 1.0);

            let period_length_at_i = global_frequency / frequency_at_i;

            ret.push(
                self.fundamental_shape_type
                    .value((i as f64 / period_length_at_i).fract() + self.offset)
                    * amplitude_at_i,
            );
        }

        ret
    }
}

fn stretch_frequency_shift(input: f64) -> f64 {
    1.0 / (1.0 - input)
}
