use core::ops::BitOr;

use crate::fixnum::Vector2D;

/// Tri-state enum. Allows for -1, 0 and +1.
/// Useful if checking if the D-Pad is pointing left, right, or unpressed.
///
/// Note that [Tri] can be converted directly to a signed integer, so can easily be used to update positions of things in games
///
/// # Examples
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// use agb::input::Tri;
///
/// # #[agb::doctest]
/// # fn test(_: agb::Gba) {
/// let x = 5;
/// let tri = Tri::Positive; // e.g. from button_controller.x_tri()
///
/// assert_eq!(x + tri as i32, 6);
/// # }
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
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

/// Represents a button on the GBA
///
/// ```rust
/// # #![no_main]
/// # #![no_std]
/// # #[agb::doctest]
/// # fn test(_gba: agb::Gba) {
/// # use agb::input::{Button, ButtonController};
/// # let mut button_controller = ButtonController::new();
/// // Check if A is pressed
/// if button_controller.is_pressed(Button::A) {
///     // ...
/// }
/// # }
/// ```
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
#[repr(u16)]
pub enum Button {
    /// The A button
    A = 1 << 0,
    /// The B button
    B = 1 << 1,
    /// The SELECT button
    SELECT = 1 << 2,
    /// The START button
    START = 1 << 3,
    /// The RIGHT button on the D-Pad
    RIGHT = 1 << 4,
    /// The LEFT button on the D-Pad
    LEFT = 1 << 5,
    /// The UP button on the D-Pad
    UP = 1 << 6,
    /// The DOWN button on the D-Pad
    DOWN = 1 << 7,
    /// The R shoulder button on the D-Pad
    R = 1 << 8,
    /// The L shoulder button on the D-Pad
    L = 1 << 9,
}

const BUTTON_INPUT: *mut u16 = (0x04000130) as *mut u16;

// const BUTTON_INTERRUPT: *mut u16 = (0x04000132) as *mut u16;

