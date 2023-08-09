use std::borrow::Cow;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Id(uuid::Uuid);

impl Id {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Default)]
pub struct State {
    pub blocks: im::Vector<Block>,
}

#[derive(Clone)]
pub struct Block {
    pub block_type: Box<dyn BlockType>,
    pub id: Id,
}

impl Block {
    pub fn new(block_type: Box<dyn BlockType>) -> Self {
        Self {
            block_type,
            id: Id::new(),
        }
    }

    pub fn name(&self) -> Cow<'static, str> {
        self.block_type.name()
    }
}

pub trait BlockClone {
    fn clone_box(&self) -> Box<dyn BlockType>;
}

pub trait BlockType: BlockClone {
    fn name(&self) -> Cow<'static, str>;
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
    fn to_string(self) -> &'static str {
        match self {
            Self::Sine => "Sine",
            Self::Square => "Square",
            Self::Triangle => "Triangle",
            Self::Saw => "Saw",
        }
    }
}

#[derive(Clone)]
pub struct FundamentalShapeBlock {
    fundamental_shape_type: FundamentalShapeType,
    should_loop: bool,
    base_frequency: f64,
    base_amplitude: f64,
}

impl FundamentalShapeBlock {
    pub fn new(fundamental_shape_type: FundamentalShapeType) -> Self {
        Self {
            fundamental_shape_type,
            should_loop: false,
            base_frequency: 256.0,
            base_amplitude: 0.5,
        }
    }
}

impl BlockType for FundamentalShapeBlock {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(self.fundamental_shape_type.to_string())
    }
}
