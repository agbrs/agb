# Multiple levels

The first thing we want to do here is to make some refactors.
Making the `World` struct hold the Player should be convenient.

```rust
struct World {
    level: &'static Level,
    // new! Player
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
            // new! make the player
            player: Player::new(level.player_start.into()),
        }
    }

    // new! an update function that updates the background and the player
    fn update(&mut self, input: &ButtonController) {
        self.set_pos(vec2(0, 0));

        self.player.update(input, self.level);
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        self.bg.show(frame);
        // new! show the player
        self.player.show(frame);
    }
}
```

And then we can use this in the main function

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let level = 0;
    // renamed to world
    let mut world = World::new(levels::LEVELS[level]);
    let mut input = ButtonController::new();
    // player removed

    loop {
        input.update();
        // replaced with `world.update`
        world.update(&input);

        let mut frame = gfx.frame();

        world.show(&mut frame);

        frame.commit();
    }
}
```

# Detecting level end

To advance to the next level, we'll want to check if you've won the current level.
This can be done using code very similar to `collides` on `Level`.

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

Then we'll add something on the `Player` to tell if it has won and forward this on the `World`.

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

Then with a small change to our main function we can advance to the next level when the player hits the flag

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    // new! mutable level
    let mut level = 0;
    let mut world = World::new(levels::LEVELS[level]);
    let mut input = ButtonController::new();

    loop {
        input.update();
        world.update(&input);

        let mut frame = gfx.frame();

        world.show(&mut frame);

        frame.commit();

        // new! handle winning the level and advancing to the next one
        if world.has_won() {
            level += 1;
            level %= levels::LEVELS.len();
            world = World::new(levels::LEVELS[level]);
        }
    }
}
```

# Actually adding more levels

To make more levels, create the level in tiled then add it to the level array in the `build.rs` file.

```rust
static LEVELS: &[&str] = &["level_01.tmx", "level_02.tmx"];
```

and the rest will be done for you!


# Summary

Here we've made a game that has multiple levels and can transition between them.
There are many aspects that should be improved in your games, these include

- Hiding the loading sequence. Loading happens over multiple frames and should be hidden from view.
- Falling off the world. The level should restart if the player falls off the world, or levels should be designed such that it is impossible to fall off.
