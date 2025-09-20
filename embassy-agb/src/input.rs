use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use agb::input::{Button, ButtonController, Tri};

/// Async wrapper for agb input operations
pub struct AsyncInput {
    controller: ButtonController,
}

impl AsyncInput {
    pub(crate) fn new() -> Self {
        Self {
            controller: ButtonController::new(),
        }
    }

    /// Wait for a specific button to be pressed
    pub async fn wait_for_button_press(&mut self, button: Button) -> ButtonEvent {
        ButtonPressFuture::new(&mut self.controller, button).await
    }

    /// Wait for any button to be pressed
    pub async fn wait_for_any_button_press(&mut self) -> (Button, ButtonEvent) {
        AnyButtonPressFuture::new(&mut self.controller).await
    }

    /// Get current button state (non-blocking)
    pub fn update(&mut self) {
        self.controller.update();
    }

    /// Check if a button is currently pressed (non-blocking)
    pub fn is_pressed(&self, button: Button) -> bool {
        self.controller.is_pressed(button)
    }

    /// Check if a button was just pressed this frame (non-blocking)
    pub fn is_just_pressed(&self, button: Button) -> bool {
        self.controller.is_just_pressed(button)
    }

    /// Get the tri-state for directional inputs (non-blocking)
    pub fn x_tri(&self) -> Tri {
        self.controller.x_tri()
    }

    /// Get the tri-state for directional inputs (non-blocking)
    pub fn y_tri(&self) -> Tri {
        self.controller.y_tri()
    }
}

/// Button event types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    /// Button was just pressed
    Pressed,
    /// Button was just released
    Released,
}

/// Future that waits for a specific button press
struct ButtonPressFuture<'a> {
    controller: &'a mut ButtonController,
    button: Button,
    waiting_for_release: bool,
}

impl<'a> ButtonPressFuture<'a> {
    fn new(controller: &'a mut ButtonController, button: Button) -> Self {
        let waiting_for_release = controller.is_pressed(button);
        Self {
            controller,
            button,
            waiting_for_release,
        }
    }
}

impl<'a> Future for ButtonPressFuture<'a> {
    type Output = ButtonEvent;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.controller.update();

        let is_pressed = self.controller.is_pressed(self.button);

        if self.waiting_for_release {
            if !is_pressed {
                self.waiting_for_release = false;
                return Poll::Ready(ButtonEvent::Released);
            }
        } else if is_pressed {
            return Poll::Ready(ButtonEvent::Pressed);
        }

        // Not ready yet, wake on next frame
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

/// Future that waits for any button press
struct AnyButtonPressFuture<'a> {
    controller: &'a mut ButtonController,
}

impl<'a> AnyButtonPressFuture<'a> {
    fn new(controller: &'a mut ButtonController) -> Self {
        Self { controller }
    }
}

impl<'a> Future for AnyButtonPressFuture<'a> {
    type Output = (Button, ButtonEvent);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.controller.update();

        // Check all buttons for press events
        let buttons = [
            Button::A,
            Button::B,
            Button::START,
            Button::SELECT,
            Button::LEFT,
            Button::RIGHT,
            Button::UP,
            Button::DOWN,
            Button::L,
            Button::R,
        ];

        for &button in &buttons {
            if self.controller.is_just_pressed(button) {
                return Poll::Ready((button, ButtonEvent::Pressed));
            }
            if self.controller.is_just_released(button) {
                return Poll::Ready((button, ButtonEvent::Released));
            }
        }

        // No button events, wake on next frame
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
