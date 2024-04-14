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
let mut paddle_start = object.object_sprite(PADDLE_END.sprite(0));
let mut paddle_mid = object.object_sprite(PADDLE_MID.sprite(0));
let mut paddle_end = object.object_sprite(PADDLE_END.sprite(0));

paddle_start.set_x(20).set_y(20).show();
paddle_mid.set_x(20).set_y(20 + 16).show();
paddle_end.set_x(20).set_y(20 + 16 * 2).show();
```

If you add this to your program, you'll see the paddle. But wait! The bottom of
the paddle is the wrong way around! Fortunately, the GBA can horizontally and vertically flip sprites.

```rust
paddle_end.set_vflip(true);
```

Now the paddle will display correctly. It's rather awkward to use, however, having to set all these positions correctly. Therefore we should encapsulate the logic of this object.

```rust
// change our imports to include what we will use
use agb::{
    display::object::{Graphics, Object, ObjectController, Tag},
    include_aseprite,
};

struct Paddle<'obj> {
    start: Object<'obj>,
    mid: Object<'obj>,
    end: Object<'obj>,
}

impl<'obj> Paddle<'obj> {
    fn new(object: &'obj ObjectController<'_>, start_x: i32, start_y: i32) -> Self {
        let mut paddle_start = object.object_sprite(PADDLE_END.sprite(0));
        let mut paddle_mid = object.object_sprite(PADDLE_MID.sprite(0));
        let mut paddle_end = object.object_sprite(PADDLE_END.sprite(0));

        paddle_start.show();
        paddle_mid.show();
        paddle_end.set_vflip(true).show();

        let mut paddle = Self {
            start: paddle_start,
            mid: paddle_mid,
            end: paddle_end,
        };

        paddle.set_position(start_x, start_y);

        paddle
    }

    fn set_position(&mut self, x: i32, y: i32) {
        // new! use of the `set_position` method. This is a helper feature using
        // agb's vector types. For now we can just use it to avoid adding them 
        // separately
        self.start.set_position((x, y));
        self.mid.set_position((x, y + 16));
        self.end.set_position((x, y + 32));
    }
}
```

Here we've made a struct to hold our paddle objects and added a convenient `new` and `set_position` function and methods to help us use it. Now we can easily create two paddles (one on each side of the screen).


```rust
// outside the loop
let mut paddle_a = Paddle::new(&object, 8, 8); // the left paddle
let mut paddle_b = Paddle::new(&object, 240 - 16 - 8, 8); // the right paddle
```

# What we did

We used multiple sprites to form one game object of a paddle. We also added
convenience around the use of the paddle to make creating a paddle and setting
its position easy.

# Exercise

The paddle on the right is facing the wrong way, it needs to be horizontally
flipped! Given that the method is called `set_hflip`, can you modify the code
such that both paddles face the correct direction.