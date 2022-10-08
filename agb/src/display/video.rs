use super::{
    bitmap3::Bitmap3,
    bitmap4::Bitmap4,
    tiled::{Tiled0, Tiled1, Tiled2, VRamManager},
};

/// The video struct controls access to the video hardware.
/// It ensures that only one video mode is active at a time.
///
/// Most games will use tiled modes, as bitmap modes are too slow to run at the full 60 FPS.
#[non_exhaustive]
pub struct Video;

impl Video {
    /// Bitmap mode that provides a 16-bit colour framebuffer
    pub fn bitmap3(&mut self) -> Bitmap3 {
        unsafe { Bitmap3::new() }
    }

    /// Bitmap 4 provides two 8-bit paletted framebuffers with page switching
    pub fn bitmap4(&mut self) -> Bitmap4 {
        unsafe { Bitmap4::new() }
    }

    /// Tiled 0 mode provides 4 regular, tiled backgrounds
    pub fn tiled0(&mut self) -> (Tiled0, VRamManager) {
        (unsafe { Tiled0::new() }, VRamManager::new())
    }

    /// Tiled 1 mode provides 2 regular tiled backgrounds and 1 affine tiled background
    pub fn tiled1(&mut self) -> (Tiled1, VRamManager) {
        (unsafe { Tiled1::new() }, VRamManager::new())
    }

    /// Tiled 2 mode provides 2 affine tiled backgrounds
    pub fn tiled2(&mut self) -> (Tiled2, VRamManager) {
        (unsafe { Tiled2::new() }, VRamManager::new())
    }
}
