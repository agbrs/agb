//! Moving sprite example with button holding support
//!
//! Demonstrates async display + async input working together:
//! - VBlank-synchronized rendering for smooth 60Hz display
//! - Timer-based input polling for responsive controls (configurable rate)
//! - Embassy task coordination between input and display
//! - Object (sprite) movement with position clamping
//! - Supports holding multiple buttons for diagonal movement
//!
//! System architecture:
//! ```text
//! Embassy Timer → [Input Polling Task] → [BUTTON_STATE]
//!                                            (shared)
//!                                               ↑
//! VBlank Interrupt → [Display Loop] → [lock()] ─┘
//!                         ↓
//!        [Calculate Net Movement & Update Position]
//!                         ↓
//!                     [Render]
//! ```
//!
//! Controls: D-pad moves the sprite, clamped to screen edges
//! - Hold buttons for continuous movement
//! - Hold multiple buttons for diagonal movement (net vector calculated)
//! Input polling: 60Hz (configurable from 30-120Hz)

#![no_std]
#![no_main]

use embassy_agb::{
    agb::{
        display::{
            object::{DynamicSprite16, Object, Size},
            Palette16, Rgb15,
        },
        fixnum::{Num, Vector2D},
        input::Button,
    },
    input::{AsyncInput, InputConfig, PollingRate},
    sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex},
    Spawner,
};

// Shared button state between input task and main loop
#[derive(Clone, Copy, Default)]
struct ButtonState {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
}

impl ButtonState {
    /// Calculate net movement vector from current button state
    fn net_movement(&self) -> Vector2D<i32> {
        let mut x = 0;
        let mut y = 0;

        if self.left {
            x -= 1;
        }
        if self.right {
            x += 1;
        }
        if self.up {
            y -= 1;
        }
        if self.down {
            y += 1;
        }

        Vector2D::new(x, y)
    }
}

static BUTTON_STATE: Mutex<CriticalSectionRawMutex, ButtonState> = Mutex::new(ButtonState {
    up: false,
    down: false,
    left: false,
    right: false,
});

// Input task: continuously poll button state and update shared state
#[embassy_executor::task]
async fn input_task(input: AsyncInput) {
    loop {
        // Poll current button state (non-blocking)
        let up_pressed = input.is_pressed(Button::UP);
        let down_pressed = input.is_pressed(Button::DOWN);
        let left_pressed = input.is_pressed(Button::LEFT);
        let right_pressed = input.is_pressed(Button::RIGHT);

        // Update shared state
        {
            let mut state = BUTTON_STATE.lock().await;
            state.up = up_pressed;
            state.down = down_pressed;
            state.left = left_pressed;
            state.right = right_pressed;
        }

        // Small delay to avoid excessive CPU usage
        embassy_agb::time::Timer::after(embassy_agb::time::Duration::from_millis(16)).await;
    }
}

#[embassy_agb::main]
async fn main(spawner: Spawner) -> ! {
    let mut gba = embassy_agb::init(Default::default());

    // Configure input polling at 60Hz (matches VBlank rate)
    let input_config = InputConfig {
        poll_rate: PollingRate::Hz60,
    };

    // Spawn the input polling task
    spawner.spawn(embassy_agb::input::input_polling_task(input_config).unwrap());

    let input = gba.input_with_config(input_config);
    let mut display = gba.display();

    // Create sprite palette
    static SPRITE_PALETTE: Palette16 = const {
        let mut palette = [Rgb15::BLACK; 16];
        palette[0] = Rgb15::new(0x0000); // Transparent
        palette[1] = Rgb15::new(0x001F); // Blue (different from original)
        Palette16::new(palette)
    };

    // Create a simple 8x8 blue square sprite
    let mut sprite = DynamicSprite16::new(Size::S8x8);
    for y in 0..8 {
        for x in 0..8 {
            sprite.set_pixel(x, y, 1); // Blue square
        }
    }

    // Convert sprite to VRAM format
    let sprite_vram = sprite.to_vram(&SPRITE_PALETTE);

    // Sprite position (in pixels)
    let mut position: Vector2D<Num<i32, 8>> = Vector2D::new(
        Num::new(120), // Center X (240/2)
        Num::new(80),  // Center Y (160/2)
    );

    // Movement speed
    const MOVE_SPEED: i32 = 2;

    // Screen bounds (in pixels) - 8x8 sprite
    const MIN_X: i32 = 0;
    const MAX_X: i32 = 240 - 8; // Screen width - sprite width
    const MIN_Y: i32 = 0;
    const MAX_Y: i32 = 160 - 8; // Screen height - sprite height

    // Spawn input task
    spawner.spawn(input_task(input).unwrap());

    loop {
        // Wait for VBlank: ensures smooth rendering without tearing
        display.wait_for_vblank().await;

        // Get current button state and calculate net movement
        let movement = {
            let state = BUTTON_STATE.lock().await;
            state.net_movement()
        };

        // Apply movement if any buttons are pressed
        if movement.x != 0 || movement.y != 0 {
            // Calculate new position with net movement
            position.x += movement.x * MOVE_SPEED;
            position.y += movement.y * MOVE_SPEED;

            // Clamp to screen bounds
            position.x = position.x.clamp(Num::new(MIN_X), Num::new(MAX_X));
            position.y = position.y.clamp(Num::new(MIN_Y), Num::new(MAX_Y));
        }

        // Render frame: show sprite at current position
        let mut frame = display.frame_no_wait();

        // Create and show the sprite object
        Object::new(sprite_vram.clone())
            .set_pos(position.floor())
            .show(&mut frame);

        frame.commit();
    }
}
