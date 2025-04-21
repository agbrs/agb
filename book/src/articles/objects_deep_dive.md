# Objects deep dive

An object is a sprite drawn to an arbitrary part of the screen.
It may be flipped and even scaled and rotated for affine objects.

The pong tutorial goes over the [basics of sprites](../pong/03_sprites.md).

## Regular objects

## Affine objects

Affine objects can be rotated and scaled by an affine transformation.
These objects are created using the `ObjectAffine` type.
This like an `Object` requires a sprite but also requires an `AffineMatrixInstance` and an `AffineMode`.
The affine matrix instance can be thought of as an affine matrix stored in oam.

The `affine` module goes over some detail in how to create affine matricies, the relevant part is `AffineMatrix::to_object_wrapping` which creates an `AffineMatrixObject` that is suitable for use in objects which then has the `oam` version of `AffineMatrixInstance`.
When using a single affine matrix for multiple sprites, it is imperetive that you reuse the `AffineMatrixInstance` as otherwise you may run out of affine matricies.

## Dynamic sprites


A dynamic sprite is a sprite whose data is defined during runtime rather than at compile time.
Agb has two kinds of dynamic sprites: `DynamicSprite16` and `DynamicSprite256`.
These are naturally for sprites that use a single palette and those that use multiple.

The easiest way to create a dynamic sprite is through the relevant type, for example here is creating a `DynamicSprite16` and setting a couple of pixels.

```rust
use agb::display::object::{DynamicSprite16, Size};

let mut sprite = DynamicSprite16::new(Size::S8x8);

sprite.set_pixel(4, 4, 1);
sprite.set_pixel(5, 5, 1);

let in_vram = sprite.to_vram(todo!());
```

And you could then go on to use the sprite however you like with `Object` as normal.

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
        Object::new(SOME_SPRITE).set_position((self.position - camera).round()).show(frame);
    }
}
```

While you can get the position of an `Object`, do not try using this to correct for the camera position as it will not work.
The precision that positions are stored in the `Object` are enough to be displayed to the screen and not much more.
Trying to use this for world coordinates will fail.