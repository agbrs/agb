use super::DISPLAY_CONTROL;
use crate::memory_mapped::MemoryMapped1DArray;

const OBJECT_ATTRIBUTE_MEMORY: MemoryMapped1DArray<u16, 512> =
    unsafe { MemoryMapped1DArray::new(0x0700_0000) };

#[non_exhaustive]
pub struct ObjectControl {
    object_count: u8,
}

pub struct ObjectStandard {
    attributes: ObjectAttribute,
    id: u8,
}

pub struct ObjectAffine {
    attributes: ObjectAttribute,
    id: u8,
    aff_id: Option<u8>,
}

pub struct AffineMatrix {
    attributes: AffineMatrixAttributes,
    id: u8,
}

pub struct AffineMatrixAttributes {
    p_a: i16,
    p_b: i16,
    p_c: i16,
    p_d: i16,
}

pub enum Mode {
    Normal = 0,
    Affine = 1,
    Hidden = 2,
    AffineDouble = 3,
}

impl ObjectStandard {
    pub fn commit(&self) {
        unsafe { self.attributes.commit(self.id) }
    }

    pub fn set_x(&mut self, x: u8) {
        self.attributes.set_x(x)
    }
    pub fn set_y(&mut self, y: u8) {
        self.attributes.set_y(y)
    }
    pub fn set_tile_id(&mut self, id: u16) {
        self.attributes.set_tile_id(id)
    }
    pub fn set_hflip(&mut self, hflip: bool) {
        self.attributes.set_hflip(hflip)
    }
    pub fn show(&mut self) {
        self.attributes.set_mode(Mode::Normal)
    }
    pub fn hide(&mut self) {
        self.attributes.set_mode(Mode::Hidden)
    }
}

impl ObjectAffine {
    pub fn commit(&self) {
        unsafe { self.attributes.commit(self.id) }
    }

    pub fn set_x(&mut self, x: u8) {
        self.attributes.set_x(x)
    }
    pub fn set_y(&mut self, y: u8) {
        self.attributes.set_y(y)
    }
    pub fn set_tile_id(&mut self, id: u16) {
        self.attributes.set_tile_id(id)
    }

    pub fn show(&mut self) {
        if self.aff_id.is_none() {
            panic!("affine matrix should be set")
        }
        self.attributes.set_mode(Mode::Affine)
    }
    pub fn hide(&mut self) {
        self.attributes.set_mode(Mode::Hidden)
    }
    pub fn set_affine_mat(&mut self, aff: &AffineMatrix) {
        self.attributes.set_affine(aff.id);
        self.aff_id = Some(aff.id);
    }
}

fn set_bits(current: u16, value: u16, length: u16, shift: u16) -> u16 {
    let mask: u16 = (1 << length) - 1;
    (current & !(mask << shift)) | ((value & mask) << shift)
}

pub struct ObjectAttribute {
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

    pub fn set_hflip(&mut self, hflip: bool) {
        self.a1 = set_bits(self.a1, hflip as u16, 1, 0xC);
    }

    pub fn set_x(&mut self, x: u8) {
        self.a1 = set_bits(self.a1, x as u16, 8, 0);
    }

    pub fn set_y(&mut self, y: u8) {
        self.a0 = set_bits(self.a0, y as u16, 8, 0)
    }

    pub fn set_tile_id(&mut self, id: u16) {
        self.a2 = set_bits(self.a2, id, 9, 0);
    }

    pub fn set_mode(&mut self, mode: Mode) {
        self.a0 = set_bits(self.a0, mode as u16, 2, 8);
    }

    pub fn set_affine(&mut self, aff_id: u8) {
        self.a1 = set_bits(self.a1, aff_id as u16, 5, 8);
    }
}

impl AffineMatrix {
    #[allow(clippy::identity_op)]
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
        ObjectControl { object_count: 0 }
    }

    pub fn enable(&mut self) {
        let disp = DISPLAY_CONTROL.get();
        let disp = disp | (1 << 0x0C);
        DISPLAY_CONTROL.set(disp);
    }

    pub fn disable(&mut self) {
        let disp = DISPLAY_CONTROL.get();
        let disp = disp & !(1 << 0x0C);
        DISPLAY_CONTROL.set(disp);
    }

    pub fn get_object(&mut self) -> ObjectStandard {
        let id = self.object_count;
        self.object_count += 1;
        assert!(id < 128, "object id must be less than 128");
        ObjectStandard {
            attributes: ObjectAttribute::new(),
            id,
        }
    }
}
