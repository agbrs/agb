#[repr(C)]
#[derive(Clone)]
pub struct Palette16 {
    pub(crate) colours: [u16; 16],
}

impl Palette16 {
    #[must_use]
    pub const fn new(colours: [u16; 16]) -> Self {
        Palette16 { colours }
    }

    pub fn update_colour(&mut self, index: usize, colour: u16) {
        self.colours[index] = colour;
    }

    #[must_use]
    pub fn colour(&self, index: usize) -> u16 {
        self.colours[index]
    }
}

#[macro_export]
macro_rules! include_palette {
    ($palette:literal) => {
        $crate::include_colours_inner!($palette)
    };
}

pub use include_palette;
