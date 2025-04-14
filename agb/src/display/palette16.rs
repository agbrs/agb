#![warn(missing_docs)]
use super::Rgb15;

/// Represents a palette of 16 colours.
///
/// The Game Boy Advance can have up to 16, 16 colour palettes active at once. For
/// objects, these are loaded dynamically as needed, but for backgrounds you will
/// need to manually load the palettes using
/// [`VRamManager::set_background_palette`](crate::display::tiled::VRamManager::set_background_palette)
#[repr(C)]
#[derive(Clone)]
pub struct Palette16 {
    pub(crate) colours: [Rgb15; 16],
}

impl Palette16 {
    /// Create a new palette with the given 16 colours.
    #[must_use]
    pub const fn new(colours: [Rgb15; 16]) -> Self {
        Palette16 { colours }
    }

    /// Set the colour at given `index` to the given colour.
    ///
    /// Index must be less than 16 or this function will panic.
    pub const fn update_colour(&mut self, index: usize, colour: Rgb15) {
        self.colours[index] = colour;
    }

    /// Gets the colour for a given index.
    ///
    /// Index must be less than 16 or this function will panic.
    #[must_use]
    pub const fn colour(&self, index: usize) -> Rgb15 {
        self.colours[index]
    }
}
