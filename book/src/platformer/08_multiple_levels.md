# Multiple levels

Our game currently has a single level.
In this chapter, we'll add win detection, refactor the code to support multiple levels, and wire it all together.

# Refactoring: moving the player into `World`

First, let's move the `Player` into the `World` struct.
This makes it easier to reset everything when changing levels.

```rust
struct World {
    level: &'static Level,
    player: Player,
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

        World {
            level,
            bg,
            player: Player::new(level.player_start.into()),
        }
    }

    fn update(&mut self, input: &ButtonController) {
        self.set_pos(vec2(0, 0));

        self.player.update(input, self.level);
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
        self.player.show(frame);
    }
}
```

# Detecting level end

To advance to the next level, we need to check if the player has reached the win tile.
This is very similar to the `collides` method:

```rust
impl Level {
    fn wins(&self, tile: Vector2D<i32>) -> bool {
        if !self.bounds().contains_point(tile) {
            return false;
        }

        let idx = (tile.x + tile.y * self.width as i32) as usize;

        self.winning_map[idx / 8] & (1 << (idx % 8)) != 0
    }
}
```

Then add a `has_won` method to both `Player` and `World`:

```rust
impl Player {
    fn has_won(&self, level: &Level) -> bool {
        level.wins(self.position.floor() / 8)
    }
}

impl World {
    fn has_won(&self) -> bool {
        self.player.has_won(self.level)
    }
}
```

# Updating the main function

With these changes, the main function becomes:

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let mut level = 0;
    let mut world = World::new(levels::LEVELS[level]);
    let mut input = ButtonController::new();

    loop {
        input.update();
        world.update(&input);

        let mut frame = gfx.frame();

        world.show(&mut frame);

        frame.commit();

        if world.has_won() {
            level += 1;
            level %= levels::LEVELS.len();
            world = World::new(levels::LEVELS[level]);
        }
    }
}
```

When the player reaches a win tile, we advance to the next level (wrapping back to the first level after the last one).

# Adding more levels

To add more levels, create them in Tiled, save them in the `tiled/` directory, and add them to the level array in `build.rs`:

```rust
static LEVELS: &[&str] = &["level_01.tmx", "level_02.tmx"];
```

And the rest will be done for you!

# What we did

We've made a game that has multiple levels and can transition between them.
There are many aspects that could be improved in your games. Here are some ideas:

- **Loading transitions.** Loading happens over multiple frames and should be hidden from view.
- **Fall detection.** The level should restart if the player falls off the world, or levels should be designed such that it is impossible to fall off.
- **Camera scrolling.** Make the camera follow the player for levels larger than the screen.
- **Sound effects.** Add a jump sound or landing sound using `agb`'s audio support.

# Exercise

Add a death mechanic: if the player falls below the bottom of the level, restart the current level.
You'll need to check `player.position.y` against the level height (in pixels: `level.height * 8`).

If you completed the spike exercises from earlier chapters, add spike detection too — touching a spike tile should also restart the level.
