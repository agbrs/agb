use super::DISPLAY_CONTROL;
use crate::memory_mapped::MemoryMapped1DArray;

const OBJECT_MEMORY_STANDARD: MemoryMapped1DArray<u32, 256> =
    unsafe { MemoryMapped1DArray::new(0x0700_0000) };

#[non_exhaustive]
pub struct ObjectControl {}

pub struct ObjectStandard {
    attributes: ObjectAttribute,
    id: usize,
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
    pub fn set_tile_id(&mut self, id: u32) {
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

pub struct ObjectAttribute {
    low: u32,
    high: u32,
}

impl ObjectAttribute {
    unsafe fn commit(&self, index: usize) {
        OBJECT_MEMORY_STANDARD.set(index * 2, self.low);
        // preserve top set of bits
        let high_bits = OBJECT_MEMORY_STANDARD.get(index * 2 + 1);
        let new_high_bits = (high_bits & !((1 << 16) - 1)) | self.high;
        OBJECT_MEMORY_STANDARD.set(index * 2 + 1, new_high_bits);
    }

    pub fn set_hflip(&mut self, hflip: bool) {
        let mask = (1 << 0xC) << 16;
        let attr = self.low;
        let attr = attr & !mask;
        if hflip {
            self.low = attr | mask
        } else {
            self.low = attr
        }
    }

    pub fn set_x(&mut self, x: u8) {
        let mask = ((1 << 8) - 1) << 16;
        let attr1 = self.low;
        let attr_without_x = attr1 & !mask;
        let attr_with_new_x = attr_without_x | ((x as u32) << 16);
        self.low = attr_with_new_x;
    }

    pub fn set_y(&mut self, y: u8) {
        let mask = (1 << 8) - 1;
        let attr0 = self.low;
        let attr_without_y = attr0 & !mask;
        let attr_with_new_y = attr_without_y | y as u32;
        self.low = attr_with_new_y;
    }

    pub fn set_tile_id(&mut self, id: u32) {
        let mask = (1 << 9) - 1;
        assert!(id <= mask, "tile id is greater than 9 bits");
        let attr = self.high;
        let attr = attr & !mask;
        let attr = attr | id;
        self.high = attr;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        let mask = 0b11 << 0x8;
        self.low = (self.low & !mask) | ((mode as u32) << 0x8);
    }
}

impl ObjectAttribute {
    fn new() -> Self {
        ObjectAttribute { low: 0, high: 0 }
    }
}

impl ObjectControl {
    pub(crate) fn new() -> Self {
        ObjectControl {}
    }

    /// # Safety
    /// Temporary, do not call if you currently hold an object
    pub unsafe fn clear_objects(&mut self) {
        let mut o = ObjectAttribute::new();
        o.set_mode(Mode::Hidden);
        for index in 0..128 {
            o.commit(index);
        }
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

    /// # Safety
    /// Temporary function, you must not get multiple objects with the same id
    pub unsafe fn get_object(&self, id: usize) -> ObjectStandard {
        assert!(id < 128, "object id must be less than 128");
        ObjectStandard {
            attributes: ObjectAttribute::new(),
            id,
        }
    }
}
