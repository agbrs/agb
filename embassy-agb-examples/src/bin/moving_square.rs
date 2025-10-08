//! Moving sprite example
//!
//! Demonstrates async display + async input working together:
//! - VBlank-synchronized rendering for smooth 60Hz display
//! - Timer-based input polling for responsive controls (configurable rate)
//! - Embassy task coordination between input and display
//! - Object (sprite) movement with position clamping
//!
//! System architecture:
//! ```text
//! Embassy Timer → [Input Polling Task] → [Button Wakers] → [Input Task]
//!                                                               ↓
//!                                                        [MOVEMENT_SIGNAL]
//!                                                            (shared)
//!                                                               ↑
//! VBlank Interrupt → [Display Loop] → [try_take()] ─────────────┘
//!                         ↓
//!             [Update Position & Render]
//! ```
//!
//! Controls: D-pad moves the sprite, clamped to screen edges
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
    input::{AsyncInput, ButtonEvent, InputConfig, PollingRate},
    sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal},
    Spawner,
};

// Signal to communicate movement from input task to display task
#[derive(Clone, Copy)]
enum Movement {
    Up,
    Down,
    Left,
    Right,
}

static MOVEMENT_SIGNAL: Signal<CriticalSectionRawMutex, Movement> = Signal::new();

// Input task: detect button presses and signal movement
#[embassy_executor::task]
async fn input_task(mut input: AsyncInput) {
    loop {
        let (button, event) = input.wait_for_any_button_press().await;

        if event == ButtonEvent::Pressed {
            match button {
                Button::UP => MOVEMENT_SIGNAL.signal(Movement::Up),
                Button::DOWN => MOVEMENT_SIGNAL.signal(Movement::Down),
                Button::LEFT => MOVEMENT_SIGNAL.signal(Movement::Left),
                Button::RIGHT => MOVEMENT_SIGNAL.signal(Movement::Right),
                _ => {} // Ignore other buttons
            }
        }
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
        palette[1] = Rgb15::new(0x7C00); // Red
        Palette16::new(palette)
    };

    // Create a simple 8x8 red square sprite
    let mut sprite = DynamicSprite16::new(Size::S8x8);
    for y in 0..8 {
        for x in 0..8 {
            sprite.set_pixel(x, y, 1); // Red square
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
    const MOVE_SPEED: i32 = 4;

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

        // Check for movement input (non-blocking)
        if let Some(movement) = MOVEMENT_SIGNAL.try_take() {
            // Update sprite position based on input
            match movement {
                Movement::Up => position.y -= MOVE_SPEED,
                Movement::Down => position.y += MOVE_SPEED,
                Movement::Left => position.x -= MOVE_SPEED,
                Movement::Right => position.x += MOVE_SPEED,
            }

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
