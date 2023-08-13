use std::{borrow::Cow, collections::HashMap};

mod band_pass_filter;
mod cross_fade;
mod fade;
mod fundamental_shape;
mod noise;

use serde::{Deserialize, Serialize};

use crate::state;

use self::{
    band_pass_filter::BandPassFilter,
    cross_fade::CrossFade,
    fade::Fade,
    fundamental_shape::{FundamentalShapeBlock, FundamentalShapeType},
    noise::Noise,
};

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum BlockCategory {
    Fundamental,
    Combine,
    Alter,
}

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BlockName {
    pub category: BlockCategory,
    pub name: String,
}

type MakeBlockType = Box<dyn Fn() -> Box<dyn BlockType>>;

pub struct BlockFactory {
    creation_functions: HashMap<BlockName, MakeBlockType>,
}

impl BlockFactory {
    pub fn new() -> Self {
        let mut creation_functions: HashMap<BlockName, MakeBlockType> = HashMap::new();

        for fundamental_shape in FundamentalShapeType::all() {
            creation_functions.insert(
                BlockName {
                    category: BlockCategory::Fundamental,
                    name: fundamental_shape.name().to_owned(),
                },
                Box::new(move || Box::new(FundamentalShapeBlock::new(fundamental_shape))),
            );
        }

        creation_functions.insert(Noise::name(), Box::new(|| Box::<Noise>::default()));
        creation_functions.insert(CrossFade::name(), Box::new(|| Box::<CrossFade>::default()));
        creation_functions.insert(Fade::name(), Box::new(|| Box::<Fade>::default()));
        creation_functions.insert(
            BandPassFilter::name(),
            Box::new(|| Box::<BandPassFilter>::default()),
        );

        Self { creation_functions }
    }

    pub fn available_blocks(&self) -> impl Iterator<Item = &BlockName> + '_ {
        let mut names = self.creation_functions.keys().collect::<Vec<_>>();
        names.sort();

        names.into_iter()
    }

    pub fn make_block(&self, name: &BlockName, pos: (f32, f32)) -> Block {
        self.make_block_with_id(name, pos, state::Id::new())
    }

    pub fn make_block_with_id(&self, name: &BlockName, pos: (f32, f32), id: state::Id) -> Block {
        let block_type = self
            .creation_functions
            .get(name)
            .unwrap_or_else(|| panic!("Failed to make block with name {name:?}"));

        Block::new_with_id(block_type(), pos, id)
    }
}

#[derive(Clone)]
pub struct Block {
    block_type: Box<dyn BlockType>,
    id: state::Id,
    x: f32,
    y: f32,
    dirty: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    Toggle(bool),
    Frequency(f64),
    Amplitude(f64),
    Periods(f64),
}

impl Block {
    pub fn new_with_id(block_type: Box<dyn BlockType>, pos: (f32, f32), id: state::Id) -> Self {
        Self {
            block_type,
            x: pos.0,
            y: pos.1,
            id,
            dirty: true,
        }
    }

    pub fn name(&self) -> BlockName {
        self.block_type.name()
    }

    pub fn id(&self) -> state::Id {
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

    pub fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    pub fn pos_delta(&mut self, delta: (f32, f32)) {
        self.x += delta.0;
        self.y += delta.1;
        // doesn't set dirty because it doesn't change the output
    }

    pub(super) fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub(super) fn clean(&mut self) {
        self.dirty = false;
    }
}

pub trait BlockClone {
    fn clone_box(&self) -> Box<dyn BlockType>;
}

pub trait BlockType: BlockClone + Send + Sync {
    fn name(&self) -> BlockName;
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

fn stretch_frequency_shift(input: f64) -> f64 {
    1.0 / (1.0 - input)
}
