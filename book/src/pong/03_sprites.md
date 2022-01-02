# Sprites

In this section, we'll put the sprites needed for our pong game onto the screen.
We'll cover what sprites are in the Game Boy Advance, and how to get them to show up on screen.
We'll briefly cover vblank and by the end of this section, you'll have a ball bouncing around the screen!

# Why do we need sprites in the first place?

The Game Boy Advance has a 240x160px screen, with 15-bit RGB colour support.
In order to manually set the colour for each pixel in the screen, you would need to update a total of 38,400 pixels per frame, or 2,304,000 pixels per second at 60 fps.
With a 16MHz processor, that means you would need to be able to calculate 1 pixel every 8 clock cycles, which is pretty much impossible.
You could get clever with how you update these pixels, but there is a much easier way which almost every game for the Game Boy Advance uses.

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

As mentioned above, we'll need to convert the sprite data into a format that the Game Boy Advance will be able to understand (so palette information and tile data).
Once we've converted it, we'll need to import this tile data into the Game Boy Advance's memory on start up and then create a sprite in the OAM.

Firstly, you're going to need to import the sprites into your project.
Save the following image into a new folder called `gfx` in your project:

![pong sprites](sprites.png)

This contains 5 `16x16px` sprites.
The first is the end cap for the paddle.
The second is the centre part of the paddle, which could potentially be repeated a few times.
The third until the fifth is the ball, with various squashed states.
The background is a lovely shade of `#ff0044` which we will use for the transparency.

`agb` needs to know what the tile size is, and also what the transparent colour is so that it can properly produce data that the Game Boy Advance can understand.
So you need to create a manifest file for the image, which is declared in a toml file.

In the same `gfx` folder as the `sprites.png` file, also create a `sprites.toml` file with the following content:

```toml
version = "1.0"

[image.sprites]
filename = "sprites.png"
tile_size = "16x16"
transparent_colour = "ff0044"
```

Now let's create a module in the `main.rs` file which imports the sprite sheet and loads it into memory.
Anything sprite related is managed by the [`ObjectControl` struct](https://docs.rs/agb/0.8.0/agb/display/object/struct.ObjectControl.html).
So we use that to load the sprite tile map and palette data.

```rust
// Put all the graphics related code in the gfx module
mod gfx {
    use agb::display::object::ObjectControl;

    // Import the sprites into this module. This will create a `sprites` module
    // and within that will be a constant called `sprites` which houses all the
    // palette and tile data.
    agb::include_gfx!("gfx/sprites.toml");

    // Loads the sprites tile data and palette data into VRAM
    pub fn load_sprite_data(object: &mut ObjectControl) {
        object.set_sprite_palettes(sprites::sprites.palettes);
        object.set_sprite_tilemap(sprites::sprites.tiles);
    }
}
```

This uses the `include_gfx!` macro which loads the sprite information file and grabs the relevant tile data from there.

Now, let's put this on screen by firstly creating the object manager and loading the sprite data from above:

```rust
let mut gba = Gba::new();

// set background mode to mode 0 (we'll cover this in more detail later)
// for now, this is required in order for anything to show up on screen at all.
let _tiled = gba.display.video.tiled0();

// Get the OAM manager
let mut object = gba.display.object.get();
gfx::load_sprite_data(&mut object);
object.enable();
```

Then, let's create the ball object and put it at 50, 50 on the screen:

```rust
// continued from above:
let mut ball = object.get_object_standard(); // (1)

ball.set_x(50)                               // (2)
    .set_y(50)
    .set_sprite_size(Size::S16x16)
    .set_tile_id(4 * 2)
    .show();

ball.commit();                               // (3)
```

There are a few new things introduced here, so lets go through these 1 by 1.

1. The first thing we're doing is creating a 'standard' object.
These are of non-affine type.
2. We now set some properties on the ball.
The position and size are self explanatory.
Interesting here is that the tile id is 4 * 2.
This is because the tile id is calculated in 8x8 pixel chunks, and in our example we have 16x16 sprites so each sprite takes 4 tiles and this is the tile in position 2 on the tilesheet above (0 indexed).
The final call to `.show()` will make it actually visible as sprites are hidden by default.
3. The call to `.commit()` actually makes the change to object memory.
Until `.commit()` is called, no changes you made will actually be visible.
This is very handy because we might want to change sprite positions etc while the frame is rendering, and then move them in the short space of time we have before the next frame starts rendering (vblank).

If you run this you should now see the ball for this pong game somewhere in the top left of the screen.

# Making the sprite move

As mentioned before, you should `.commit()` your sprites only during `vblank` which is the (very short) period of time nothing is being rendered to screen.
`agb` provides a convenience function for waiting until this happens called `agb::display::busy_wait_for_vblank()`.
You shouldn't use this is a real game (we'll do it properly later on), but for now we can use this to wait for the correct time to `commit` our sprites to memory.

Making the sprite move 1 pixel every frame (so approximately 60 pixels per second) can be done as follows:

```rust
// replace the call to ball.commit() with the following:

let mut ball_x = 50;
let mut ball_y = 50;
let mut x_velocity = 1;
let mut y_velocity = 1;

loop {
    ball_x = (ball_x + x_velocity).clamp(0, agb::display::WIDTH - 16);
    ball_y = (ball_y + y_velocity).clamp(0, agb::display::HEIGHT - 16);

    if ball_x == 0 || ball_x == agb::display::WIDTH - 16 {
        x_velocity = -x_velocity;
    }

    if ball_y == 0 || ball_y == agb::display::HEIGHT - 16 {
        y_velocity = -y_velocity;
    }

    ball.set_x(ball_x as u16).set_y(ball_y as u16);

    agb::display::busy_wait_for_vblank();
    ball.commit();
}
```

# What we did

In this section, we covered why sprites are important, how to create and manage them using the `ObjectControl` in `agb` and make a ball bounce around the screen.