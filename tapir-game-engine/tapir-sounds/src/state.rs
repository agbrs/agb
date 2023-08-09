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
    pub name: String,
    pub id: Id,
}

impl Block {
    pub fn new(name: String) -> Self {
        Self {
            name,
            id: Id::new(),
        }
    }
}
