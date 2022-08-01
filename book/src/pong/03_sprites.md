# Sprites

In this section, we'll put the sprites needed for our pong game onto the screen.
We'll cover what sprites are in the Game Boy Advance, and how to get them to show up on screen.
We'll briefly cover vblank and by the end of this section, you'll have a ball bouncing around the screen!

# Why do we need sprites in the first place?

The Game Boy Advance has a 240x160px screen, with 15-bit RGB colour support.
In order to manually set the colour for each pixel in the screen, you would need to update a total of 38,400 pixels per frame, or 2,304,000 pixels per second at 60 fps.
With a 16MHz processor, that means you would need to be able to calculate 1 pixel every 8 clock cycles, which is pretty much impossible.
You could get clever with how you update these pixels, but using the tools provided by the Game Boy Advance to put pixels on the screen, you'll have a much easier time.

So there are 2 ways that the Game Boy Advance allows you to get these pixels on screen much more easily.
Tiles and sprites.
Tiles are 8x8 pixels in size and can be placed in a grid on the screen.
You can also scroll the whole tile layer to arbitrary positions, but the tiles will remain in this 8x8 pixel grid.
We'll cover tiles in more detail later.

The other way you can draw things on screen is using sprites, which we'll cover in more detail in this section.

# Sprites on the Game Boy Advance

The Game Boy Advance supports 256 hardware sprites.
These can be in one of many sizes, ranging from square 8x8 to more exotic sizes like 8x32 pixels.
For our pong game, all the sprites will be 16x16 pixels to make things a bit simpler.

Sprites are stored in the Game Boy Advance in a special area of video memory called the 'Object Attribute Memory' (OAM).
This has space for the 'attributes' of the sprites (things like whether or not they are visible, the location, which tile to use etc) but it does not store the actual pixel data.
The pixel data is stored in a different part of video RAM (VRAM) and the OAM only stores which tiles to use from this area.

Since RAM is in short supply, and at the time was quite expensive, the tile data is stored as indexed palette data.
So rather than storing the full colour data for each pixel in the tile, the Game Boy Advance instead stores a 'palette' of colours and the tiles which make up the sprites are stored as indexes to the palette.
You don't need to worry about this though, because `agb` handles it for you, but it is important to keep in mind that each sprite can use a maximum of 16 colours out of the total sprite palette of 256 colours.

There are technically 2 types of sprite, regular and affine sprites.
For now, we will only be dealing with regular sprites.

# Import the sprite

Firstly, you're going to need to import the sprites into your project.
`agb` has great support for the [aseprite](https://www.aseprite.org/) sprite editor which can be bought for $20 or you can compile it yourself for free.
Aseprite files can be natively imported by `agb` for use on the Game Boy Advance.
Here is the sprite sheet we will use as a png, but you should [download the aseprite file](sprites.aseprite) and place it in `gfx/sprites.aseprite`.

![pong sprites](sprites.png)

This contains 5 `16x16px` sprites.
The first is the end cap for the paddle.
The second is the centre part of the paddle, which could potentially be repeated a few times.
The third until the fifth is the ball, with various squashed states.
The aseprite file defines tags for these sprites, being "Paddle End", "Paddle Mid", and "Ball".

```rust
use agb::{include_aseprite,
    display::object::{Graphics, Tag}
};

// Import the sprites in to this constant. This holds the sprite 
// and palette data in a way that is manageable by agb.
const GRAPHICS: &Graphics = include_aseprite!("gfx/sprites.aseprite");

// We define some easy ways of referencing the sprites
const PADDLE_END: &Tag = GRAPHICS.tags().get("Paddle End");
const PADDLE_MID: &Tag = GRAPHICS.tags().get("Paddle Mid");
const BALL: &Tag = GRAPHICS.tags().get("Ball");
```

This uses the `include_aseprite` macro to include the sprites in the given aseprite file.
Now, let's put this on screen by firstly creating the object manager and then creating an object, this will also involve the creation of the main entry function using the `entry` macro.
The signature of this function takes the `Gba` struct and has the never return type, this means Rust will enforce that this function never returns, for now we will achieve this using a busy loop.
Using the `Gba` struct we get the [`ObjectController` struct](https://docs.rs/agb/latest/agb/display/object/struct.ObjectController.html) which manages loading and unloading sprites and objects.

```rust
#[agb::entry]
fn main(gba: mut agb::Gba) -> ! {
    // Get the OAM manager
    let object = gba.display.object.get();

    // Create an object with the ball sprite
    let mut ball = object.object_sprite(BALL.sprite(0));

    // Place this at some point on the screen, (50, 50) for example
    ball.set_x(50).set_y(50).show();

    // Now commit the object controller so this change is reflected on the screen, 
    // this should normally be done in vblank but it'll work just fine here for now
    object.commit();
    
    loop {}
}
```

If you run this you should now see the ball for this pong game somewhere in the top left of the screen.

# Making the sprite move

As mentioned before, you should `.commit()` your sprites only during `vblank` which is the (very short) period of time nothing is being rendered to screen.
`agb` provides a convenience function for waiting until this happens called `agb::display::busy_wait_for_vblank()`.
You shouldn't use this is a real game (we'll do it properly later on), but for now we can use this to wait for the correct time to `commit` our sprites to memory.

Making the sprite move 1 pixel every frame (so approximately 60 pixels per second) can be done as follows:

```rust
// replace the call to object.commit() with the following:

let mut ball_x = 50;
let mut ball_y = 50;
let mut x_velocity = 1;
let mut y_velocity = 1;

loop {
    // This will calculate the new position and enforce the position
    // of the ball remains within the screen
    ball_x = (ball_x + x_velocity).clamp(0, agb::display::WIDTH - 16);
    ball_y = (ball_y + y_velocity).clamp(0, agb::display::HEIGHT - 16);

    // We check if the ball reaches the edge of the screen and reverse it's direction
    if ball_x == 0 || ball_x == agb::display::WIDTH - 16 {
        x_velocity = -x_velocity;
    }

    if ball_y == 0 || ball_y == agb::display::HEIGHT - 16 {
        y_velocity = -y_velocity;
    }

    // Set the position of the ball to match our new calculated position
    ball.set_x(ball_x as u16).set_y(ball_y as u16);

    // Wait for vblank, then commit the objects to the screen
    agb::display::busy_wait_for_vblank();
    object.commit();
}
```

# What we did

In this section, we covered why sprites are important, how to create and manage them using the `ObjectController` in `agb` and make a ball bounce around the screen.