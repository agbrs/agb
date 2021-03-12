#[non_exhaustive]
pub struct Object {}

impl Object {
    pub(crate) fn new() -> Self {
        Object {}
    }

    pub fn enable(&mut self) {}

    pub fn disable(&mut self) {}
}