/// Helper to make it easy to get the current state of the GBA's buttons.
///
/// # Example
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// use agb::input::{ButtonController, Tri};
///
/// # #[agb::doctest]
/// # fn test(_gba: agb::Gba) {
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
/// #   break;
/// }
/// # }
/// ```
pub struct ButtonController {
    previous: ButtonState,
    current: ButtonState,
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
        let pressed = ButtonState::current();
        ButtonController {
            previous: pressed,
            current: pressed,
        }
    }

    /// Updates the state of the button controller.
    /// You should call this every frame (either at the start or the end) to ensure that you have the latest state of each button press.
    /// Calls to any method won't change until you call this.
    pub fn update(&mut self) {
        self.update_with_state(ButtonState::current());
    }

    /// Updates the state of the button controller with a given input.
    /// This is mainly useful for unit tests where you want to control what input is set. Is equivalent to
    /// `update()` assuming that the given buttons in `state` are the new ones being pressed
    pub fn update_with_state(&mut self, state: impl Into<ButtonState>) {
        self.previous = self.current;
        self.current = state.into();
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

    /// Returns [Tri::Positive] if R is pressed, [Tri::Negative] if L is pressed and [Tri::Zero] if neither or both are pressed.
    #[must_use]
    pub fn lr_tri(&self) -> Tri {
        let l = self.is_pressed(Button::L);
        let r = self.is_pressed(Button::R);

        (l, r).into()
    }

    /// Returns a vector which represents the current direction being pressed.
    ///
    /// ```rust
    /// # #![no_main]
    /// # #![no_std]
    /// # #[agb::doctest]
    /// # fn test(_gba: agb::Gba) {
    /// use agb::{
    ///     input::ButtonController,
    ///     fixnum::{Vector2D, Num, vec2, num},
    /// };
    ///
    /// let mut player_position: Vector2D<Num<i32, 8>> = vec2(num!(10), num!(20));
    /// let mut button_controller = ButtonController::new();
    ///
    /// loop {
    ///     button_controller.update();
    ///
    ///     player_position += button_controller.vector();
    ///     # break;
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn vector<T>(&self) -> Vector2D<T>
    where
        T: From<i32> + crate::fixnum::Number,
    {
        (self.x_tri() as i32, self.y_tri() as i32).into()
    }

    #[must_use]
    /// Returns [Tri::Positive] if right was just pressed, [Tri::Negative] if left was just pressed and [Tri::Zero] if neither or both are just pressed.
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
    /// Returns [Tri::Positive] if `R` was just pressed, [Tri::Negative] if `L` was just pressed and [Tri::Zero] if neither or both are just pressed.
    ///
    /// Also returns [Tri::Zero] after the call to [`update()`](ButtonController::update()) if the button is still held.
    pub fn just_pressed_lr_tri(&self) -> Tri {
        let l = self.is_just_pressed(Button::L);
        let r = self.is_just_pressed(Button::R);

        (l, r).into()
    }

    #[must_use]
    /// Returns a vector which represents the direction the button was just pressed in.
    pub fn just_pressed_vector<T>(&self) -> Vector2D<T>
    where
        T: From<i32> + crate::fixnum::Number,
    {
        (
            self.just_pressed_x_tri() as i32,
            self.just_pressed_y_tri() as i32,
        )
            .into()
    }

    #[must_use]
    /// Returns `true` if any of the provided buttons are pressed.
    pub fn is_pressed(&self, buttons: impl Into<ButtonState>) -> bool {
        self.current.any_pressed(buttons.into())
    }

    /// Returns `true` if any of the provided buttons are not pressed.
    #[must_use]
    pub fn is_released(&self, buttons: impl Into<ButtonState>) -> bool {
        !self.current.all_pressed(buttons.into())
    }

    /// Returns true the button specified in `button` went from not pressed to pressed in the last frame.
    /// Very useful for menu navigation or selection if you want the players actions to only happen for one frame.
    ///
    /// If you pass multiple buttons (via [`ButtonState`]), then this will return true if _any_ of the provided
    /// buttons transitioned from not pressed to pressed
    ///
    /// # Example
    /// ```rust
    /// # #![no_std]
    /// # #![no_main]
    /// use agb::input::{Button, ButtonController};
    ///
    /// # #[agb::doctest]
    /// # fn main(_gba: agb::Gba) {
    /// let mut button_controller = ButtonController::new();
    ///
    /// loop {
    ///     button_controller.update();
    ///
    ///     if button_controller.is_just_pressed(Button::A) {
    ///         // A button was just pressed, maybe select the currently selected item
    ///     }
    ///     # break;
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn is_just_pressed(&self, buttons: impl Into<ButtonState>) -> bool {
        let buttons = buttons.into();
        ButtonState(self.current.0 & !self.previous.0).any_pressed(buttons)
    }

    /// Returns true if the button specified in `key` went from pressed to not pressed in the last frame.
    /// Very useful for menu navigation or selection if you want players actions to only happen for one frame.
    ///
    /// If you pass multiple buttons (via [`ButtonState`]), then this will return true if _any_ of the provided
    /// buttons transitioned from pressed to not pressed.
    #[must_use]
    pub fn is_just_released(&self, buttons: impl Into<ButtonState>) -> bool {
        let buttons = buttons.into();
        ButtonState(!self.current.0 & self.previous.0).any_pressed(buttons)
    }
}

/// Represents the state of potentially multiple buttons being pressed at once
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ButtonState(u16);

impl From<Button> for ButtonState {
    fn from(value: Button) -> Self {
        Self::single(value)
    }
}

impl BitOr for Button {
    type Output = ButtonState;

    fn bitor(self, rhs: Self) -> Self::Output {
        ButtonState(self as u16 | rhs as u16)
    }
}

