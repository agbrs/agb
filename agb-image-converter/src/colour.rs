#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Colour {
    pub fn from_rgb(r: u8, g: u8, b: u8, a: u8) -> Self {
        Colour { r, g, b, a }
    }

    pub fn to_rgb15(self) -> u16 {
        let (r, g, b) = (self.r as u16, self.g as u16, self.b as u16);
        ((r >> 3) & 31) | (((g >> 3) & 31) << 5) | (((b >> 3) & 31) << 10)
    }

    pub fn is_transparent(self) -> bool {
        self.a != 255
    }
}
