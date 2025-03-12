# Paddle movement and collision

In this section, we'll be making the paddles move using your knowledge of input handling.
We'll also implement collision between the ball and the paddles to start having an actual game.

# Using Vector2D<i32>

However, the first thing we're going to do is a quick refactor to using agb's [vector](https://docs.rs/agb/latest/agb/fixnum/struct.Vector2D.html)
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

Note that the `vec2` method is a convienence method which is the same as `Vector2D::new()` but shorter.

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
        ball.set_position(ball_pos);
```

## Vector2D for the paddle position

You can change the `set_position()` method on `Paddle` to take a `Vector2D<i32>` instead of separate `x` and `y` arguments as follows:

```rust
    fn set_position(&mut self, pos: Vector2D<i32>) {
        // new! use of the `set_position` method. This is a helper feature using
        // agb's vector types. For now we can just use it to avoid adding them
        // separately
        self.start.set_position(pos);
        self.mid.set_position(pos + vec2(0, 16));
        self.end.set_position(pos + vec2(0, 32));
    }
```

### Mini exercise

You will also need to update the `new()` function and the calls to `Paddle::new`.
