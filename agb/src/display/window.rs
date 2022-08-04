use agb_fixnum::Rect;

use crate::memory_mapped::MemoryMapped;

use super::tiled::BackgroundID;

pub struct Windows {
    wins: [MovableWindow; 2],
    out: Window,
    obj: Window,
}

const REG_HORIZONTAL_BASE: *mut u16 = 0x0400_0040 as *mut _;
const REG_VERTICAL_BASE: *mut u16 = 0x0400_0044 as *mut _;

const REG_WINDOW_CONTROL_BASE: *mut u16 = 0x0400_0048 as *mut _;

pub enum WinIn {
    Win0,
    Win1,
}

impl Windows {
    pub(crate) fn new() -> Self {
        let s = Self {
            wins: [MovableWindow::new(), MovableWindow::new()],
            out: Window::new(),
            obj: Window::new(),
        };
        s.commit();
        s
    }

    pub fn enable(&self) {}

    pub fn disable(&self) {}

    pub fn win_out(&mut self) -> &mut Window {
        &mut self.out
    }

    pub fn win_in(&mut self, id: WinIn) -> &mut MovableWindow {
        &mut self.wins[id as usize]
    }

    pub fn win_obj(&mut self) -> &mut Window {
        &mut self.obj
    }

    pub fn commit(&self) {
        for (id, win) in self.wins.iter().enumerate() {
            win.commit(id);
        }
        self.out.commit(2);
        self.obj.commit(3);
    }
}

pub struct Window {
    window_bits: u8,
}

pub struct MovableWindow {
    inner: Window,
    rect: Rect<u8>,
}

impl Window {
    fn new() -> Window {
        Self { window_bits: 0 }
    }

    fn set_bit(&mut self, bit: usize, value: bool) {
        self.window_bits &= u8::MAX ^ (1 << bit);
        self.window_bits |= (value as u8) << bit;
    }

    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new();

        self
    }
    pub fn set_blend_enable(&mut self, blnd: bool) -> &mut Self {
        self.set_bit(5, blnd);

        self
    }
    pub fn set_background_enable(&mut self, back: BackgroundID, enable: bool) -> &mut Self {
        self.set_bit(back.0 as usize, enable);

        self
    }
    pub fn set_object_enable(&mut self, obj: bool) -> &mut Self {
        self.set_bit(4, obj);

        self
    }

    fn commit(&self, id: usize) {
        let base_reg = id / 2;
        let offset_in_reg = (id % 2) * 8;

        unsafe {
            let reg = MemoryMapped::new(REG_WINDOW_CONTROL_BASE.add(base_reg) as usize);
            reg.set_bits(self.window_bits as u16, 8, offset_in_reg as u16);
        }
    }
}

impl MovableWindow {
    fn new() -> Self {
        Self {
            inner: Window::new(),
            rect: Rect::new((0, 0).into(), (0, 0).into()),
        }
    }

    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new();

        self
    }
    pub fn set_blend_enable(&mut self, blnd: bool) -> &mut Self {
        self.inner.set_blend_enable(blnd);
        self
    }
    pub fn set_background_enable(&mut self, back: BackgroundID, enable: bool) -> &mut Self {
        self.inner.set_background_enable(back, enable);
        self
    }
    pub fn set_object_enable(&mut self, obj: bool) -> &mut Self {
        self.inner.set_object_enable(obj);
        self
    }

    fn commit(&self, id: usize) {
        self.inner.commit(id);

        let left_right =
            (self.rect.position.x as u16) << 8 | (self.rect.position.x + self.rect.size.x) as u16;

        let top_bottom =
            (self.rect.position.y as u16) << 8 | (self.rect.position.y + self.rect.size.y) as u16;
        unsafe {
            REG_HORIZONTAL_BASE.add(id).write_volatile(left_right);
            REG_VERTICAL_BASE.add(id).write_volatile(top_bottom);
        }
    }

    pub fn set_position_u8(&mut self, rect: Rect<u8>) -> &mut Self {
        self.rect = rect;

        self
    }

    pub fn set_position(&mut self, rect: &Rect<i32>) -> &mut Self {
        let new_rect = Rect::new(
            (rect.position.x as u8, rect.position.y as u8).into(),
            (rect.size.x as u8, rect.size.y as u8).into(),
        );
        self.set_position_u8(new_rect)
    }
}
