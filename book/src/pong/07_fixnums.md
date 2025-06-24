# Fixnums

Currently the gameplay of our pong game is a little un-exciting.
Part of this reason is that the ball is always moving at a 45° angle.
However, it is currently moving at 1 pixel per frame in the horizontal and vertical directions.
So to move at a different angle without making the game run too fast for us to be able to react, we need to make it move at less than 1 pixel per frame.

You may want to reach out to [floating point numbers](https://en.wikipedia.org/wiki/Floating-point_arithmetic) to do this, but on the Game Boy Advance, this is a big problem.

The Game Boy Advance doesn't have a [floating point unit](https://en.wikipedia.org/wiki/Floating-point_unit),
so all work with floating point numbers is done in software, which is really slow, especially on the 16MHz processor of the console.
Even simple operations, like addition of two floating point numbers will take 100s of CPU cycles, so ideally we'd avoid needing to use that.

The solution to this problem used by almost every Game Boy Advance game is to use 'fixed point numbers' rather than floating point numbers.

# Preliminary refactor

Before we go to put fixed point numbers in the game, we need to do a quick change to pull the ball into its own struct.

```rust
pub struct Ball {
    pos: Vector2D<i32>,
    velocity: Vector2D<i32>,
}

impl Ball {
    pub fn new(pos: Vector2D<i32>, velocity: Vector2D<i32>) -> Self {
        Self { pos, velocity }
    }

    pub fn update(&mut self, paddle_a: &Paddle, paddle_b: &Paddle) {
        // Speculatively move the ball, we'll update the velocity if this causes it to intersect with either the
        // edge of the map or a paddle.
        let potential_ball_pos = self.pos + self.velocity;

        let ball_rect = Rect::new(potential_ball_pos, vec2(16, 16));
        if paddle_a.collision_rect().touches(ball_rect) {
            self.velocity.x = 1;
        }

        if paddle_b.collision_rect().touches(ball_rect) {
            self.velocity.x = -1;
        }

        // We check if the ball reaches the edge of the screen and reverse it's direction
        if potential_ball_pos.x <= 0 || potential_ball_pos.x >= agb::display::WIDTH - 16 {
            self.velocity.x *= -1;
        }

        if potential_ball_pos.y <= 0 || potential_ball_pos.y >= agb::display::HEIGHT - 16 {
            self.velocity.y *= -1;
        }

        self.pos += self.velocity;
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(sprites::BALL.sprite(0))
            .set_pos(self.pos)
            .show(frame);
    }
}
```

Then replace all the ball related code outside of the loop with

```rust
let mut ball = Ball::new(vec2(50, 50), vec2(1, 1));
```

and the collision handling code can be replaced with

```rust
ball.update(&paddle_a, &paddle_b);
```

Since we've kept the `.show()` pattern, you don't need to update the call to `ball.show()`.

# Using fixnums

Fixed point numbers (fixnums) store a fixed number of bits for the fractional part of the number, rather than how floating point numbers are stored.
This allows for very fast addition and multiplication, but you can't store very large or very small numbers any more.

Let's first swap all of the positions with a fixed point number.
Firstly, we'll define a type for our fixed point numbers for this game:

```rust
use agb::fixnum::{Num, num};

type Fixed = Num<i32, 8>;
```

[`Num<i32, 8>`](https://docs.rs/agb/latest/agb/fixnum/struct.Num.html) means we'll store 8 bits of precision
(allowing for up to 256 values between each integer value) with an underlying integer type of `i32`.
This is a pretty good default to use for most fixed number usage in the Game Boy Advance, since it strikes a pretty good balance between being reasonably precise, while giving a pretty good range of possible maximum and minimum values.
Also, the Game Boy Advance is a 32-bit platform, so is optimised for 32-bit arithmetic operations.
Adding and subtracting with 32-bit values is often faster than working with 16-bit values.

We'll now replace the paddle position and the ball position and velocity with `Fixed` instead of `i32`, fixing compiler errors as you go.

Some notable changes:

```rust
pub fn move_by(&mut self, y: Fixed) {
    // we now need to cast the 0 to a Fixed which you can do with
    // `Fixed::from(0)` or `0.into()`. But the preferred one is the `num!` macro
    // which we imported above.
    self.pos += vec2(num!(0), y);
}

pub fn collision_rect(&self) -> Rect<Fixed> {
    // Same idea here with creating a fixed point rectangle
    Rect::new(self.pos, vec2(num!(16), num!(16 * 3)))
}
```

Since you can only show things on the Game Boy Advance's screen in whole pixel coordinates, you'll need to convert the fixed number to an
integer to show the paddle in a specific location:

```rust
pub fn show(&self, frame: &mut GraphicsFrame) {
    let sprite_pos = self.pos.round();

    Object::new(sprites::PADDLE_END.sprite(0))
        .set_pos(sprite_pos)
        .show(frame);
    Object::new(sprites::PADDLE_MID.sprite(0))
        .set_pos(sprite_pos + vec2(0, 16))
        .show(frame);
    Object::new(sprites::PADDLE_END.sprite(0))
        .set_pos(sprite_pos + vec2(0, 32))
        .set_vflip(true)
        .show(frame);
}
```

It is best to use [`.round()`](https://docs.rs/agb/lastest/agb/fixnum/struct.Vector2D.html#method.round) rather than `.floor()` for converting from fixnums back to integers because it works better when approaching integer locations (which becomes more relevant if you add some smooth animations in future).

The call to `paddle_a.move_by()` needs updating using `Fixed::from(...)` rather than `num!(...)` because the [`num!()`](https://docs.rs/agb/latest/agb/fixnum/macro.num.html) macro requires a constant value.

Once you've done all these changes and the code now compiles, if you run the game, it will be exactly the same as before.
However, we'll now take advantage of those fixed point numbers.

# More dynamic movement

Let's first make the ball move less vertically by setting the initial ball velocity to `0.5`.

```rust
let mut ball = Ball::new(vec2(num!(50), num!(50)), vec2(num!(1), num!(0.5)));
```

But now it feels a bit slow, so maybe increase the horizontal speed a little as well to maybe `2`.

Now we notice that the paddle collision sets the horizontal speed component to `1`, so update that:

```rust
if paddle_a.collision_rect().touches(ball_rect) {
    self.velocity.x = self.velocity.x.abs();
}

if paddle_b.collision_rect().touches(ball_rect) {
    self.velocity.x = -self.velocity.x.abs();
}
```

And finally, to make it slightly more exciting, let's alter the `y` component depending on where the hit happened by putting this
inside the `if` statement where we handle the collision.

```rust
let y_difference = (ball_rect.centre().y - paddle_a.collision_rect().centre().y) / 32;
self.velocity.y += y_difference;
```

And something similar for the `paddle_b` case.

Now the game feels a lot more dynamic where the game changes depending on where you hit the ball.

# What we did

We learned the basics of using fixed point numbers, and made the game feel more interesting by making the ball movement depend on how you hit it.
Next we'll add some sound effects and background music to make the game feel a bit more dynamic.

# Exercise

Change the velocity calculations to instead change the angle but keep the speed the same.
Then make the ball speed up a bit after each hit so that eventually you won't be able to always return the ball.

# See also

The [fixnum deep dive article](../articles/fixed_point_numbers.md).
