use super::{
    bitmap3::Bitmap3,
    bitmap4::Bitmap4,
    tiled::{TiledBackground, VRamManager},
};

/// The video struct controls access to the video hardware.
/// It ensures that only one video mode is active at a time.
///
/// Most games will use tiled modes, as bitmap modes are too slow to run at the full 60 FPS.
#[non_exhaustive]
pub struct Video;

impl Video {
    /// Bitmap mode that provides a 16-bit colour framebuffer
    pub fn bitmap3(&mut self) -> Bitmap3<'_> {
        unsafe { Bitmap3::new() }
    }

    /// Bitmap 4 provides two 8-bit paletted framebuffers with page switching
    pub fn bitmap4(&mut self) -> Bitmap4<'_> {
        unsafe { Bitmap4::new() }
    }

    /// Tiled mode allows for up to 4 backgrounds
    pub fn tiled(&mut self) -> (TiledBackground<'_>, VRamManager) {
        (unsafe { TiledBackground::new() }, VRamManager::new())
    }
}
