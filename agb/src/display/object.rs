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
        let mask = 1 << 0xC;
        let attr = self.a1;
        let attr = attr & !mask;
        if hflip {
            self.a1 = attr | mask
        } else {
            self.a1 = attr
        }
    }

    pub fn set_x(&mut self, x: u8) {
        let mask = (1 << 8) - 1;
        let attr1 = self.a1;
        let attr_without_x = attr1 & !mask;
        let attr_with_new_x = attr_without_x | x as u16;
        self.a1 = attr_with_new_x;
    }

    pub fn set_y(&mut self, y: u8) {
        let mask = (1 << 8) - 1;
        let attr0 = self.a0;
        let attr_without_y = attr0 & !mask;
        let attr_with_new_y = attr_without_y | y as u16;
        self.a0 = attr_with_new_y;
    }

    pub fn set_tile_id(&mut self, id: u16) {
        let mask = (1 << 9) - 1;
        assert!(id <= mask, "tile id is greater than 9 bits");
        let attr = self.a2;
        let attr = attr & !mask;
        let attr = attr | id;
        self.a2 = attr;
    }

    pub fn set_mode(&mut self, mode: Mode) {
        let mask = 0b11 << 0x8;
        self.a0 = (self.a0 & !mask) | ((mode as u16) << 0x8);
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
