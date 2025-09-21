//! Async input system with per-button interrupt handling
//!
//! ## Design Decisions:
//!
//! 1. **VBlank-based detection**: Uses VBlank interrupt (60Hz) instead of keypad interrupt
//!    - Reason: GBA keypad interrupt is level-triggered (fires continuously while pressed)
//!    - Embassy pattern: Hardware-driven timing with clean edge detection
//!
//! 2. **Per-button wakers**: Each button has its own AtomicWaker in static array
//!    - Reason: Only wake futures waiting for specific buttons that changed
//!    - Embassy pattern: Targeted waking, not broadcast waking
//!
//! 3. **16ms max latency**: VBlank at 60Hz provides excellent game input responsiveness
//!    - Reason: Much simpler than complex keypad interrupt disable/re-enable logic
//!    - Trade-off: Slight latency for major simplicity and reliability gains

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use portable_atomic::Ordering;

use agb::input::{Button, ButtonController, Tri};
use agb::interrupt::{add_interrupt_handler, Interrupt};
use embassy_sync::waitqueue::AtomicWaker;

/// Keypad input register (KEYINPUT) at 0x04000130  
const KEYPAD_INPUT: *mut u16 = 0x04000130 as *mut u16;

const BUTTON_COUNT: usize = 10;
/// Per-button wakers - following Embassy's pattern
static BUTTON_WAKERS: [AtomicWaker; BUTTON_COUNT] = [const { AtomicWaker::new() }; BUTTON_COUNT];

/// Global button state for VBlank monitoring
static GLOBAL_BUTTON_STATE: portable_atomic::AtomicU16 = portable_atomic::AtomicU16::new(0);

/// Whether the VBlank handler is initialized
static INITIALIZED: portable_atomic::AtomicBool = portable_atomic::AtomicBool::new(false);

/// Convert Button to array index
fn button_to_index(button: Button) -> Option<usize> {
    match button {
        Button::A => Some(0),
        Button::B => Some(1),
        Button::SELECT => Some(2),
        Button::START => Some(3),
        Button::RIGHT => Some(4),
        Button::LEFT => Some(5),
        Button::UP => Some(6),
        Button::DOWN => Some(7),
        Button::R => Some(8),
        Button::L => Some(9),
        _ => None,
    }
}

/// Initialize the VBlank-based input system
fn init_input_system() {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return; // Already initialized
    }

    // Set up VBlank handler for all button change detection
    let vblank_handler = unsafe {
        add_interrupt_handler(Interrupt::VBlank, |_| {
            on_vblank_input_check();
        })
    };
    core::mem::forget(vblank_handler);

    // Initialize global state
    let current = !unsafe { KEYPAD_INPUT.read_volatile() };
    GLOBAL_BUTTON_STATE.store(current, Ordering::SeqCst);
}

