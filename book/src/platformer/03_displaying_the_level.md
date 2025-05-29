# Displaying the level

To display the level we have made, we will make use of `agb`'s `InfiniteScrolledMap`.
This is a background that we can scroll around to any position and provides a callback to define what each tile should be.
It tries to be as efficient as possible in minimising the number of calls to the callback and so the number of tiles we modify.
For the task of displaying a level or a world, the `InfiniteScrolledMap` is the best choice.

Lets add something convenient to the `Level` struct we have, a method to get the bounds in the form of a Rect.

```rust
impl Level {
    fn bounds(&self) -> Rect<i32> {
        Rect::new(
            vec2(0, 0),
            // rect is inclusive of the edge, so we'll need to
            // correct that by subtracting 1
            vec2(self.width as i32 - 1, self.height as i32 - 1),
        )
    }
}
```

Then we can define a `World` that encapsulates the background and the level.

```rust
struct World {
    level: &'static Level,
    bg: InfiniteScrolledMap,
}

impl World {
    fn new(level: &'static Level) -> Self {
        let bg = RegularBackground::new(
            Priority::P0,
            RegularBackgroundSize::Background32x32,
            TileFormat::FourBpp,
        );
        let bg = InfiniteScrolledMap::new(bg);

        World { level, bg }
    }

    fn set_pos(&mut self, pos: Vector2D<i32>) {
        // It's always a good idea to wrap the set_scroll_pos call.
        // By giving the callback every time, it makes lifetimes
        // easier to manage and is almost required to support streaming
        // in levels from some compressed form.
        self.bg.set_scroll_pos(pos, |pos| {
            // using the bounds we just added to see if the given
            // point is in our tiles
            let tile = if self.level.bounds().contains_point(pos) {
                // Calculating the index of the tile from the
                // coordiniates given
                let idx = pos.x + pos.y * self.level.width;
                self.level.background[idx as usize]
            } else {
                // Just use the transparent tile if we were to rende
                // outside of the level. This is a good tile to use as
                // agb may contain specific optimisations around the transparent tile.
                TileSetting::BLANK
            };

            (&tiles::TILES.tiles, tile)
        });
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
    }
}
```

And then we can now write our `main` function to use what we've just written.
This should look very familiar, using the normal graphics frame system.

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);
    let mut bg = World::new(levels::LEVELS[0]);

    loop {
        bg.set_pos(vec2(0, 0));

        let mut frame = gfx.frame();

        bg.show(&mut frame);

        frame.commit();
    }
}
```

Running this will now display our level on the screen.

# Summary

We've seen how to use the `InfiniteScrolledMap` to display a level that we made with Tiled to the screen.
Using the `set_pos` method, can you make a bigger level and scroll it around with the dpad?
