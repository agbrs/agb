use core::cell::Cell;

use alloc::rc::Rc;

use crate::display::affine::AffineMatrixObject;

use super::OBJECT_ATTRIBUTE_MEMORY;

#[derive(Debug)]
struct AffineMatrixData {
    frame_count: Cell<u32>,
    location: Cell<u32>,
    matrix: AffineMatrixObject,
}

#[derive(Debug, Clone)]
pub(crate) struct AffineMatrixVram(Rc<AffineMatrixData>);

#[derive(Debug, Clone)]
pub struct AffineMatrix {
    location: AffineMatrixVram,
}

impl AffineMatrix {
    #[must_use]
    pub fn new(affine_matrix: AffineMatrixObject) -> AffineMatrix {
        AffineMatrix {
            location: AffineMatrixVram(Rc::new(AffineMatrixData {
                frame_count: Cell::new(u32::MAX),
                location: Cell::new(u32::MAX),
                matrix: affine_matrix,
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

    pub fn write_to_location(&self) {
        let components = self.0.matrix.components();
        let location = self.0.location.get() as usize;
        for (idx, component) in components.iter().enumerate() {
            unsafe {
                (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                    .add(location * 16 + idx * 4 + 3)
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
            core::mem::size_of::<AffineMatrix>(),
            core::mem::size_of::<Option<AffineMatrix>>()
        );
    }
}
