# Sprites

In this section, we'll cover what sprites are in the Game Boy Advance and how to put them on the screen in our pong game.
We'll briefly cover vblank, and by the end of this section, you'll have a ball bouncing around the screen!

# Why do we need sprites?

The Game Boy Advance has a 240x160px screen with 15-bit RGB color support. Setting the color for each pixel manually would require updating 38,400 pixels per frame, or 2,304,000 pixels per second at 60 fps.
With a 16 MHz processor, this means calculating 1 pixel every 8 clock cycles, which is pretty much impossible.
The Game Boy Advance provides two ways to easily put pixels on the screen: tiles and sprites.

Tiles are 8x8 pixels in size and can be placed in a grid on the screen.
You can also scroll the whole tile layer to arbitrary positions, but the tiles will remain in this 8x8 pixel grid.

Sprites are the other way to draw things on the screen, which we'll cover in this section.
The Game Boy Advance supports 256 hardware sprites, with different sizes ranging from square 8x8 to more exotic sizes like 8x32 pixels.
In our pong game, all the sprites will be 16x16 pixels to make things simpler.

Sprites are stored in a special area of video memory called the 'Object Attribute Memory' (OAM).
OAM has space for the 'attributes' of the sprites, such as their location, whether or not they are visible, and which tile to use, but it does not store the actual pixel data.
The pixel data is stored in video RAM (VRAM).
This split allows multiple sprites to refer to the same tiles in VRAM, which saves space and allows for more objects on screen than would be possible by repeating them.

Since RAM is in short supply and expensive, the tile data is stored as indexed palette data.
Instead of storing the full color data for each pixel in the tile, the Game Boy Advance stores a 'palette' of colors, and the tiles that make up the sprites are stored as indexes to the palette.
Each sprite can use a maximum of 16 colors out of the total sprite palette of 256 colors.

There are technically two types of sprites: regular and affine sprites.
For now, we will only be dealing with regular sprites.

# Import the sprite

Firstly, you're going to need to import the sprites into your project.
`agb` has excellent support for the [aseprite](https://www.aseprite.org/) sprite editor which can be bought for $20 or you can compile it yourself for free.
Aseprite files can be natively imported by `agb` for use on the Game Boy Advance.
Here is the sprite sheet we will use as a png, but you should [download the aseprite file](sprites.aseprite) and place it in `gfx/sprites.aseprite`.

![pong sprites](sprites.png)

This contains 5 `16x16px` sprites: the end cap for the paddle, the center part of the paddle, which could potentially be repeated a few times, and the ball with various squashed states.
The aseprite file defines tags for these sprites: "Paddle End," "Paddle Mid," and "Ball."

```rust
use agb::include_aseprite;

// Import the sprites in to this static. This holds the sprite
// and palette data in a way that is manageable by agb.
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);
```

This uses the `include_aseprite` macro to include the sprites in the given aseprite file.
Now, let's put this on screen by firstly creating the object manager and then creating an object, this will also involve the creation of the main entry function using the `entry` macro.
The signature of this function takes the `Gba` struct and has the never return type, this means Rust will enforce that this function never returns, for now we will achieve this using a busy loop.
Using the `Gba` struct we get the [`Graphics`](https://docs.rs/agb/latest/agb/display/struct.Graphics.html) we get a [`GraphicsFrame`](https://docs.rs/agb/latest/agb/display/struct.GraphicsFrame.html) and pass it to `.show`.

```rust
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Get the graphics manager, responsible for all the graphics
    let mut gfx = gba.graphics.get();

    // Create an object with the ball sprite
    let mut ball = Object::new(sprites::BALL.sprite(0));

    // Place this at some point on the screen, (50, 50) for example
    ball.set_pos((50, 50));

    // Start a frame and add the one object to it
    let mut frame = gfx.frame();
    ball.show(&mut frame);
    frame.commit();

    loop {}
}
```

If you run this you should now see the ball for this pong game somewhere in the top left of the screen.

# Making the sprite move

The GBA renders to the screen one pixel at a time a line at a time from left to right.
After it has finished rendering to each pixel of the screen, it briefly pauses rendering before starting again.
This period of no drawing is called the 'vertical blanking interval' which is shortened to `vblank`.
There is also a 'horizontal blanking interval', but that is outside of the scope of this tutorial.

The `frame.commit()` method automatically waits for this `vblank` state before rendering your sprites to avoid moving a sprite during the rendering which could cause tearing of your objects[^hblank].

Making the sprite move 1 pixel every frame (so 60 pixels per second) can be done as follows:

```rust
// replace the loop with this

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
    ball.set_pos((ball_x, ball_y));

    // prepare the frame
    let mut frame = gfx.frame();
    ball.show(&mut frame);

    frame.commit();
}
```

# What we did

In this section, we covered why sprites are important, how to create and manage them using the `Frame` in `agb` and make a ball bounce around the screen.

[^hblank]: Timing this can give you some really cool effects allowing you to push the hardware.
    `agb` provides support for this by using `dma`, this is an advanced technique that is out of scope of this tutorial.
