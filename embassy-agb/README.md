# Embassy AGB - Async Support for Game Boy Advance Development

This crate provides async/await support for Game Boy Advance development using the [embassy](https://embassy.dev) executor integrated with the [agb](https://agbrs.dev) library.

## Features

- **Async/await support**: Write GBA games using modern async Rust
- **Embassy executor integration**: Leverage embassy's powerful task scheduling
- **Time driver**: Precise timing using GBA's hardware timers
- **Async APIs**: Async wrappers for display, input, and sound operations
- **Full agb compatibility**: Works alongside existing agb code

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
embassy-agb = { path = "path/to/embassy-agb" }
```

Create an async GBA application:

```rust
#![no_std]
#![no_main]

use embassy_agb::time::Timer;
use embassy_executor::Spawner;

#[embassy_agb::main]
async fn main(spawner: Spawner) {
    let mut gba = embassy_agb::init(Default::default());
    
    // Spawn background tasks
    spawner.spawn(display_task(gba.display())).unwrap();
    spawner.spawn(audio_task(gba.mixer())).unwrap();
    
    // Main game loop
    let mut input = gba.input();
    loop {
        // Wait for input asynchronously
        let (button, event) = input.wait_for_any_button_press().await;
        
        // Handle input...
        
        // Run at 60 FPS
        Timer::after_millis(16).await;
    }
}

#[embassy_agb::task]
async fn display_task(mut display: embassy_agb::display::AsyncDisplay<'_>) {
    loop {
        // Wait for VBlank and render frame
        let mut frame = display.frame().await;
        // Rendering code...
    }
}

#[embassy_agb::task] 
async fn audio_task(mut mixer: embassy_agb::sound::AsyncMixer<'_>) {
    mixer.init(agb::sound::mixer::Frequency::Hz32768);
    
    loop {
        // Process audio frame
        mixer.frame().await;
    }
}
```

## Architecture

Embassy-agb integrates the embassy async executor with agb's hardware abstraction:

- **Executor**: Uses embassy's `arch-spin` executor optimized for the GBA's ARM7TDMI processor
- **Time Driver**: Implements embassy's time driver interface using GBA Timer0/Timer1
- **Async APIs**: Provides async wrappers around agb's display, input, and sound systems
- **Task Management**: Supports spawning multiple concurrent tasks for different game systems

## Timing and Performance

- **Time Resolution**: 32.768kHz tick rate for precise timing
- **Frame Rate**: Designed for 60 FPS game loops
- **Power Efficiency**: Uses `halt()` instruction when no tasks are ready
- **Memory Overhead**: Minimal overhead over synchronous agb code

## Compatibility

Embassy-agb is designed to be fully compatible with existing agb code:

- Use `gba.agb()` to access the underlying `agb::Gba` instance
- Mix async and sync code as needed
- Existing agb examples can be gradually migrated to async

## Examples

See the `examples/` directory for complete examples demonstrating:

- Basic async game loop
- Multi-task game architecture
- Async input handling
- Async display operations
- Async audio mixing
- Integration with existing agb code

## Requirements

- Rust nightly (required by agb)
- GBA development toolchain
- Embassy ecosystem crates

## License

Licensed under the Mozilla Public License 2.0, same as agb.
