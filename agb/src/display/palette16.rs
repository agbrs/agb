pub struct Palette16 {
    colours: [u16; 16],
}

impl Palette16 {
    pub const fn new(colours: [u16; 16]) -> Self {
        Palette16 { colours }
    }
}
