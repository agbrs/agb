# Controls

In this section, we'll make the ball that we displayed in the last section move by pressing the D-Pad.

# The GBA controls

The GBA has 10 buttons we can read the state of, and this is the only way a player can directly control the game.
They are the 4 directions on the D-Pad, A, B, Start, Select, and the L and R triggers.

# Reading the button state

To add button control to our game, we will need a [`ButtonController`](https://docs.rs/agb/latest/agb/input/struct.ButtonController.html).
Add this near the top of your main function:

```rust
let mut input = agb::input::ButtonController::new();
```

The button controller is not part of the `Gba` struct because it only allows for reading and not writing so does not need to be controlled by the borrow checker.

Replace the inner loop with the following:

```rust
let mut ball_x = 50;
let mut ball_y = 50;

// now we initialise the x and y velocities to 0 rather than 1
let mut x_velocity = 0;
let mut y_velocity = 0;

loop {
    ball_x = (ball_x + x_velocity).clamp(0, agb::display::WIDTH - 16);
    ball_y = (ball_y + y_velocity).clamp(0, agb::display::HEIGHT - 16);

    // x_tri and y_tri describe with -1, 0 and 1 which way the d-pad
    // buttons are being pressed
    x_velocity = input.x_tri() as i32;
    y_velocity = input.y_tri() as i32;

    ball.set_pos((ball_x, ball_y));

    let mut frame = object.frame();
    ball.show(&mut frame);

    frame.commit();

    // We must call input.update() every frame otherwise it won't update based
    // on the actual button press state.
    input.update();
}
```

Here we use the `x_tri()` and `y_tri()` methods.
They return instances of the [`Tri`](https://docs.rs/agb/latest/agb/input/enum.Tri.html) enum which describes which buttons are being pressed, and are very helpful in situations like these where you want to move something in a cardinal direction based on which buttons are pressed.

# Detecting individual button presses

If you want to detect if any button is pressed, you can use the `is_pressed` method on `ButtonController`.
For example, we can do the following:

```rust
use agb::input::Button;

if input.is_pressed(Button::A) {
    // the A button is pressed
}
```

`ButtonController` also provides the `is_just_pressed` method.
This will return true for 1 frame, the one where the player actually pressed the button.
From that point on, it'll return false again until the player presses it again.

# What we did

We added very basic button control to our bouncing ball example.
In the next step, we'll cover meta-sprites and actually add a bat to our game of pong.

# Exercise

Make it so the ball moves twice as fast if you're pressing the `A` button while moving it around.
