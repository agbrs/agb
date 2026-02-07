# The player

In this chapter, we'll get our player character on screen with movement, gravity, and jumping.
We'll use a temporary floor to keep the player from falling off the screen, and replace it with proper collision detection in the next chapter.

# The wizard sprite

This will be the character we use, the wizard from _The Hat Chooses the Wizard_.
The file contains a few tags that define various animations that we will be using.
We already placed this file at `gfx/sprites.aseprite` during setup.

<img src="./wizard.png" style="width: 128px; image-rendering: pixelated; display: block; margin-left: auto; margin-right: auto;" />

Add the sprite import to your `main.rs`, alongside the existing `include_background_gfx!`:

```rust
use agb::include_aseprite;

include_aseprite!(mod sprites, "gfx/sprites.aseprite");
```

The `include_aseprite!` macro works similarly to `include_background_gfx!` — it reads the aseprite file at compile time and creates a module with an entry for each tag.
For more detail, see the [Sprites](../pong/02_sprites.md) chapter of the pong tutorial and the [Objects deep dive](../articles/objects_deep_dive.md).

# Fixed-point numbers

We need sub-pixel precision for smooth movement.
We'll use `agb`'s fixed-point numbers for this — if you need a refresher, see the [Fixnums](../pong/07_fixnums.md) chapter of the pong tutorial or the [Fixnums article](../articles/fixed_point_numbers.md).

Define these type aliases near the top of `main.rs`:

```rust
use agb::fixnum::{Num, num};

type Number = Num<i32, 8>;
type Vector = Vector2D<Number>;
```

`Num<i32, 8>` gives us a fixed-point number with 8 fractional bits — enough precision for smooth sub-pixel movement.

# Step 1: Define the `Player` struct

```rust
use agb::display::object::{Object, SpriteVram};

struct Player {
    position: Vector,
    velocity: Vector,
    frame: usize,
    sprite: SpriteVram,
    flipped: bool,
    start_y: Number,
}

impl Player {
    fn new(start: Vector2D<i32>) -> Self {
        Player {
            position: start.change_base(),
            velocity: vec2(num!(0), num!(0)),
            frame: 0,
            sprite: sprites::STANDING.sprite(0).into(),
            flipped: false,
            start_y: Number::new(start.y),
        }
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(self.sprite.clone())
            .set_hflip(self.flipped)
            .set_pos(self.position.round() - vec2(8, 8))
            .show(frame);
    }
}
```

The `sprite` field holds a `SpriteVram` — a reference to sprite data that has been loaded into video RAM.
We initialize it with the first frame of the `STANDING` animation.

The `show` method creates an `Object`, flips it horizontally if needed, and positions it.
We subtract `(8, 8)` because the sprite's origin is at its top-left corner, but we want `position` to represent the center of the sprite.

