# Sprites

In this section, we'll put the sprites needed for our pong game onto the screen.

# Import the sprite

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