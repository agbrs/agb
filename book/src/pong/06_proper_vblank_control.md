# Proper vblank control

So far we've been using `agb::display::busy_wait_for_vblank()`.
This method is also referred to as a 'space heater' because it runs the CPU at 100% while we wait for the screen to finish rendering.
Doing so isn't great for the battery life of your GBA, so ideally we'd like to pause the CPU while we wait for rendering to finish.

You can do this by creating an instance of a [`VBlank`](https://docs.rs/agb/latest/agb/interrupt/struct.VBlank.html) using the
[`agb::interrupt::VBlank::get()`](https://docs.rs/agb/latest/agb/interrupt/struct.VBlank.html#method.get) function and then
calling [`wait_for_vblank()`](https://docs.rs/agb/latest/agb/interrupt/struct.VBlank.html#method.wait_for_vblank) on it.
The `wait_for_vblank()` method will pause the CPU until the vblank happens, and then wake up again.
This puts the CPU in a low power state until rendering is finished, saving battery on the console.

Add this to the start of your main function:

```rust
    let vblank_provider = agb::interrupt::VBlank::get();
```

and then replace the call to `busy_wait_for_vblank()` with

```rust
    vblank_provider.wait_for_vblank();
```

# What we did

We saved a fair bit of CPU time and therefore battery life in a battery powered console by pausing the CPU while we wait for rendering to finish.

In the next part, we'll implement some collision between the paddles and the ball.
