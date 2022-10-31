use crate::fixnum::Vector2D;
use bitflags::bitflags;
use core::convert::From;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Tri {
    Positive = 1,
    Zero = 0,
    Negative = -1,
}

impl From<(bool, bool)> for Tri {
    fn from(a: (bool, bool)) -> Tri {
        let b1 = i8::from(a.0);
        let b2 = i8::from(a.1);
        unsafe { core::mem::transmute(b2 - b1) }
    }
}

bitflags! {
    pub struct Button: u32 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const RIGHT = 1 << 4;
        const LEFT = 1 << 5;
        const UP = 1 << 6;
        const DOWN = 1 << 7;
        const R = 1 << 8;
        const L = 1 << 9;
    }
}

const BUTTON_INPUT: *mut u16 = (0x04000130) as *mut u16;

// const BUTTON_INTURRUPT: *mut u16 = (0x04000132) as *mut u16;

pub struct ButtonController {
    previous: u16,
    current: u16,
}

impl Default for ButtonController {
    fn default() -> Self {
        ButtonController::new()
    }
}

impl ButtonController {
    #[must_use]
    pub fn new() -> Self {
        let pressed = !unsafe { BUTTON_INPUT.read_volatile() };
        ButtonController {
            previous: pressed,
            current: pressed,
        }
    }

    pub fn update(&mut self) {
        self.previous = self.current;
        self.current = !unsafe { BUTTON_INPUT.read_volatile() };
    }

    #[must_use]
    pub fn x_tri(&self) -> Tri {
        let left = self.is_pressed(Button::LEFT);
        let right = self.is_pressed(Button::RIGHT);

        (left, right).into()
    }

    #[must_use]
    pub fn y_tri(&self) -> Tri {
        let up = self.is_pressed(Button::UP);
        let down = self.is_pressed(Button::DOWN);

        (up, down).into()
    }

    #[must_use]
    pub fn vector<T>(&self) -> Vector2D<T>
    where
        T: From<i32> + crate::fixnum::FixedWidthUnsignedInteger,
    {
        (self.x_tri() as i32, self.y_tri() as i32).into()
    }

    #[must_use]
    pub fn just_pressed_x_tri(&self) -> Tri {
        let left = self.is_just_pressed(Button::LEFT);
        let right = self.is_just_pressed(Button::RIGHT);

        (left, right).into()
    }

    #[must_use]
    pub fn just_pressed_y_tri(&self) -> Tri {
        let up = self.is_just_pressed(Button::UP);
        let down = self.is_just_pressed(Button::DOWN);

        (up, down).into()
    }

    #[must_use]
    pub fn just_pressed_vector<T>(&self) -> Vector2D<T>
    where
        T: From<i32> + crate::fixnum::FixedWidthUnsignedInteger,
    {
        (
            self.just_pressed_x_tri() as i32,
            self.just_pressed_y_tri() as i32,
        )
            .into()
    }

    #[must_use]
    pub fn is_pressed(&self, keys: Button) -> bool {
        let currently_pressed = u32::from(self.current);
        let keys = keys.bits();
        (currently_pressed & keys) != 0
    }

    #[must_use]
    pub fn is_released(&self, keys: Button) -> bool {
        !self.is_pressed(keys)
    }

    #[must_use]
    pub fn is_just_pressed(&self, keys: Button) -> bool {
        let current = u32::from(self.current);
        let previous = u32::from(self.previous);
        let keys = keys.bits();
        ((current & keys) != 0) && ((previous & keys) == 0)
    }

    #[must_use]
    pub fn is_just_released(&self, keys: Button) -> bool {
        let current = u32::from(self.current);
        let previous = u32::from(self.previous);
        let keys = keys.bits();
        ((current & keys) == 0) && ((previous & keys) != 0)
    }
}
