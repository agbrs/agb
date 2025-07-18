# Paddle movement and collision

In this section we'll implement collision between the ball and the paddles to start having an actual game.

# Using Vector2D<i32>

However, the first thing we're going to do is a quick refactor to using `agb`'s [Vector2D](https://docs.rs/agb/latest/agb/fixnum/struct.Vector2D.html)
type for managing positions more easily.
Note that this is the [mathematical definition](<https://en.wikipedia.org/wiki/Vector_(mathematics_and_physics)>) of 'vector' rather than the computer science [dynamic array](https://en.wikipedia.org/wiki/Dynamic_array).

## Vector2D for the ball position and velocity

We're currently storing the ball's x and y coordinate as 2 separate variables, along with it's velocity.
Let's change that first.

Change ball position to:

```rust
let mut ball_pos = vec2(50, 50);
let mut ball_velocity = vec2(1, 1);
```

You will also need to add the relevant import line to the start of the file.
Which will be:

```rust
use agb::fixnum::{Vector2D, vec2};
```

Note that the `vec2` method is a convenience method which is the same as `Vector2D::new()` but shorter.

You can now simplify the calculation:

```rust
// Move the ball
ball_pos += ball_velocity;

// We check if the ball reaches the edge of the screen and reverse it's direction
if ball_pos.x <= 0 || ball_pos.x >= agb::display::WIDTH - 16 {
    ball_velocity.x *= -1;
}

if ball_pos.y <= 0 || ball_pos.y >= agb::display::HEIGHT - 16 {
    ball_velocity.y *= -1;
}

// Set the position of the ball to match our new calculated position
ball.set_pos(ball_pos);
```

## Vector2D for the paddle position

You can store the paddle position as `pos` instead of `x` and `y` separately:

```rust
pub struct Paddle {
    pos: Vector2D<i32>,
}
```

You can change the `set_pos()` method on `Paddle` to take a `Vector2D<i32>` instead of separate `x` and `y` arguments as follows:

```rust
pub fn set_pos(&mut self, pos: Vector2D<i32>) {
    self.pos = pos;
}
```

And when rendering:

```rust
pub fn show(frame: &mut GraphicsFrame) {
    Object::new(sprites::PADDLE_END.sprite(0))
        .set_pos(self.pos)
        .show(frame);
    Object::new(sprites::PADDLE_MID.sprite(0))
        .set_pos(self.pos + vec2(0, 16))
        .show(frame);
    Object::new(sprites::PADDLE_END.sprite(0))
        .set_pos(self.pos + vec2(0, 32))
        .set_vflip(true)
        .show(frame);
}
```

`move_by()` can also be updated as follows:

```rust
pub fn move_by(&mut self, y: i32) {
    self.y += vec2(0, y);
}
```

## Mini exercise

You will also need to update the `new()` function and the calls to `Paddle::new`.

# Collision handling

We now want to handle collision between the paddle and the ball.
We will assume that the ball and the paddle both have axis-aligned bounding boxes, which will make collision checks very easy.

`agb`'s fixnum library provides a `Rect` type which will allow us to detect this collision.

Lets add a simple method to the `Paddle` impl which returns the collision rectangle for it:

```rust
pub fn collision_rect(&self) -> Rect<i32> {
    Rect::new(self.pos, vec2(16, 16 * 3))
}
```

Don't forget to update the `use` statement:

```rust
use agb::fixnum::{Rect, Vector2D, vec2};
```

And then we can get the ball's collision rectangle in a similar way.
We can now implement collision between the ball and the paddle like so:

```rust
// Speculatively move the ball, we'll update the velocity if this causes it to
// intersect with either the edge of the map or a paddle.
let potential_ball_pos = ball_pos + ball_velocity;

let ball_rect = Rect::new(potential_ball_pos, vec2(16, 16));
if paddle_a.collision_rect().touches(ball_rect) {
    ball_velocity.x = 1;
}

if paddle_b.collision_rect().touches(ball_rect) {
    ball_velocity.x = -1;
}

// We check if the ball reaches the edge of the screen and reverse it's direction
if potential_ball_pos.x <= 0 || potential_ball_pos.x >= agb::display::WIDTH - 16 {
    ball_velocity.x *= -1;
}

if potential_ball_pos.y <= 0 || potential_ball_pos.y >= agb::display::HEIGHT - 16 {
    ball_velocity.y *= -1;
}

ball_pos += ball_velocity;
```

This now gives us collision between the paddles and the ball.

# What we did

We've refactored the code a little to use `Rect` and `Vector2D` which simplifies some of the code.
We've also now got collision handling between the paddle and the ball, which will set us up for paddle movement in the next section.

# Exercise

The CPU player could do with some moving now.
Implement some basic behaviour for them so that they try to return the ball.
