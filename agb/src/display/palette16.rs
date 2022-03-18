#[repr(C)]
#[derive(Clone)]
pub struct Palette16 {
    pub(crate) colours: [u16; 16],
}

impl Palette16 {
    pub const fn new(colours: [u16; 16]) -> Self {
        Palette16 { colours }
    }

    #[allow(clippy::only_used_in_recursion)]
    pub fn update_colour(&mut self, index: usize, colour: u16) {
        self.colours[index] = colour;
    }

    pub fn get_colour(&self, index: usize) -> u16 {
        self.colours[index]
    }
}
