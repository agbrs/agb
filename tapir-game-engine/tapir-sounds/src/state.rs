#[derive(Default)]
pub struct State {
    pub blocks: im::Vector<Block>,
}

#[derive(Clone)]
pub struct Block {
    pub name: String,
}

impl Block {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}
