use std::{borrow::Cow, f64::consts::PI};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Id(uuid::Uuid);

impl Id {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Clone)]
pub struct State {
    pub blocks: im::Vector<Block>,

    frequency: f64,
}

impl Default for State {
    fn default() -> Self {
        Self {
            blocks: Default::default(),
            frequency: 18157.0,
        }
    }
}

impl State {
    pub fn is_dirty(&self) -> bool {
        self.blocks.iter().any(|block| block.is_dirty())
    }

    pub fn clean(&mut self) {
        for block in self.blocks.iter_mut() {
            block.clean();
        }
    }

    pub fn frequency(&self) -> f64 {
        self.frequency
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

    pub fn set_input(&mut self, name: &str, value: &Input) {
        self.block_type.set_input(name, value);
        self.dirty = true;
    }

    pub fn calculate(&self, global_frequency: f64) -> Vec<f64> {
        self.block_type.calculate(global_frequency)
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
    fn set_input(&mut self, name: &str, value: &Input);
    fn calculate(&self, global_frequency: f64) -> Vec<f64>;
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
}

impl FundamentalShapeBlock {
    pub fn new(fundamental_shape_type: FundamentalShapeType) -> Self {
        Self {
            fundamental_shape_type,
            periods: 1.0,
            base_frequency: 256.0,
            base_amplitude: 0.5,
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
        ]
    }

    fn set_input(&mut self, name: &str, value: &Input) {
        match (name, value) {
            ("Frequency", Input::Frequency(new_frequency)) => {
                if *new_frequency != 0.0 {
                    self.base_frequency = *new_frequency;
                }
            }
            ("Amplitude", Input::Amplitude(new_amplitude)) => {
                self.base_amplitude = *new_amplitude;
            }
            ("Periods", Input::Periods(new_periods)) => {
                self.periods = *new_periods;
            }
            (name, value) => panic!("Invalid input {name} with value {value:?}"),
        }
    }

    fn calculate(&self, global_frequency: f64) -> Vec<f64> {
        let periods = if self.periods == 0.0 {
            1.0
        } else {
            self.periods
        };

        let period_length = (global_frequency / self.base_frequency).ceil();
        let length = (period_length * periods) as usize;

        let mut ret = Vec::with_capacity(length);
        for i in 0..length {
            ret.push(
                self.fundamental_shape_type
                    .value((i as f64 / period_length).fract())
                    * self.base_amplitude,
            );
        }

        ret
    }
}
