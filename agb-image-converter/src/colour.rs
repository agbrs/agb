#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Colour {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Colour { r, g, b }
    }
}
