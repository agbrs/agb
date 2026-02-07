# Displaying the level

To display the level we have made, we will make use of `agb`'s `InfiniteScrolledMap`.
The `InfiniteScrolledMap` wraps a regular background and handles scrolling for you.
It calls a callback for each tile position that becomes visible on screen, asking "what tile should I display here?"
This means it only updates tiles that have actually changed, which is very efficient.
For the task of displaying a level or a world, the `InfiniteScrolledMap` is the best choice.

# Updating our imports

Update the imports in `src/main.rs` to include everything we need for this chapter:

```rust
use agb::{
    display::{
        GraphicsFrame, Priority,
        tiled::{
            InfiniteScrolledMap, RegularBackground, RegularBackgroundSize,
            TileFormat, TileSetting, VRAM_MANAGER,
        },
    },
    fixnum::{Rect, Vector2D, vec2},
    include_background_gfx,
};

extern crate alloc;
```

# Adding `bounds()` to `Level`

Let's add a convenience method to the `Level` struct to get its bounds as a `Rect`.
Add this after the `Level` struct definition:

```rust
impl Level {
    fn bounds(&self) -> Rect<i32> {
        Rect::new(
            vec2(0, 0),
            // Rect's size is inclusive of the edge, so a size of (width - 1)
            // covers tile coordinates 0 through width - 1.
            vec2(self.width as i32 - 1, self.height as i32 - 1),
        )
    }
}
```

# The `World` struct

We'll encapsulate the background and the level into a `World` struct:

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
        self.bg.set_scroll_pos(pos, |pos| {
            let tile = if self.level.bounds().contains_point(pos) {
                // Convert 2D coordinates to a 1D index into our flat tile array.
                // This is called row-major indexing.
                let idx = pos.x + pos.y * self.level.width as i32;
                self.level.background[idx as usize]
            } else {
                // Use the transparent tile outside the level bounds.
                // agb may contain specific optimizations around the blank tile.
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

It's always a good idea to wrap the `set_scroll_pos` call in a method like this.
By providing the callback every time, it makes lifetimes easier to manage.

The callback receives a `Vector2D<i32>` representing a tile coordinate (not a pixel coordinate).
We return a tuple of the tileset and the `TileSetting` for that position.

# The main function

Now we can write the main function to use our `World`:

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);
    let mut bg = World::new(levels::LEVELS[0]);

    loop {
        // We pass (0, 0) for now since the camera doesn't move yet.
        // We'll revisit this when we add a player.
        bg.set_pos(vec2(0, 0));

        let mut frame = gfx.frame();

        bg.show(&mut frame);

        frame.commit();
    }
}
```

Running this should display your level on the screen!

# What we did

We've seen how to use the `InfiniteScrolledMap` to display a Tiled level on the screen.

# Exercise

Make a level wider than the screen (e.g. 60x20 tiles) and use the D-Pad to scroll around.
You'll need a `ButtonController` — see the [Paddle movement](../pong/04_paddle_movement.md) chapter for a refresher on handling input.
