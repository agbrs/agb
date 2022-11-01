#![deny(missing_docs)]
use crate::fixnum::Vector2D;
use bitflags::bitflags;
use core::convert::From;

/// Tri-state enum. Allows for -1, 0 and +1.
/// Useful if checking if the D-Pad is pointing left, right, or unpressed.
///
/// Note that [Tri] can be converted directly to a signed integer, so can easily be used to update positions of things in games
///
/// # Examples
/// ```rust,no_run
/// # #![no_std]
/// use agb::input::Tri;
///
/// # fn main() {
/// let x = 5;
/// let tri = Tri::Positive; // e.g. from button_controller.x_tri()
///
/// assert_eq!(x + tri as i32, 6);
/// # }
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Tri {
    /// Right or down
    Positive = 1,
    /// Unpressed
    Zero = 0,
    /// Left or up
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
    /// Represents a button on the GBA
    pub struct Button: u32 {
        /// The A button
        const A = 1 << 0;
        /// The B button
        const B = 1 << 1;
        /// The SELECT button
        const SELECT = 1 << 2;
        /// The START button
        const START = 1 << 3;
        /// The RIGHT button on the D-Pad
        const RIGHT = 1 << 4;
        /// The LEFT button on the D-Pad
        const LEFT = 1 << 5;
        /// The UP button on the D-Pad
        const UP = 1 << 6;
        /// The DOWN button on the D-Pad
        const DOWN = 1 << 7;
        /// The R button on the D-Pad
        const R = 1 << 8;
        /// The L button on the D-Pad
        const L = 1 << 9;
    }
}

const BUTTON_INPUT: *mut u16 = (0x04000130) as *mut u16;

// const BUTTON_INTURRUPT: *mut u16 = (0x04000132) as *mut u16;

/// Helper to make it easy to get the current state of the GBA's buttons.
///
/// # Example
///
/// ```rust,no_run
/// # #![no_std]
/// use agb::input::{ButtonController, Tri};
///
/// # fn main() {
/// let mut input = ButtonController::new();
///
/// loop {
///     input.update(); // call update every loop
///
///     match input.x_tri() {
///         Tri::Negative => { /* left is being pressed */ }
///         Tri::Positive => { /* right is being pressed */ }
///         Tri::Zero => { /* Neither left nor right (or both) are pressed */ }
///     }
/// }
/// # }
/// ```
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
    /// Create a new ButtonController.
    /// This is the preferred way to create it.
    #[must_use]
    pub fn new() -> Self {
        let pressed = !unsafe { BUTTON_INPUT.read_volatile() };
        ButtonController {
            previous: pressed,
            current: pressed,
        }
    }

    /// Updates the state of the button controller.
    /// You should call this every frame (either at the start or the end) to ensure that you have the latest state of each button press.
    /// Calls to any method won't change until you call this.
    pub fn update(&mut self) {
        self.previous = self.current;
        self.current = !unsafe { BUTTON_INPUT.read_volatile() };
    }

    /// Returns [Tri::Positive] if right is pressed, [Tri::Negative] if left is pressed and [Tri::Zero] if neither or both are pressed.
    /// This is the normal behaviour you'll want if you're using orthogonal inputs.
    #[must_use]
    pub fn x_tri(&self) -> Tri {
        let left = self.is_pressed(Button::LEFT);
        let right = self.is_pressed(Button::RIGHT);

        (left, right).into()
    }

    /// Returns [Tri::Positive] if down is pressed, [Tri::Negative] if up is pressed and [Tri::Zero] if neither or both are pressed.
    /// This is the normal behaviour you'll want if you're using orthogonal inputs.
    #[must_use]
    pub fn y_tri(&self) -> Tri {
        let up = self.is_pressed(Button::UP);
        let down = self.is_pressed(Button::DOWN);

        (up, down).into()
    }

    /// Returns true if all the buttons specified in `keys` are pressed.
    #[must_use]
    pub fn vector<T>(&self) -> Vector2D<T>
    where
        T: From<i32> + crate::fixnum::FixedWidthUnsignedInteger,
    {
        (self.x_tri() as i32, self.y_tri() as i32).into()
    }

    #[must_use]
    /// Returns [Tri::Positive] if left was just pressed, [Tri::Negative] if right was just pressed and [Tri::Zero] if neither or both are just pressed.
    ///
    /// Also returns [Tri::Zero] after the call to [`update()`](ButtonController::update()) if the button is still held.
    pub fn just_pressed_x_tri(&self) -> Tri {
        let left = self.is_just_pressed(Button::LEFT);
        let right = self.is_just_pressed(Button::RIGHT);

        (left, right).into()
    }

    #[must_use]
    /// Returns [Tri::Positive] if down was just pressed, [Tri::Negative] if up was just pressed and [Tri::Zero] if neither or both are just pressed.
    ///
    /// Also returns [Tri::Zero] after the call to [`update()`](ButtonController::update()) if the button is still held.
    pub fn just_pressed_y_tri(&self) -> Tri {
        let up = self.is_just_pressed(Button::UP);
        let down = self.is_just_pressed(Button::DOWN);

        (up, down).into()
    }

    #[must_use]
    /// Returns a vector which represents the direction the button was just pressed in.
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
    /// Returns `true` if the provided keys are all pressed, and `false` if not.
    pub fn is_pressed(&self, keys: Button) -> bool {
        let currently_pressed = u32::from(self.current);
        let keys = keys.bits();
        (currently_pressed & keys) != 0
    }

    /// Returns true if all the buttons specified in `keys` are not pressed. Equivalent to `!is_pressed(keys)`.
    #[must_use]
    pub fn is_released(&self, keys: Button) -> bool {
        !self.is_pressed(keys)
    }

    /// Returns true if all the buttons specified in `keys` went from not pressed to pressed in the last frame.
    /// Very useful for menu navigation or selection if you want the players actions to only happen for one frame.
    ///
    /// # Example
    /// ```no_run,rust
    /// # #![no_std]
    /// use agb::input::{Button, ButtonController};
    ///
    /// # fn main() {
    /// let mut button_controller = ButtonController::new();
    ///
    /// loop {
    ///     button_controller.update();
    ///
    ///     if button_controller.is_just_pressed(Button::A) {
    ///         // A button was just pressed, maybe select the currently selected item
    ///     }
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_just_pressed(&self, keys: Button) -> bool {
        let current = u32::from(self.current);
        let previous = u32::from(self.previous);
        let keys = keys.bits();
        ((current & keys) != 0) && ((previous & keys) == 0)
    }

    /// Returns true if all the buttons specified in `keys` went from pressed to not pressed in the last frame.
    /// Very useful for menu navigation or selection if you want players actions to only happen for one frame.
    #[must_use]
    pub fn is_just_released(&self, keys: Button) -> bool {
        let current = u32::from(self.current);
        let previous = u32::from(self.previous);
        let keys = keys.bits();
        ((current & keys) == 0) && ((previous & keys) != 0)
    }
}