impl BitOr for ButtonState {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl ButtonState {
    /// Creates a new ButtonState with just a single button pressed (equivalent to `ButtonState::from(...)`)
    #[must_use]
    pub const fn single(button: Button) -> Self {
        Self(button as u16)
    }

    /// Returns the current button state based on which buttons are being pressed right now
    #[must_use]
    pub fn current() -> Self {
        Self(!unsafe { BUTTON_INPUT.read_volatile() })
    }

    /// Returns a `ButtonState` where everything is being pressed
    #[must_use]
    pub const fn all() -> Self {
        Self(0b0000_0011_1111_1111)
    }

    /// Returns a `ButtonState` where nothing is being pressed
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Returns true if the button `button` is pressed in this state
    #[must_use]
    pub const fn is_pressed(self, button: Button) -> bool {
        self.any_pressed(Self::single(button))
    }

    /// Returns true if the button `button` is released in this state
    #[must_use]
    pub const fn is_released(self, button: Button) -> bool {
        !self.is_pressed(button)
    }

    /// Returns true if any of the buttons in the button state `state` are pressed in self
    #[must_use]
    pub const fn any_pressed(self, state: ButtonState) -> bool {
        self.0 & state.0 != 0
    }

    /// Returns true if all of the buttons in the button state `state` are pressed in self
    #[must_use]
    pub const fn all_pressed(self, state: ButtonState) -> bool {
        self.0 & state.0 == state.0
    }
}

#[cfg(test)]
mod test {
    use crate::Gba;

    use super::*;

    #[test_case]
    fn test_tri_bool_tuple_from_impl(_gba: &mut Gba) {
        assert_eq!(Tri::from((true, false)), Tri::Negative);
        assert_eq!(Tri::from((false, true)), Tri::Positive);
        assert_eq!(Tri::from((false, false)), Tri::Zero);
        assert_eq!(Tri::from((true, true)), Tri::Zero);
    }

    #[test_case]
    fn test_button_state_is_pressed(_: &mut Gba) {
        assert!(ButtonState::from(Button::A).is_pressed(Button::A));
        assert!((Button::A | Button::B).is_pressed(Button::A));
        assert!(!(Button::A | Button::B).is_pressed(Button::START));
    }

    #[test_case]
    fn test_button_state_is_released(_: &mut Gba) {
        assert!(ButtonState::from(Button::A).is_released(Button::B));
        assert!(!ButtonState::from(Button::B).is_released(Button::B));
    }

    #[test_case]
    fn test_button_controller_is_just_pressed(_: &mut Gba) {
        let mut controller = ButtonController::new();

        controller.update_with_state(Button::B);
        controller.update_with_state(Button::A);

        assert!(controller.is_just_pressed(Button::A));
        assert!(controller.is_just_released(Button::B));
        assert!(!controller.is_just_pressed(Button::START));
        assert!(!controller.is_just_released(Button::SELECT));
    }

    #[test_case]
    fn test_button_controller_tri(_: &mut Gba) {
        let mut controller = ButtonController::new();

        controller.update_with_state(Button::L | Button::RIGHT);

        assert_eq!(controller.lr_tri(), Tri::Negative);
        assert_eq!(controller.x_tri(), Tri::Positive);
        assert_eq!(controller.y_tri(), Tri::Zero);
    }

    #[test_case]
    fn test_button_state_all(_: &mut Gba) {
        assert!(ButtonState::all().is_pressed(Button::A));
        assert!(ButtonState::all().is_pressed(Button::L));
    }

    #[test_case]
    fn test_just_pressed_multiple(_: &mut Gba) {
        let mut controller = ButtonController::new();

        controller.update_with_state(ButtonState::empty());
        controller.update_with_state(Button::A | Button::B);

        assert!(controller.is_just_pressed(Button::A | Button::START));

        controller.update_with_state(Button::A);

        assert!(!controller.is_just_pressed(Button::A | Button::START));
        assert!(controller.is_just_released(Button::B | Button::SELECT));
    }
}
