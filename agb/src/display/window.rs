#![warn(missing_docs)]
//! The window feature of the GBA.

use agb_fixnum::Vector2D;

use crate::{dma, fixnum::Rect, memory_mapped::MemoryMapped};

use super::{DISPLAY_CONTROL, HEIGHT, WIDTH, tiled::BackgroundId};

/// Access to the windows feature of the Game Boy Advance.
///
/// The windows feature can selectively display backgrounds or objects on the screen
/// and can selectively enable and disable effects. This gives out references and
/// holds changes before they can be committed.
///
/// The window for the current frame can be accessed using the
/// [`.windows()`](super::GraphicsFrame::windows()) method on the current [`GraphicsFrame`](super::GraphicsFrame).
pub struct Windows {
    wins: [MovableWindow; 2],
    out: Window,
    obj: Window,
}

const REG_HORIZONTAL_BASE: *mut u16 = 0x0400_0040 as *mut _;
const REG_VERTICAL_BASE: *mut u16 = 0x0400_0044 as *mut _;

const REG_WINDOW_CONTROL_BASE: *mut u16 = 0x0400_0048 as *mut _;

/// The two Windows that have an effect inside of them.
///
/// These are represented by [`MovableWindow`] instances and you can get those
/// by using the [`win_in()`](Windows::win_in()) method.
pub enum WinIn {
    /// The higher priority window
    Win0,
    /// The lower priority window
    Win1,
}

impl Windows {
    pub(crate) fn new() -> Self {
        Self {
            wins: [MovableWindow::new(0), MovableWindow::new(1)],
            out: Window::new(),
            obj: Window::new(),
        }
    }

    /// Returns a reference to the window that is used when outside all other windows
    #[inline(always)]
    pub fn win_out(&mut self) -> &mut Window {
        self.out.enable()
    }

    /// Gives a reference to the specified window that has effect when inside of it's boundary
    #[inline(always)]
    pub fn win_in(&mut self, id: WinIn) -> &mut MovableWindow {
        self.wins[id as usize].enable()
    }

    /// Gives a reference to the window that is controlled by objects with the
    /// [`GraphicsMode`](crate::display::object::GraphicsMode) `Window`.
    #[inline(always)]
    pub fn win_obj(&mut self) -> &mut Window {
        self.obj.enable()
    }

    pub(crate) fn commit(&self) {
        for win in &self.wins {
            win.commit();
        }
        self.out.commit(2);
        self.obj.commit(3);

        let mut display_control_register = DISPLAY_CONTROL.get();
        display_control_register.set_obj_window_display(self.obj.is_enabled());
        display_control_register.set_window0_display(self.wins[0].is_enabled());
        display_control_register.set_window1_display(self.wins[1].is_enabled());

        DISPLAY_CONTROL.set(display_control_register);
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
    id: usize,
}

impl Window {
    fn new() -> Window {
        Self { window_bits: 0 }
    }

    fn enable(&mut self) -> &mut Self {
        self.set_bit(7, true);

        self
    }

    fn is_enabled(&self) -> bool {
        (self.window_bits >> 7) != 0
    }

    fn set_bit(&mut self, bit: usize, value: bool) {
        self.window_bits &= u8::MAX ^ (1 << bit);
        self.window_bits |= (value as u8) << bit;
    }

    /// Sets whether the blend is enabled inside of this window.
    #[inline(always)]
    pub fn enable_blending(&mut self) -> &mut Self {
        self.set_bit(5, true);

        self
    }
    /// Sets whether the given background will be rendered inside this window.
    #[inline(always)]
    pub fn enable_background(&mut self, back: impl Into<BackgroundId>) -> &mut Self {
        self.set_bit(back.into().0 as usize, true);

        self
    }
    /// Sets whether objects will be rendered inside this window.
    #[inline(always)]
    pub fn enable_objects(&mut self) -> &mut Self {
        self.set_bit(4, true);

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
    fn new(id: usize) -> Self {
        Self {
            inner: Window::new(),
            rect: Rect::new((0, 0).into(), (0, 0).into()),
            id,
        }
    }

    fn enable(&mut self) -> &mut Self {
        self.inner.enable();

        self
    }

    fn is_enabled(&self) -> bool {
        self.inner.is_enabled()
    }

    /// Sets whether the blend is enabled inside of this window.
    #[inline(always)]
    pub fn enable_blending(&mut self) -> &mut Self {
        self.inner.enable_blending();
        self
    }
    /// Sets whether the given background will be rendered inside this window.
    #[inline(always)]
    pub fn enable_background(&mut self, back: impl Into<BackgroundId>) -> &mut Self {
        self.inner.enable_background(back);
        self
    }
    /// Sets whether objects will be rendered inside this window.
    #[inline(always)]
    pub fn enable_objects(&mut self) -> &mut Self {
        self.inner.enable_objects();
        self
    }

    fn commit(&self) {
        self.inner.commit(self.id);

        let left_right =
            ((self.rect.position.x as u16) << 8) | (self.rect.position.x + self.rect.size.x) as u16;

        let top_bottom =
            ((self.rect.position.y as u16) << 8) | (self.rect.position.y + self.rect.size.y) as u16;
        unsafe {
            REG_HORIZONTAL_BASE.add(self.id).write_volatile(left_right);
            REG_VERTICAL_BASE.add(self.id).write_volatile(top_bottom);
        }
    }

    #[inline(always)]
    fn set_pos_u8(&mut self, rect: Rect<u8>) -> &mut Self {
        self.rect = rect;

        self
    }

    /// Sets the position of the area that is inside the window.
    #[inline(always)]
    pub fn set_pos(&mut self, rect: Rect<i32>) -> &mut Self {
        let new_rect = Rect::new(
            (
                rect.position.x.clamp(0, WIDTH) as u8,
                rect.position.y.clamp(0, HEIGHT) as u8,
            )
                .into(),
            (rect.size.x as u8, rect.size.y as u8).into(),
        );
        self.set_pos_u8(new_rect)
    }

    /// DMA to control the horizontal position of the window.
    ///
    /// The [`Vector2D`] returned here isn't an `x` and `y` component but instead represents the
    /// left and right hand sides of the window.
    ///
    /// When you use this, you should also set the height of the window appropriately using
    /// [`set_pos`](Self::set_pos).
    #[must_use]
    pub fn horizontal_pos_dma(&self) -> dma::DmaControllable<Vector2D<u8>> {
        unsafe { dma::DmaControllable::new(REG_HORIZONTAL_BASE.add(self.id).cast()) }
    }
}
