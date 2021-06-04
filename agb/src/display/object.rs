use super::DISPLAY_CONTROL;
use crate::memory_mapped::MemoryMapped1DArray;

const OBJECT_ATTRIBUTE_MEMORY: MemoryMapped1DArray<u16, 512> =
    unsafe { MemoryMapped1DArray::new(0x0700_0000) };

/// Handles distributing objects and matricies along with operations that effect all objects.
pub struct ObjectControl {
    object_count: u8,
    affine_count: u8,
}

/// The standard object, without rotation.
pub struct ObjectStandard {
    attributes: ObjectAttribute,
    id: u8,
}

/// The affine object, with potential for using a transformation matrix to alter
/// how the sprite is rendered to screen.
pub struct ObjectAffine {
    attributes: ObjectAttribute,
    id: u8,
    aff_id: Option<u8>,
}

/// Refers to an affine matrix in the OAM. Includes both an index and the
/// components of the affine matrix.
pub struct AffineMatrix {
    pub attributes: AffineMatrixAttributes,
    id: u8,
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

impl ObjectStandard {
    /// Commits the object to OAM such that the updated version is displayed on
    /// screen. Recommend to do this during VBlank.
    pub fn commit(&self) {
        unsafe { self.attributes.commit(self.id) }
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

impl ObjectAffine {
    /// Commits the object to OAM such that the updated version is displayed on
    /// screen. Recommend to do this during VBlank.
    pub fn commit(&self) {
        unsafe { self.attributes.commit(self.id) }
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
        self.attributes.set_affine(aff.id);
        self.aff_id = Some(aff.id);
    }
}

fn set_bits(current: u16, value: u16, length: u16, shift: u16) -> u16 {
    let mask: u16 = (1 << length) - 1;
    (current & !(mask << shift)) | ((value & mask) << shift)
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

impl AffineMatrix {
    #[allow(clippy::identity_op)]
    /// Commits matrix to OAM, will cause any objects using this matrix to be updated.
    pub fn commit(&self) {
        let id = self.id as usize;
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
            object_count: 0,
            affine_count: 0,
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

    /// Get an unused standard object. Currently dropping an unused object will
    /// not free this. You should either keep around all objects you need
    /// forever or drop and reobtain ObjectControl. Panics if more than 128
    /// objects are obtained.
    pub fn get_object_standard(&mut self) -> ObjectStandard {
        let id = self.object_count;
        self.object_count += 1;
        assert!(id < 128, "object id must be less than 128");
        ObjectStandard {
            attributes: ObjectAttribute::new(),
            id,
        }
    }

    /// Get an unused affine object. Currently dropping an unused object will
    /// not free this. You should either keep around all objects you need
    /// forever or drop and reobtain ObjectControl. Panics if more than 128
    /// objects are obtained.
    pub fn get_object_affine(&mut self) -> ObjectAffine {
        let id = self.object_count;
        self.object_count += 1;
        assert!(id < 128, "object id must be less than 128");
        ObjectAffine {
            attributes: ObjectAttribute::new(),
            id,
            aff_id: None,
        }
    }

    /// Get an unused affine matrix. Currently dropping an unused object will
    /// not free this. You should either keep around all affine matricies you
    /// need forever or drop and reobtain ObjectControl. Panics if more than 32
    /// affine matricies are obtained.
    pub fn get_affine(&mut self) -> AffineMatrix {
        let id = self.affine_count;
        self.affine_count += 1;
        assert!(id < 32, "affine id must be less than 32");
        AffineMatrix {
            attributes: AffineMatrixAttributes {
                p_a: 0,
                p_b: 0,
                p_c: 0,
                p_d: 0,
            },
            id,
        }
    }
}
