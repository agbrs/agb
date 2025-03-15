# Paddle movement

So far we have a static game that you can't interact with.
In this section, we're going to include the knowledge from the controls section to move the player paddle.

# Controls

First we'll need a button controller, which we can create as follows (near the top of the main function):

```rust
    let mut button_controller = ButtonController::new();
```

and then at the start of the loop, you should update the button state with:

```rust
        button_controller.update();
```

We probably want to move the paddle before handling collisions (from a game design perspective), so lets do that now.
Firstly, we'll add a `move_by()` method to `Paddle`.

```rust
    fn move_by(&mut self, y: i32) {
        let current_pos = self.start.position();
        self.set_position(current_pos + vec2(0, y));
    }
```

Then we can call this with the current `y_tri` on only `paddle_a` as follows (below the `button_controller.update()` line):

```rust
        paddle_a.move_by(button_controller.y_tri() as i32);
```

You will have to mark `paddle_a` as `mut` for this to compile.

# What we did

You can now move the player paddle! Next we'll take an aside from gameplay and put a cool image in the background.

# Exercise

The CPU player could do with moving too. Implement movement for the CPU player in what ever way seems interesting to you.
