use alloc::rc::Rc;

use crate::{display::affine::AffineMatrixObject, sync::Static};

use super::OBJECT_ATTRIBUTE_MEMORY;

#[derive(Debug)]
struct AffineMatrixLocation {
    location: u16,
}

#[derive(Debug, Clone)]
pub(crate) struct AffineMatrixVram(Rc<AffineMatrixLocation>);

#[derive(Debug)]
pub struct AffineMatrix {
    location: AffineMatrixVram,
}

impl AffineMatrix {
    pub fn new(affine_matrix: AffineMatrixObject) -> Option<AffineMatrix> {
        let mut matrix = AFFINE_MATRIX_DISTRIBUTOR.get_matrix()?;
        matrix.write(affine_matrix);

        Some(matrix)
    }

    pub(crate) fn vram(self) -> AffineMatrixVram {
        self.location
    }

    fn write(&mut self, affine_matrix: AffineMatrixObject) {
        let components = affine_matrix.components();
        let location = self.location.0.location as usize;
        for (idx, component) in components.iter().enumerate() {
            unsafe {
                (OBJECT_ATTRIBUTE_MEMORY as *mut u16)
                    .add(location * 4 * idx + 3)
                    .write_volatile(*component);
            }
        }
    }
}

impl AffineMatrixVram {
    pub fn location(&self) -> u16 {
        self.0.location
    }
}

impl Drop for AffineMatrixLocation {
    fn drop(&mut self) {
        // safety: obtained via affine matrix distributor
        unsafe { AFFINE_MATRIX_DISTRIBUTOR.return_matrix(self.location) }
    }
}

struct AffineMatrixDistributor {
    tip: Static<u16>,
}

static AFFINE_MATRIX_DISTRIBUTOR: AffineMatrixDistributor = AffineMatrixDistributor::new();

pub(crate) unsafe fn init_affine() {
    AFFINE_MATRIX_DISTRIBUTOR.initialise_affine_matricies();
}

impl AffineMatrixDistributor {
    const fn new() -> Self {
        AffineMatrixDistributor {
            tip: Static::new(u16::MAX),
        }
    }

    unsafe fn initialise_affine_matricies(&self) {
        for i in 0..32 {
            let ptr = (OBJECT_ATTRIBUTE_MEMORY as *mut u16).add(i * 16 + 3);

            if i == 31 {
                // none
                ptr.write_volatile(u16::MAX);
            } else {
                ptr.write_volatile(i as u16 + 1);
            }
        }

        self.tip.write(0);
    }

    fn location_of(affine_matrix_location: u16) -> *mut u16 {
        unsafe {
            (OBJECT_ATTRIBUTE_MEMORY as *mut u16).add(affine_matrix_location as usize * 16 + 3)
        }
    }

    fn get_matrix(&self) -> Option<AffineMatrix> {
        let location = self.tip.read();
        if location == u16::MAX {
            return None;
        }

        let next_tip = unsafe { Self::location_of(location).read_volatile() };

        self.tip.write(next_tip);

        Some(AffineMatrix {
            location: AffineMatrixVram(Rc::new(AffineMatrixLocation { location })),
        })
    }

    unsafe fn return_matrix(&self, mat_id: u16) {
        Self::location_of(mat_id).write_volatile(mat_id);
        self.tip.write(mat_id);
    }
}
