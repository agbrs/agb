use core::cell::RefCell;

use super::DISPLAY_CONTROL;
use crate::bitarray::Bitarray;
use crate::memory_mapped::MemoryMapped1DArray;

const OBJECT_ATTRIBUTE_MEMORY: MemoryMapped1DArray<u16, 512> =
    unsafe { MemoryMapped1DArray::new(0x0700_0000) };

/// Handles distributing objects and matricies along with operations that effect all objects.
pub struct ObjectControl {
    objects: RefCell<Bitarray<4>>,
    affines: RefCell<Bitarray<1>>,
}

struct ObjectLoan<'a> {
    index: u8,
    objects: &'a RefCell<Bitarray<4>>,
}

struct AffineLoan<'a> {
    index: u8,
    affines: &'a RefCell<Bitarray<1>>,
}

/// The standard object, without rotation.
pub struct ObjectStandard<'a> {
    attributes: ObjectAttribute,
    loan: ObjectLoan<'a>,
}

/// The affine object, with potential for using a transformation matrix to alter
/// how the sprite is rendered to screen.
pub struct ObjectAffine<'a> {
    attributes: ObjectAttribute,
    loan: ObjectLoan<'a>,
    aff_id: Option<u8>,
}

/// Refers to an affine matrix in the OAM. Includes both an index and the
/// components of the affine matrix.
pub struct AffineMatrix<'a> {
    pub attributes: AffineMatrixAttributes,
    loan: AffineLoan<'a>,
}

/// The components of the affine matrix. The components are fixed point 8:8.
/// TODO is a type that can handle fixed point arithmetic.
pub struct AffineMatrixAttributes {
    pub p_a: i16,
    pub p_b: i16,
    pub p_c: i16,
    pub p_d: i16,
}

enum Mode {
    Normal = 0,
    Affine = 1,
    Hidden = 2,
    AffineDouble = 3,
}

#[derive(Clone, Copy)]
pub enum Size {
    // stored as attr0 attr1
    S8x8 = 0b00_00,
    S16x16 = 0b00_01,
    S32x32 = 0b00_10,
    S64x64 = 0b00_11,

    S16x8 = 0b01_00,
    S32x8 = 0b01_01,
    S32x16 = 0b01_10,
    S64x32 = 0b01_11,

    S8x16 = 0b10_00,
    S8x32 = 0b10_01,
    S16x32 = 0b10_10,
    S32x64 = 0b10_11,
}

impl ObjectStandard<'_> {
    /// Commits the object to OAM such that the updated version is displayed on
    /// screen. Recommend to do this during VBlank.
    pub fn commit(&self) {
        unsafe { self.attributes.commit(self.loan.index) }
    }

    /// Sets the x coordinate of the sprite on screen.
    pub fn set_x(&mut self, x: u8) {
        self.attributes.set_x(x)
    }
    /// Sets the y coordinate of the sprite on screen.
    pub fn set_y(&mut self, y: u8) {
        self.attributes.set_y(y)
    }
    /// Sets the index of the tile to use as the sprite. Potentially a temporary function.
    pub fn set_tile_id(&mut self, id: u16) {
        self.attributes.set_tile_id(id)
    }
    /// Sets whether the sprite is horizontally mirrored or not.
    pub fn set_hflip(&mut self, hflip: bool) {
        self.attributes.set_hflip(hflip)
    }
    /// Sets the sprite size, will read tiles in x major order to construct this.
    pub fn set_sprite_size(&mut self, size: Size) {
        self.attributes.set_size(size);
    }
    /// Show the object on screen.
    pub fn show(&mut self) {
        self.attributes.set_mode(Mode::Normal)
    }
    /// Hide the object and do not render.
    pub fn hide(&mut self) {
        self.attributes.set_mode(Mode::Hidden)
    }
}

impl ObjectAffine<'_> {
    /// Commits the object to OAM such that the updated version is displayed on
    /// screen. Recommend to do this during VBlank.
    pub fn commit(&self) {
        unsafe { self.attributes.commit(self.loan.index) }
    }

    /// Sets the x coordinate of the sprite on screen.
    pub fn set_x(&mut self, x: u8) {
        self.attributes.set_x(x)
    }
    /// Sets the y coordinate of the sprite on screen.
    pub fn set_y(&mut self, y: u8) {
        self.attributes.set_y(y)
    }
    /// Sets the index of the tile to use as the sprite. Potentially a temporary function.
    pub fn set_tile_id(&mut self, id: u16) {
        self.attributes.set_tile_id(id)
    }
    /// Sets the sprite size, will read tiles in x major order to construct this.
    pub fn set_sprite_size(&mut self, size: Size) {
        self.attributes.set_size(size);
    }

    /// Show the object on screen. Panics if affine matrix has not been set.
    pub fn show(&mut self) {
        if self.aff_id.is_none() {
            panic!("affine matrix should be set")
        }
        self.attributes.set_mode(Mode::Affine)
    }
    /// Hide the object and do not render the sprite.
    pub fn hide(&mut self) {
        self.attributes.set_mode(Mode::Hidden)
    }
    /// Sets the affine matrix to use. Changing the affine matrix will change
    /// how the sprite is rendered.
    pub fn set_affine_mat(&mut self, aff: &AffineMatrix) {
        self.attributes.set_affine(aff.loan.index);
        self.aff_id = Some(aff.loan.index);
    }
}

fn set_bits(current: u16, value: u16, length: u16, shift: u16) -> u16 {
    let mask: u16 = (1 << length) - 1;
    (current & !(mask << shift)) | ((value & mask) << shift)
}

impl Drop for ObjectLoan<'_> {
    fn drop(&mut self) {
        let mut objs = self.objects.borrow_mut();
        objs.set(self.index as usize, false);
    }
}

