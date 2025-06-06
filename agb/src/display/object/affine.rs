use core::cell::Cell;

use agb_fixnum::{FixedWidthSignedInteger, Num};
use alloc::rc::Rc;

use crate::display::affine::AffineMatrix;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(C, packed(4))]
/// An affine matrix that can be used in affine objects
///
/// ```txt
/// a b
/// c d
/// ```
struct AffineMatrixObjectElements {
    pub a: Num<i16, 8>,
    pub b: Num<i16, 8>,
    pub c: Num<i16, 8>,
    pub d: Num<i16, 8>,
}

impl Default for AffineMatrixObjectElements {
    fn default() -> Self {
        Self::from(AffineMatrix::<Num<i16, 8>>::identity())
    }
}

impl AffineMatrixObjectElements {
    #[must_use]
    /// Converts to the affine matrix that is usable in performing efficient
    /// calculations.
    pub fn to_affine_matrix(self) -> AffineMatrix<Num<i16, 8>> {
        AffineMatrix {
            a: self.a,
            b: self.b,
            c: self.c,
            d: self.d,
            x: 0.into(),
            y: 0.into(),
        }
    }

    #[must_use]
    /// Converts from an affine matrix, wrapping if it overflows
    pub fn from_affine<I, const N: usize>(affine: AffineMatrix<Num<I, N>>) -> Self
    where
        I: FixedWidthSignedInteger,
        i32: From<I>,
    {
        let a: Num<i32, 8> = affine.a.change_base();
        let b: Num<i32, 8> = affine.b.change_base();
        let c: Num<i32, 8> = affine.c.change_base();
        let d: Num<i32, 8> = affine.d.change_base();

        Self {
            a: Num::from_raw(a.to_raw() as i16),
            b: Num::from_raw(b.to_raw() as i16),
            c: Num::from_raw(c.to_raw() as i16),
            d: Num::from_raw(d.to_raw() as i16),
        }
    }

    pub(crate) fn components(self) -> [u16; 4] {
        [
            self.a.to_raw() as u16,
            self.b.to_raw() as u16,
            self.c.to_raw() as u16,
            self.d.to_raw() as u16,
        ]
    }
}

impl From<AffineMatrixObjectElements> for AffineMatrix<Num<i16, 8>> {
    fn from(mat: AffineMatrixObjectElements) -> Self {
        mat.to_affine_matrix()
    }
}

impl<I, const N: usize> From<AffineMatrix<Num<I, N>>> for AffineMatrixObjectElements
where
    I: FixedWidthSignedInteger,
    i32: From<I>,
{
    fn from(value: AffineMatrix<Num<I, N>>) -> Self {
        AffineMatrixObjectElements::from_affine(value)
    }
}

#[derive(Debug)]
struct AffineMatrixData {
    frame_count: Cell<u32>,
    location: Cell<u32>,
    matrix: AffineMatrixObjectElements,
}

#[derive(Debug, Clone)]
pub(crate) struct AffineMatrixVram(Rc<AffineMatrixData>);

/// An affine matrix that can be used on objects.
///
/// It is just in time copied to vram, so you can have as many as you like
/// of these but you can only use up to 32 in one frame. They are reference
/// counted (Cloning is cheap) and immutable, if you want to change a matrix
/// you must make a new one and set it
/// on all your objects.
#[derive(Debug, Clone)]
pub struct AffineMatrixObject {
    location: AffineMatrixVram,
}

impl AffineMatrixObject {
    fn new_inner(affine_matrix: AffineMatrixObjectElements) -> AffineMatrixObject {
        AffineMatrixObject {
            location: AffineMatrixVram(Rc::new(AffineMatrixData {
                frame_count: Cell::new(u32::MAX),
                location: Cell::new(u32::MAX),
                matrix: affine_matrix,
            })),
        }
    }

    #[must_use]
    /// Creates an instance of an affine matrix from its object form. Check out
    /// the docs for [AffineMatrix][crate::display::affine::AffineMatrix] to see
    /// how you can use them to create effects.
    pub fn new<I, const N: usize>(affine_matrix: AffineMatrix<Num<I, N>>) -> Self
    where
        I: FixedWidthSignedInteger,
        i32: From<I>,
    {
        Self::new_inner(affine_matrix.into())
    }

    pub(crate) fn vram(self) -> AffineMatrixVram {
        self.location
    }
}

impl From<AffineMatrix> for AffineMatrixObject {
    fn from(value: AffineMatrix) -> Self {
        AffineMatrixObject::new(value)
    }
}

impl Default for AffineMatrixObject {
    fn default() -> Self {
        Self::new_inner(AffineMatrixObjectElements::default())
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
            core::mem::size_of::<AffineMatrixObject>(),
            core::mem::size_of::<Option<AffineMatrixObject>>()
        );
    }
}
