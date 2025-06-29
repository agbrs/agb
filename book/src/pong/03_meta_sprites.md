# Meta Sprites

In this section we'll discuss how the GBA's concept on sprites and objects
doesn't need to correspond to your game's concept of objects and we will make
the paddle display on screen.

# What is a meta sprite?

Imagine all you had were 8x8 pixel sprites, but you wanted an enemy to be 16x16
pixels. You could use 4 sprites in a square arrangement to achieve this. Using
multiple of these GBA objects to form one of your game objects is what we call a
meta sprite.

# Making the paddle

In the paddle sprite we gave you a "Paddle End" and a "Paddle Mid". Therefore in
order to show a full paddle we will need 2 paddle ends with a paddle mid between
them.

Let's just write that and we'll get to neatening it up later.

```rust
// outside the game loop
let mut paddle_start = Object::new(sprites::PADDLE_END.sprite(0));
let mut paddle_mid = Object::new(sprites::PADDLE_MID.sprite(0));
let mut paddle_end = Object::new(sprites::PADDLE_END.sprite(0));

paddle_start.set_pos((20, 20));
paddle_mid.set_pos((20, 20 + 16));
paddle_end.set_pos((20, 20 + 16 * 2));
```

If you add this to your program and `show()` all of them, you'll see the paddle. But wait! The bottom of
the paddle is the wrong way around! Fortunately, the GBA can horizontally and vertically flip sprites.

```rust
paddle_end.set_vflip(true);
```

Now the paddle will display correctly. It's rather awkward to use, however, having to set all these positions correctly. Therefore we should encapsulate the logic of this object.

```rust
pub struct Paddle {
    x: i32,
    y: i32,
}

impl Paddle {
    pub fn new(start_x: i32, start_y: i32) -> Self {
        Self {
            x: start_x,
            y: start_y,
        }
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos((self.x, self.y))
            .show(frame);
        Object::new(sprites::PADDLE_MID.sprite(0))
            .set_pos((self.x, self.y + 16))
            .show(frame);
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos((self.x, self.y + 32))
            .set_vflip(true)
            .show(frame);
    }
}
```

Here we've made a struct to hold our paddle objects and added a convenient
`new`, `set_pos`, and `show` function and methods to help us use it. Now we
can easily create two paddles (one on each side of the screen).

```rust
// outside the loop
let mut paddle_a = Paddle::new(8, 8); // the left paddle
let mut paddle_b = Paddle::new(240 - 16 - 8, 8); // the right paddle
```

# What we did

We used multiple sprites to form one game object of a paddle. We also added
convenience around the use of the paddle to make creating a paddle and setting
its position easy.

In the next section, we'll allow the player paddle to move around.

# Exercise

The paddle on the right is facing the wrong way, it needs to be horizontally
flipped! Given that the method is called `set_hflip`, can you modify the code
such that both paddles face the correct direction.
