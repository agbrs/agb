#[derive(Clone)]
pub struct Palette16 {
    pub(crate) colours: [u16; 16],
}

impl Palette16 {
    pub const fn new(colours: [u16; 16]) -> Self {
        Palette16 { colours }
    }
}
