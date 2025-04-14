#![warn(missing_docs)]
use super::Rgb15;

#[repr(C)]
#[derive(Clone)]
pub struct Palette16 {
    pub(crate) colours: [Rgb15; 16],
}

impl Palette16 {
    #[must_use]
    pub const fn new(colours: [Rgb15; 16]) -> Self {
        Palette16 { colours }
    }

    pub fn update_colour(&mut self, index: usize, colour: Rgb15) {
        self.colours[index] = colour;
    }

    #[must_use]
    pub fn colour(&self, index: usize) -> Rgb15 {
        self.colours[index]
    }
}
