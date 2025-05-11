use core::cell::Cell;

use alloc::rc::Rc;

use crate::display::affine::AffineMatrixObject;

#[derive(Debug)]
struct AffineMatrixData {
    frame_count: Cell<u32>,
    location: Cell<u32>,
    matrix: AffineMatrixObject,
}

#[derive(Debug, Clone)]
pub(crate) struct AffineMatrixVram(Rc<AffineMatrixData>);

/// An affine matrix that can be used on objects.
///
/// It is just in time copied to vram, so you can have as many as you like
/// of these but you can only use up to 16 in one frame. They are reference
/// counted (Cloning is cheap) and immutable, if you want to change a matrix
/// you must make a new one and set it
/// on all your objects.
#[derive(Debug, Clone)]
pub struct AffineMatrixInstance {
    location: AffineMatrixVram,
}

impl AffineMatrixInstance {
    #[must_use]
    /// Creates an instance of an affine matrix from its object form. Check out
    /// the docs for [AffineMatrix][crate::display::affine::AffineMatrix] to see
    /// how you can use them to create effects.
    pub fn new(affine_matrix: impl Into<AffineMatrixObject>) -> AffineMatrixInstance {
        AffineMatrixInstance {
            location: AffineMatrixVram(Rc::new(AffineMatrixData {
                frame_count: Cell::new(u32::MAX),
                location: Cell::new(u32::MAX),
                matrix: affine_matrix.into(),
            })),
        }
    }

    pub(crate) fn vram(self) -> AffineMatrixVram {
        self.location
    }
}

impl AffineMatrixVram {
    pub fn frame_count(&self) -> u32 {
        self.0.frame_count.get()
    }

    pub fn set_frame_count(&self, frame: u32) {
        self.0.frame_count.set(frame);
    }

    pub fn location(&self) -> u32 {
        self.0.location.get()
    }

    pub fn set_location(&self, location: u32) {
        self.0.location.set(location);
    }

    pub fn write_to_location(&self, oam: *mut u16) {
        let components = self.0.matrix.components();
        let location = self.0.location.get() as usize;
        for (idx, component) in components.iter().enumerate() {
            unsafe {
                oam.add(location * 16 + idx * 4 + 3)
                    .write_volatile(*component);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn niche_optimisation(_gba: &mut crate::Gba) {
        assert_eq!(
            core::mem::size_of::<AffineMatrixInstance>(),
            core::mem::size_of::<Option<AffineMatrixInstance>>()
        );
    }
}
