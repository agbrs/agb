# Paddle movement

So far we have a static game that you can't interact with.
In this section, we'll make the paddle move while pressing the D-Pad.

# The GBA controls

The GBA has 10 buttons we can read the state of, and this is the only way a player can directly control the game.
They are the 4 directions on the D-Pad, A, B, Start, Select, and the L and R triggers.

On a standard QWERTY keyboard, the default configuration on mGBA is as follows:

| GBA button | mGBA       |
| ---------- | ---------- |
| D-pad      | Arrow keys |
| A          | X          |
| B          | Z          |
| Start      | Enter      |
| Select     | Backspace  |
| L trigger  | A          |
| R trigger  | S          |

# Reading the button state

To add button control to our game, we will need a [`ButtonController`](https://docs.rs/agb/latest/agb/input/struct.ButtonController.html).
Add this near the top of your main function:

```rust
let mut button_controller = agb::input::ButtonController::new();
```

The button controller is not part of the `Gba` struct because it only allows for reading and not writing so does not need to be controlled by the borrow checker.

At the start of the loop, you should update the button state with:

```rust
button_controller.update();
```

To handle the movement of the paddles, let's add a new method to the `Paddle` struct.

```rust
pub fn move_by(&mut self, y: i32) {
    self.y += y;
}
```

You can use the `y_tri()` method to get the current state of the up-down buttons on the D-Pad.
It returns an instance of the [`Tri`](https://docs.rs/agb/latest/agb/input/enum.Tri.html) enum which describes which buttons are being pressed, and are very helpful in situations like these where you want to move something in a cardinal direction based on which buttons are pressed.

Add the following code after the call to `button_controller.update()`.

```rust
paddle_a.move_by(button_controller.y_tri() as i32);
```

You will have to mark `paddle_a` as `mut` for this to compile.

# What we did

We've learned about how to handle button input in `agb` and you can now move the player paddle!
In the next section, we'll add some collision between the ball and the paddles.

# Exercise

Add a power-up which moves the player at twice the speed while pressing the `A` button by using the [`is_pressed()`](https://docs.rs/agb/latest/agb/input/struct.ButtonController.html#method.is_pressed) method.
