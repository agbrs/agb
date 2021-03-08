use super::{bitmap3::Bitmap3, bitmap4::Bitmap4};

#[non_exhaustive]
pub struct Video {}

impl Video {
    /// Bitmap mode that provides a 16-bit colour framebuffer
    pub fn bitmap3(&mut self) -> Bitmap3 {
        unsafe { Bitmap3::new() }
    }

    /// Bitmap 4 provides two 8-bit paletted framebuffers with page switching
    pub fn bitmap4(&mut self) -> Bitmap4 {
        unsafe { Bitmap4::new() }
    }
}
