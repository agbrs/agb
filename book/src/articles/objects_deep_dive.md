# Objects deep dive

An object is a sprite drawn to an arbitrary part of the screen.
They are typically used for anything that moves such as characters and NPCs.
All objects can be flipped and affine objects can have an affine transformation applied to them that can rotate and scale them.

## Importing sprites

Sprites are imported from aseprite files.
Aseprite is an excellent pixel sprite editor that can be acquired for low cost or compiled yourself for free.
It also provides features around adding tags for grouping sprites to form animations and dictating how an animation is performed.
[The aseprite documentation contains detail on tags.](https://www.aseprite.org/docs/tags/)
This makes it very useful for creating art for use by agb.

You can import 15 colour sprites using the [`include_aseprite`](https://docs.rs/agb/latest/agb/macro.include_aseprite.html) macro.
In a single invocation of [`include_aseprite`](https://docs.rs/agb/latest/agb/macro.include_aseprite.html) the palettes are all optimised together.
For example, you might use the following to import some sprites.

```rust
agb::include_aseprite!(mod sprites, "sprites.aseprite", "other_sprites.aseprite");
```

This will create a module called `sprites` that contains `static`s corresponding to every tag in the provided aseprite files as an agb [`Tag`](https://docs.rs/agb/latest/agb/display/object/struct.Tag.html).

You can import 255 colour sprites using the [`include_aseprite_256`](https://docs.rs/agb/latest/agb/macro.include_aseprite_256.html) macro similarly.
You have 15 colour and 255 colours because the 0th index of the palette is always fully transparent.

## ROM and VRAM

Sprites must be in VRAM to be displayed on screen.
The [`SpriteVram`](https://docs.rs/agb/latest/agb/display/object/struct.SpriteVram.html) type represents a sprite in VRAM.
This implements the `From` trait from [`&'static Sprite`](https://docs.rs/agb/latest/agb/display/object/struct.Sprite.html).
The `From` implementation will deduplicate the sprites in VRAM, this means that you can repeatedly use the same sprite and it won't blow up your VRAM usage.
This deduplication does have the performance implication of requiring a HashMap lookup, although for many games this will a rather small penalty.
By storing and reusing the [`SpriteVram`](https://docs.rs/agb/latest/agb/display/object/struct.SpriteVram.html) you can avoid this lookup.
Furthermore, [`SpriteVram`](https://docs.rs/agb/latest/agb/display/object/struct.SpriteVram.html) is reference counted, so `Clone`ing it is cheap and doesn't allocate more `VRAM`.

```rust
use agb::display::object::SpriteVram;

agb::include_aseprite!(mod sprites, "examples/chicken.aseprite");

let sprite = SpriteVram::from(sprites::IDLE.sprite(0));
let clone = sprite.clone();
```

## Regular objects

When you have a sprite, you will want to display it to the screen.
This is an [`Object`](https://docs.rs/agb/latest/agb/display/object/struct.Object.html).
Like many things in agb, you can display to the screen using the `show` method on [`Object`](https://docs.rs/agb/latest/agb/display/object/struct.Object.html) on the frame.

```rust
use agb::display::{GraphicsFrame, object::Object};

agb::include_aseprite!(mod sprites, "examples/chicken.aseprite");

fn chicken(frame: &mut GraphicsFrame) {
    // Object::new takes anything that implements Into<SpriteVram>, so we can pass in a static sprite.
    Object::new(sprites::IDLE.sprite(0))
        .set_pos((32, 32))
        .set_hflip(true)
        .show(frame);
}
```

## Affine objects

Affine objects can be rotated and scaled by an affine transformation.
These objects are created using the [`ObjectAffine`](https://docs.rs/agb/latest/agb/display/object/struct.ObjectAffine.html) type.
This, like an [`Object`](https://docs.rs/agb/latest/agb/display/object/struct.Object.html), requires a sprite but also requires an [`AffineMatrixObject`](https://docs.rs/agb/latest/agb/display/object/struct.AffineMatrixObject.html) and an `AffineMode`.
The affine matrix object can be thought of as an affine matrix stored in oam.

The [affine article](./affine.md) goes over some detail in how to create affine matrices.
With a given affine matrix, you can use `AffineMatrixObject::new` or the `From` impl to create an [`AffineMatrixObject`](https://docs.rs/agb/latest/agb/display/object/struct.AffineMatrixObject.html).

When using the same affine matrix for multiple sprites, it is important to reuse the `AffineMatrixObject` as otherwise you may run out of affine matrices.
You can use up to 32 affine matrices at once.
`AffineMatrixObject` implements `Clone`, and cloning is very cheap as it just increases a reference count.

An `AffineMatrix` also stores a translation component.
However, creating the `AffineMatrixObject` will lose this translation component, so you'll also need to set it as the position as follows:

```rust
let affine_matrix = calculate_affine_matrix();
let affine_matrix_instance = AffineMatrixObject::new(affine_matrix);

ObjectAffine::new(sprite, affine_matrix_instance, AffineMode::Affine)
    .set_position(affine_matrix.position().round())
    .show(frame);
```

Be aware that the position of an affine object is the centre of the sprite, and not the top left corner like it is for regular sprites.

Affine objects have two display modes, the regular and the double modes.
The double mode allows for the sprite to be scaled to twice the size of the original sprite while the single would cut off anything outside of the regular bounding box.
You can see the behaviour in the [affine objects example](https://agbrs.dev/examples/affine_objects).

## Dynamic sprites

A dynamic sprite is a sprite whose data is defined during runtime rather than at compile time.
Agb has two kinds of dynamic sprites: [`DynamicSprite16`](https://docs.rs/agb/latest/agb/display/object/struct.DynamicSprite16.html) and [`DynamicSprite256`](https://docs.rs/agb/latest/agb/display/object/struct.DynamicSprite256.html).
These are naturally for sprites that use a single palette and those that use multiple.

The easiest way to create a dynamic sprite is through the relevant type, for example here is creating a [`DynamicSprite16`](https://docs.rs/agb/latest/agb/display/object/struct.DynamicSprite16.html) and setting a couple of pixels.

```rust
use agb::display::object::{DynamicSprite16, Size};

let mut sprite = DynamicSprite16::new(Size::S8x8);

sprite.set_pixel(4, 4, 1);
sprite.set_pixel(5, 5, 1);

let in_vram = sprite.to_vram(todo!());
```

And you could then go on to use the sprite however you like with [`Object`](https://docs.rs/agb/latest/agb/display/object/struct.Object.html) as normal.

## How to handle the camera position?

In many games, you will have objects both in screen space and in world space.
You will find that to correctly draw objects to the screen you will need to convert world space coordinates to screen spaces coordinates before showing it.
The position of your "camera" needs to be propagated to where the object is shown.
There are many ways of achieving this, the simplest being wherever you create your object you can pass through a camera position to correct the position

```rust
use agb::{
    fixnum::{Vector2D, Num},
    display::{
        GraphicsFrame,
        object::Object,
    },
};

struct MyObject {
    position: Vector2D<Num<i32, 8>>
}

impl MyObject {
    fn show(&self, camera: Vector2D<Num<i32, 8>>, frame: &mut GraphicsFrame) {
        Object::new(SOME_SPRITE).set_pos((self.position - camera).round()).show(frame);
    }
}
```

While you can get the position of an [`Object`](https://docs.rs/agb/latest/agb/display/object/struct.Object.html), do not try using this to correct for the camera position as it will not work.
The precision that positions are stored in the [`Object`](https://docs.rs/agb/latest/agb/display/object/struct.Object.html) are enough to be displayed to the screen and not much more.
Trying to use this for world coordinates will fail.

## See also

- The pong tutorial goes over the [basics of sprites](../pong/03_sprites.md).
