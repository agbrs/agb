#![deny(missing_docs)]
//! The window feature of the GBA.
use crate::{fixnum::Rect, memory_mapped::MemoryMapped};

use super::{tiled::BackgroundID, DISPLAY_CONTROL, HEIGHT, WIDTH};

/// The windows feature of the Game Boy Advance can selectively display
/// backgrounds or objects on the screen and can selectively enable and disable
/// effects. This gives out references and holds changes before they can be committed.
pub struct Windows {
    wins: [MovableWindow; 2],
    out: Window,
    obj: Window,
}

const REG_HORIZONTAL_BASE: *mut u16 = 0x0400_0040 as *mut _;
const REG_VERTICAL_BASE: *mut u16 = 0x0400_0044 as *mut _;

const REG_WINDOW_CONTROL_BASE: *mut u16 = 0x0400_0048 as *mut _;

/// The two Windows that have an effect inside of them
pub enum WinIn {
    /// The higher priority window
    Win0,
    /// The lower priority window
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

    /// Returns a reference to the window that is used when outside all other windows
    #[inline(always)]
    pub fn win_out(&mut self) -> &mut Window {
        &mut self.out
    }

    /// Gives a reference to the specified window that has effect when inside of it's boundary
    #[inline(always)]
    pub fn win_in(&mut self, id: WinIn) -> &mut MovableWindow {
        &mut self.wins[id as usize]
    }

    /// Gives a reference to the window that is controlled by sprites and objects
    #[inline(always)]
    pub fn win_obj(&mut self) -> &mut Window {
        &mut self.obj
    }

    /// Commits the state of the windows as dictated by the various functions to
    /// modify them. This should be done during vblank shortly after the wait
    /// for next vblank call.
    pub fn commit(&self) {
        for (id, win) in self.wins.iter().enumerate() {
            win.commit(id);
        }
        self.out.commit(2);
        self.obj.commit(3);

        let enabled_bits = ((self.obj.is_enabled() as u16) << 2)
            | ((self.wins[1].is_enabled() as u16) << 1)
            | (self.wins[0].is_enabled() as u16);
        DISPLAY_CONTROL.set_bits(enabled_bits, 3, 0xD);
    }
}

/// A non movable window
pub struct Window {
    window_bits: u8,
}

/// A window that can be moved
pub struct MovableWindow {
    inner: Window,
    rect: Rect<u8>,
}

impl Window {
    fn new() -> Window {
        Self { window_bits: 0 }
    }

    /// Enables the window, must call [Windows::commit] for this change to be
    /// seen. If a window is not enabled it will not have an effect on the
    /// display.
    #[inline(always)]
    pub fn enable(&mut self) -> &mut Self {
        self.set_bit(7, true);

        self
    }

    /// Disables the window, must call [Windows::commit] for this change to be
    /// seen.
    #[inline(always)]
    pub fn disable(&mut self) -> &mut Self {
        self.set_bit(7, false);

        self
    }

    fn is_enabled(&self) -> bool {
        (self.window_bits >> 7) != 0
    }

    fn set_bit(&mut self, bit: usize, value: bool) {
        self.window_bits &= u8::MAX ^ (1 << bit);
        self.window_bits |= (value as u8) << bit;
    }

    /// Resets the window to it's default state, must call [Windows::commit] for
    /// this change to be seen. The default state is the window disabled with
    /// nothing rendered.
    #[inline(always)]
    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new();

        self
    }
    /// Sets whether the blend is enabled inside of this window, must call
    /// [Windows::commit] for this change to be seen.
    #[inline(always)]
    pub fn set_blend_enable(&mut self, blend: bool) -> &mut Self {
        self.set_bit(5, blend);

        self
    }
    /// Sets whether the given background will be rendered inside this window,
    /// must call [Windows::commit] for this change to be seen.
    #[inline(always)]
    pub fn set_background_enable(&mut self, back: BackgroundID, enable: bool) -> &mut Self {
        self.set_bit(back.0 as usize, enable);

        self
    }
    /// Sets whether objects will be rendered inside this window, must call
    /// [Windows::commit] for this change to be seen.
    #[inline(always)]
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

    /// Enables the window, must call [Windows::commit] for this change to be
    /// seen. If a window is not enabled it will not have an effect on the
    /// display.
    #[inline(always)]
    pub fn enable(&mut self) -> &mut Self {
        self.inner.enable();

        self
    }

    /// Disables the window, must call [Windows::commit] for this change to be
    /// seen.
    #[inline(always)]
    pub fn disable(&mut self) -> &mut Self {
        self.inner.disable();

        self
    }

    fn is_enabled(&self) -> bool {
        self.inner.is_enabled()
    }

    /// Resets the window to it's default state, must call [Windows::commit] for
    /// this change to be seen. The default state is the window disabled with
    /// nothing rendered and represents a 0x0 rectangle at (0, 0).
    #[inline(always)]
    pub fn reset(&mut self) -> &mut Self {
        *self = Self::new();

        self
    }
    /// Sets whether the blend is enabled inside of this window, must call
    /// [Windows::commit] for this change to be seen.
    #[inline(always)]
    pub fn set_blend_enable(&mut self, blend: bool) -> &mut Self {
        self.inner.set_blend_enable(blend);
        self
    }
    /// Sets whether the given background will be rendered inside this window,
    /// must call [Windows::commit] for this change to be seen.
    #[inline(always)]
    pub fn set_background_enable(&mut self, back: BackgroundID, enable: bool) -> &mut Self {
        self.inner.set_background_enable(back, enable);
        self
    }
    /// Sets whether objects will be rendered inside this window, must call
    /// [Windows::commit] for this change to be seen.
    #[inline(always)]
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

    /// Sets the area of what is inside the window using [u8] representation,
    /// which is closest to what the GBA uses. Most of the time
    /// [MovableWindow::set_position] should be used.
    #[inline(always)]
    pub fn set_position_u8(&mut self, rect: Rect<u8>) -> &mut Self {
        self.rect = rect;

        self
    }

    /// Sets the position of the area that is inside the window.
    #[inline(always)]
    pub fn set_position(&mut self, rect: &Rect<i32>) -> &mut Self {
        let new_rect = Rect::new(
            (
                rect.position.x.clamp(0, WIDTH) as u8,
                rect.position.y.clamp(0, HEIGHT) as u8,
            )
                .into(),
            (rect.size.x as u8, rect.size.y as u8).into(),
        );
        self.set_position_u8(new_rect)
    }
}