impl Drop for AffineLoan<'_> {
    fn drop(&mut self) {
        let mut affs = self.affines.borrow_mut();
        affs.set(self.index as usize, false);
    }
}

struct ObjectAttribute {
    a0: u16,
    a1: u16,
    a2: u16,
}

impl ObjectAttribute {
    unsafe fn commit(&self, index: u8) {
        OBJECT_ATTRIBUTE_MEMORY.set(index as usize * 4, self.a0);
        OBJECT_ATTRIBUTE_MEMORY.set(index as usize * 4 + 1, self.a1);
        OBJECT_ATTRIBUTE_MEMORY.set(index as usize * 4 + 2, self.a2);
    }

    fn set_hflip(&mut self, hflip: bool) {
        self.a1 = set_bits(self.a1, hflip as u16, 1, 0xC);
    }

    fn set_size(&mut self, size: Size) {
        let lower = size as u16 & 0b11;
        let upper = (size as u16 >> 2) & 0b11;

        self.a0 = set_bits(self.a0, lower, 2, 0xE);
        self.a1 = set_bits(self.a1, upper, 2, 0xE);
    }

    fn set_x(&mut self, x: u8) {
        self.a1 = set_bits(self.a1, x as u16, 8, 0);
    }

    fn set_y(&mut self, y: u8) {
        self.a0 = set_bits(self.a0, y as u16, 8, 0)
    }

    fn set_tile_id(&mut self, id: u16) {
        self.a2 = set_bits(self.a2, id, 9, 0);
    }

    fn set_mode(&mut self, mode: Mode) {
        self.a0 = set_bits(self.a0, mode as u16, 2, 8);
    }

    fn set_affine(&mut self, aff_id: u8) {
        self.a1 = set_bits(self.a1, aff_id as u16, 5, 8);
    }
}

impl AffineMatrix<'_> {
    #[allow(clippy::identity_op)]
    /// Commits matrix to OAM, will cause any objects using this matrix to be updated.
    pub fn commit(&self) {
        let id = self.loan.index as usize;
        OBJECT_ATTRIBUTE_MEMORY.set((id + 0) * 4 + 3, self.attributes.p_a as u16);
        OBJECT_ATTRIBUTE_MEMORY.set((id + 1) * 4 + 3, self.attributes.p_b as u16);
        OBJECT_ATTRIBUTE_MEMORY.set((id + 2) * 4 + 3, self.attributes.p_c as u16);
        OBJECT_ATTRIBUTE_MEMORY.set((id + 3) * 4 + 3, self.attributes.p_d as u16);
    }
}

impl ObjectAttribute {
    fn new() -> Self {
        ObjectAttribute {
            a0: 0,
            a1: 0,
            a2: 0,
        }
    }
}

impl ObjectControl {
    pub(crate) fn new() -> Self {
        let mut o = ObjectAttribute::new();
        o.set_mode(Mode::Hidden);
        for index in 0..128 {
            unsafe { o.commit(index) };
        }
        ObjectControl {
            objects: RefCell::new(Bitarray::new()),
            affines: RefCell::new(Bitarray::new()),
        }
    }

    /// Enable objects on the GBA.
    pub fn enable(&mut self) {
        let disp = DISPLAY_CONTROL.get();
        let disp = disp | (1 << 0x0C);
        DISPLAY_CONTROL.set(disp);
    }

    /// Disable objects, objects won't be rendered.
    pub fn disable(&mut self) {
        let disp = DISPLAY_CONTROL.get();
        let disp = disp & !(1 << 0x0C);
        DISPLAY_CONTROL.set(disp);
    }

    fn get_unused_object_index(&self) -> u8 {
        let mut objects = self.objects.borrow_mut();
        for index in 0..128 {
            if !objects.get(index).unwrap() {
                objects.set(index, true);
                return index as u8;
            }
        }
        panic!("object id must be less than 128");
    }

    fn get_unused_affine_index(&self) -> u8 {
        let mut affines = self.affines.borrow_mut();
        for index in 0..32 {
            if !affines.get(index).unwrap() {
                affines.set(index, true);
                return index as u8;
            }
        }
        panic!("affine id must be less than 32");
    }

    /// Get an unused standard object. Panics if more than 128 objects are
    /// obtained.
    pub fn get_object_standard(&self) -> ObjectStandard {
        let id = self.get_unused_object_index();
        ObjectStandard {
            attributes: ObjectAttribute::new(),
            loan: ObjectLoan {
                objects: &self.objects,
                index: id,
            },
        }
    }

    /// Get an unused affine object. Panics if more than 128 objects are
    /// obtained.
    pub fn get_object_affine(&self) -> ObjectAffine {
        let id = self.get_unused_object_index();
        ObjectAffine {
            attributes: ObjectAttribute::new(),
            loan: ObjectLoan {
                objects: &self.objects,
                index: id,
            },
            aff_id: None,
        }
    }

    /// Get an unused affine matrix. Panics if more than 32 affine matricies are
    /// obtained.
    pub fn get_affine(&self) -> AffineMatrix {
        let id = self.get_unused_affine_index();
        AffineMatrix {
            attributes: AffineMatrixAttributes {
                p_a: 0,
                p_b: 0,
                p_c: 0,
                p_d: 0,
            },
            loan: AffineLoan {
                affines: &self.affines,
                index: id,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn get_and_release(gba: &mut crate::Gba) {
        let gfx = gba.display.video.tiled0();
        let objs = gfx.object;

        {
            let o0 = objs.get_object_standard();
            let o1 = objs.get_object_standard();
            assert_eq!(o0.loan.index, 0);
            assert_eq!(o1.loan.index, 1);
        }

        let o0 = objs.get_object_standard();
        assert_eq!(o0.loan.index, 0);
    }
}
