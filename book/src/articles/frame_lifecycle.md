# Frame lifecycle

Games written using `agb` typically follow the ['update-render loop'](https://gameprogrammingpatterns.com/game-loop.html).
The way your components update will vary depending on the game you are writing, but each frame you would normally do the following:

```rust
let mut gfx = gba.graphics.get();

loop {
    my_game.update();

    let mut frame = gfx.frame();
    my_game.show(&mut frame);
    frame.commit();
}
```

Here we will discuss the idiomatic way of using the [`GraphicsFrame`](https://docs.rs/agb/latest/agb/display/struct.GraphicsFrame.html) during the lifecycle of a frame.

## `.show(frame: &mut GraphicsFrame)`

The most common pattern involving `GraphicsFrame` you'll see in the `agb` library is a `.show()` method which typically accepts a mutable reference to a `GraphicsFrame`.

Due to this naming convention, it is also conventional in games written using `agb` to name the `render` method `show()` and have the same method signature.
You should not be doing any mutation of state during the `show()` method, and as much loading and other CPU intensive work as possible should be done prior to the call to `show()`.

See the [frame lifecycle](https://agbrs.dev/examples/frame_lifecycle) example for a simple walkthrough for how to manage a frame with a single player character.

## `.commit()`

Once everything you want to be visible on the frame is ready, you should follow this up with a call to `.commit()` on the frame.
This will wait for the current frame to finish rendering before quickly setting everything up for the next frame.

This method takes ownership of the current `frame` instance, so you won't be able to use it for any further calls once this is done.
You will need to create a new frame object from the `gfx` instance.

# See also

These are the various aspects of `agb` that interact with the `GraphicsFrame` system.

- [Backgrounds](./backgrounds.md)
- [Objects](./objects_deep_dive.md)
- [Blending and windows](./blending_and_windows.md)
- [DMA](./dma.md)