/// VBlank handler - detect all button changes at 60Hz
fn on_vblank_input_check() {
    let current = !unsafe { KEYPAD_INPUT.read_volatile() };
    let previous = GLOBAL_BUTTON_STATE.load(Ordering::SeqCst);

    if current != previous {
        // Find which buttons changed and wake only those wakers
        let changed = current ^ previous;
        let buttons = [
            Button::A,
            Button::B,
            Button::SELECT,
            Button::START,
            Button::RIGHT,
            Button::LEFT,
            Button::UP,
            Button::DOWN,
            Button::R,
            Button::L,
        ];

        for (i, button) in buttons.iter().enumerate() {
            let button_mask = button.bits() as u16;
            if (changed & button_mask) != 0 {
                // Only wake the waker for this specific button
                BUTTON_WAKERS[i].wake();
            }
        }

        // Update global state after waking relevant futures
        GLOBAL_BUTTON_STATE.store(current, Ordering::SeqCst);
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

/// Future that waits for a specific button event
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct ButtonEventFuture {
    button: Button,
    waiting_for_press: bool,
    completed: bool,
}

impl ButtonEventFuture {
    fn new(button: Button, waiting_for_press: bool) -> Self {
        init_input_system();

        Self {
            button,
            waiting_for_press,
            completed: false,
        }
    }
}

impl Future for ButtonEventFuture {
    type Output = ButtonEvent;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completed {
            return Poll::Ready(if self.waiting_for_press {
                ButtonEvent::Pressed
            } else {
                ButtonEvent::Released
            });
        }

        if let Some(index) = button_to_index(self.button) {
            BUTTON_WAKERS[index].register(cx.waker());

            // Check current state
            let current = !unsafe { KEYPAD_INPUT.read_volatile() };
            let is_pressed = (current & self.button.bits() as u16) != 0;

            if self.waiting_for_press && is_pressed {
                self.completed = true;
                Poll::Ready(ButtonEvent::Pressed)
            } else if !self.waiting_for_press && !is_pressed {
                self.completed = true;
                Poll::Ready(ButtonEvent::Released)
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(if self.waiting_for_press {
                ButtonEvent::Pressed
            } else {
                ButtonEvent::Released
            })
        }
    }
}

/// Future that waits for any button event
#[must_use = "futures do nothing unless you `.await` or poll them"]
struct AnyButtonEventFuture {
    last_state: u16,
}

impl AnyButtonEventFuture {
    fn new() -> Self {
        init_input_system();

        let current = !unsafe { KEYPAD_INPUT.read_volatile() };
        Self {
            last_state: current,
        }
    }
}

impl Future for AnyButtonEventFuture {
    type Output = (Button, ButtonEvent);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Register with all button wakers
        for waker in &BUTTON_WAKERS {
            waker.register(cx.waker());
        }

        // Check current state
        let current = !unsafe { KEYPAD_INPUT.read_volatile() };
        let changed = current ^ self.last_state;

        if changed != 0 {
            // Find which button changed
            let buttons = [
                Button::A,
                Button::B,
                Button::SELECT,
                Button::START,
                Button::RIGHT,
                Button::LEFT,
                Button::UP,
                Button::DOWN,
                Button::R,
                Button::L,
            ];

            for button in buttons.iter() {
                let button_mask = button.bits() as u16;
                if (changed & button_mask) != 0 {
                    let is_pressed = (current & button_mask) != 0;

                    // Update only the specific bit that we're handling
                    if is_pressed {
                        self.last_state |= button_mask;
                    } else {
                        self.last_state &= !button_mask;
                    }

                    return Poll::Ready((
                        *button,
                        if is_pressed {
                            ButtonEvent::Pressed
                        } else {
                            ButtonEvent::Released
                        },
                    ));
                }
            }
        }

        Poll::Pending
    }
}

/// Async wrapper for agb input operations
pub struct AsyncInput {
    controller: ButtonController,
}

impl AsyncInput {
    pub(crate) fn new() -> Self {
        init_input_system();

        Self {
            controller: ButtonController::new(),
        }
    }

    /// Wait for a specific button to be pressed
    pub async fn wait_for_button_press(&mut self, button: Button) -> ButtonEvent {
        // If button is already pressed, wait for release first
        let current = !unsafe { KEYPAD_INPUT.read_volatile() };
        let is_pressed = (current & button.bits() as u16) != 0;

        if is_pressed {
            // Wait for release first
            ButtonEventFuture::new(button, false).await;
        }

        // Now wait for press
        ButtonEventFuture::new(button, true).await
    }

    /// Wait for any button to be pressed or released
    pub async fn wait_for_any_button_press(&mut self) -> (Button, ButtonEvent) {
        AnyButtonEventFuture::new().await
    }

    /// Wait for a specific button to be pressed (polling-based, for compatibility)
    pub async fn wait_for_button_press_polling(&mut self, button: Button) -> ButtonEvent {
        ButtonPressFuture::new(&mut self.controller, button).await
    }

    /// Wait for any button to be pressed (polling-based, for compatibility)
    pub async fn wait_for_any_button_press_polling(&mut self) -> (Button, ButtonEvent) {
        AnyButtonPressFuture::new(&mut self.controller).await
    }

    /// Get current button state (non-blocking)
    pub fn update(&mut self) {
        self.controller.update();
    }

    /// Check if a button is currently pressed
    pub fn is_pressed(&self, button: Button) -> bool {
        let current = !unsafe { KEYPAD_INPUT.read_volatile() };
        (current & button.bits() as u16) != 0
    }

    /// Check if a button is currently pressed (polling-based, for compatibility)
    pub fn is_pressed_polling(&self, button: Button) -> bool {
        self.controller.is_pressed(button)
    }

    /// Check if a button was just pressed this frame (polling-based, for compatibility)
    pub fn is_just_pressed_polling(&self, button: Button) -> bool {
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

/// Future that waits for a specific button press (polling-based)
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

/// Future that waits for any button press (polling-based)
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
