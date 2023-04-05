use core::alloc::Layout;

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

    // Clippy bug: claims that index is only used in recursion. I can't reproduce in
    // other examples, even just copy pasting this struct and impl into a blank project :/
    pub fn update_colour(&mut self, index: usize, colour: u16) {
        self.colours[index] = colour;
    }

    #[must_use]
    pub fn colour(&self, index: usize) -> u16 {
        self.colours[index]
    }

    pub(crate) const fn layout() -> Layout {
        Layout::new::<Self>()
    }
}