Update the `main` function to create and show the player:

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let level = levels::LEVELS[0];
    let mut bg = World::new(level);
    let mut player = Player::new(level.player_start.into());

    loop {
        bg.set_pos(vec2(0, 0));

        let mut frame = gfx.frame();

        bg.show(&mut frame);
        player.show(&mut frame);

        frame.commit();
    }
}
```

Run this now — you should see the wizard at the start position on your level.

# Step 2: Horizontal movement

Add input handling to the player.
First, add the `ButtonController` import:

```rust
use agb::input::{Button, ButtonController};
```

Now add the horizontal input handler:

```rust
impl Player {
    fn handle_horizontal_input(&mut self, x_tri: i32, on_ground: bool) {
        let mut x = x_tri;

        // If we're trying to move in a direction opposite to what
        // we're currently moving, we should decelerate faster.
        // This is a classic trick that makes direction changes feel
        // snappy — the original Super Mario Bros uses the same trick!
        if x_tri.signum() != self.velocity.x.to_raw().signum() {
            x *= 2;
        }

        if on_ground {
            x *= 2;
        }

        self.velocity.x += Number::new(x) / 16;
    }
}
```

The opposing-direction trick is subtle but important: if you're moving right and press left, we double the deceleration so the character responds immediately.
When on the ground, we also double the acceleration for snappier movement.

# Step 3: Jumping and gravity

```rust
impl Player {
    fn handle_jump(&mut self) {
        self.velocity.y = Number::new(-2);
    }
}
```

A simple negative Y velocity sends the player upward.
Many games reduce gravity while the jump button is held to allow variable jump heights — that's a good exercise to try later.

# Step 4: Animation

We need to update the sprite based on the player's state:

```rust
impl Player {
    fn update_sprite(&mut self) {
        self.frame += 1;

        // We need to keep track of the facing direction rather than
        // deriving it because the zero velocity case needs to keep
        // facing the same direction.
        if self.velocity.x > num!(0.1) {
            self.flipped = false;
        }
        if self.velocity.x < num!(-0.1) {
            self.flipped = true;
        }

        self.sprite = if self.velocity.y < num!(-0.1) {
            sprites::JUMPING.animation_frame(&mut self.frame, 2)
        } else if self.velocity.y > num!(0.1) {
            sprites::FALLING.animation_frame(&mut self.frame, 2)
        } else if self.velocity.x.abs() > num!(0.05) {
            sprites::WALKING.animation_frame(&mut self.frame, 2)
        } else {
            sprites::STANDING.animation_frame(&mut self.frame, 2)
        }
        .into()
    }
}
```

The `animation_frame` method cycles through the frames of a tag at the given speed.
The `2` parameter means each frame is displayed for 2 game frames.

# Step 5: The update function

Now tie it all together:

```rust
impl Player {
    fn update(&mut self, input: &ButtonController) {
        // We don't have collision yet, so use the starting y position
        // as a temporary floor to prevent the player from falling off
        // the screen.
        let on_ground = self.position.y >= self.start_y;

        self.handle_horizontal_input(input.x_tri() as i32, on_ground);

        if input.is_just_pressed(Button::A) && on_ground {
            self.handle_jump();
        }

        // gravity
        self.velocity.y += num!(0.05);

        // friction: multiply by 15/16 each frame
        self.velocity.x *= 15;
        self.velocity.x /= 16;

        // apply velocity to position (no collision yet!)
        self.position += self.velocity;

        // clamp to the temporary floor
        if self.position.y > self.start_y {
            self.position.y = self.start_y;
            self.velocity.y = num!(0);
        }

        self.update_sprite();
    }
}
```

Friction is applied as a simple ratio: each frame, horizontal velocity is multiplied by 15/16 (93.75%).
This causes the player to gradually slow down when not pressing any buttons.

We use the player's starting Y position as a temporary floor so we can test movement and jumping before we have real collision detection.
We'll replace this with proper collision in the next chapter.

Update the main loop to call `update`:

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.graphics.get();

    VRAM_MANAGER.set_background_palettes(tiles::PALETTES);

    let level = levels::LEVELS[0];
    let mut bg = World::new(level);
    let mut input = ButtonController::new();

    let mut player = Player::new(level.player_start.into());

    loop {
        input.update();
        bg.set_pos(vec2(0, 0));
        player.update(&input);

        let mut frame = gfx.frame();

        bg.show(&mut frame);
        player.show(&mut frame);

        frame.commit();
    }
}
```

Run the game now.
You should see the wizard standing at the start position.
You can move left and right with the D-Pad and jump with A.
The player won't fall through the floor thanks to our temporary clamp, but they will walk through walls — we'll add proper collision in the next chapter.

# What we did

We've got the player on screen with movement, gravity, and jumping.
The character can move left and right, jump, and is affected by gravity.
The temporary floor keeps the player from falling off the screen, but they pass through walls and platforms.
In the next chapter, we'll add proper collision detection.

# Exercise

Experiment with the gravity and jump velocity values.
What happens if you set gravity to `num!(0.1)`?
What about jump velocity of `num!(-3)`?
Many platformers spend a lot of time tuning these values to get the movement feeling right.
