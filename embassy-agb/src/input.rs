//! Async input system with configurable timer-based polling
//!
//! ## Design Decisions:
//!
//! 1. **Timer-based polling**: Uses Embassy timer instead of VBlank interrupt
//!    - Reason: Avoids VBlank interrupt conflicts with display system
//!    - Benefit: Configurable poll rate for different game requirements
//!    - Default: 60Hz polling (16.67ms) to match VBlank rate
//!
//! 2. **Per-button wakers**: Each button has its own AtomicWaker in static array
//!    - Reason: Only wake futures waiting for specific buttons that changed
//!    - Embassy pattern: Targeted waking, not broadcast waking
//!
//! 3. **Configurable latency**: Poll rate can be adjusted from 30Hz to 120Hz
//!    - Higher rates: Lower latency but more CPU usage
//!    - Lower rates: Higher latency but better power efficiency
//!    - Note: Button presses may not register until next poll cycle

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use portable_atomic::Ordering;

use agb::input::{Button, ButtonController, Tri};
use embassy_sync::waitqueue::AtomicWaker;

#[cfg(feature = "time")]
use embassy_time;

#[cfg(feature = "executor")]
use embassy_executor;

/// Keypad input register (KEYINPUT) at 0x04000130  
const KEYPAD_INPUT: *mut u16 = 0x04000130 as *mut u16;

const BUTTON_COUNT: usize = 10;
/// Per-button wakers - following Embassy's pattern
static BUTTON_WAKERS: [AtomicWaker; BUTTON_COUNT] = [const { AtomicWaker::new() }; BUTTON_COUNT];

/// Global button state for timer-based monitoring
static GLOBAL_BUTTON_STATE: portable_atomic::AtomicU16 = portable_atomic::AtomicU16::new(0);

/// Whether the input polling task is running
static POLLING_TASK_RUNNING: portable_atomic::AtomicBool = portable_atomic::AtomicBool::new(false);

/// Input polling rate options
#[derive(Debug, Clone, Copy)]
pub enum PollingRate {
    /// 30Hz - Lower latency, more power efficient
    Hz30,
    /// 60Hz - Default, matches VBlank rate
    Hz60,
    /// 90Hz - Higher responsiveness
    Hz90,
    /// 120Hz - Highest responsiveness, more CPU usage
    Hz120,
    /// Custom rate in Hz (clamped to 10-240 range)
    Custom(u32),
}

impl PollingRate {
    /// Get the polling rate as Hz value
    pub fn as_hz(self) -> u32 {
        match self {
            PollingRate::Hz30 => 30,
            PollingRate::Hz60 => 60,
            PollingRate::Hz90 => 90,
            PollingRate::Hz120 => 120,
            PollingRate::Custom(hz) => hz.clamp(10, 240),
        }
    }
}

impl Default for PollingRate {
    fn default() -> Self {
        PollingRate::Hz60
    }
}

/// Input polling configuration
#[derive(Debug, Clone, Copy)]
pub struct InputConfig {
    /// Polling rate
    pub poll_rate: PollingRate,
}

impl InputConfig {
    /// Create config with specific polling rate
    pub fn new(poll_rate: PollingRate) -> Self {
        Self { poll_rate }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            poll_rate: PollingRate::default(),
        }
    }
}

impl From<PollingRate> for InputConfig {
    fn from(poll_rate: PollingRate) -> Self {
        Self { poll_rate }
    }
}

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

/// Mark that input polling should be enabled
fn ensure_input_initialized() {
    if !POLLING_TASK_RUNNING.swap(true, Ordering::SeqCst) {
        // Initialize global state on first call
        let current = !unsafe { KEYPAD_INPUT.read_volatile() };
        GLOBAL_BUTTON_STATE.store(current, Ordering::SeqCst);
    }
}

/// Check for button changes and wake appropriate wakers
fn poll_input_changes() {
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

/// Background task that polls input at the configured rate
#[cfg(all(feature = "time", feature = "executor"))]
#[embassy_executor::task]
pub async fn input_polling_task(config: InputConfig) {
    let poll_interval_ms = 1000 / config.poll_rate.as_hz() as u64;

    // Initialize global button state
    let current = !unsafe { KEYPAD_INPUT.read_volatile() };
    GLOBAL_BUTTON_STATE.store(current, Ordering::SeqCst);

    loop {
        poll_input_changes();
        embassy_time::Timer::after(embassy_time::Duration::from_millis(poll_interval_ms)).await;
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
    _config: InputConfig,
}

impl AsyncInput {
    pub(crate) fn new() -> Self {
        Self::with_config(InputConfig::default())
    }

    pub(crate) fn with_config(config: InputConfig) -> Self {
        ensure_input_initialized();

        Self {
            controller: ButtonController::new(),
            _config: config,
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

    /// Wait for a specific button to be pressed using agb's ButtonController
    pub async fn wait_for_button_press_polling(&mut self, button: Button) -> ButtonEvent {
        ButtonPressFuture::new(&mut self.controller, button).await
    }

    /// Wait for any button to be pressed using agb's ButtonController
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

    /// Check if a button is currently pressed using agb's ButtonController
    pub fn is_pressed_polling(&self, button: Button) -> bool {
        self.controller.is_pressed(button)
    }

    /// Check if a button was just pressed this frame using agb's ButtonController
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

/// Future that waits for a specific button press using agb's ButtonController
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

/// Future that waits for any button press using agb's ButtonController
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
