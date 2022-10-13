# Controls

In this section, we'll make the ball that we displayed in the last section move by pressing the D-Pad.

# The GBA controls

The GBA has 10 buttons we can read the state of, and this is the only way a player can directly control the game.
They are the 4 directions on the D-Pad, A, B, Start, Select, and the L and R triggers.

# Reading the button state

There are two ways to capture the button state in **agb**, interrupts and polling.
In most games, you will want to use polling, so that is what we will use now.
Interrupts will be covered in a later chapter.

To add button control to our game, we will need a [ButtonController](https://docs.rs/agb/latest/agb/input/struct.ButtonController.html).
Add this near the top of your main function:

```rust
    let mut input = agb::input::ButtonController::new();
```

The button controller is not part of the `Gba` struct because it only allows for reading and not writing so does not need to be controlled by the borrow checker.