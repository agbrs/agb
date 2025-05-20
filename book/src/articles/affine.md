# Affine backgrounds and objects

The Game Boy Advance can perform basic transformations like rotation and scaling to backgrounds and objects before they are displayed on screen.
These transformations are used to perform many of the graphical tricks which give Game Boy Advance games their unique aesthetic.

> Note that this article will assume familiarity with vectors and matrix mathematics.
> If these are new to you, you should first understand the basics of [linear algebra](https://www.3blue1brown.com/topics/linear-algebra) before delving much deeper into this topic.

## Game Boy Advance affine transformations

The transformations supported by the Game Boy Advance are [affine transformations](https://en.wikipedia.org/wiki/Affine_transformation)[^affine-cheat].
The technical definition is that affine transformations preserve lines and parallelism.
However, it might be easiest to think of affine transformations as any combination of the following transformations:

| Transformation | Description                            |
| -------------- | -------------------------------------- |
| Translation    | Moving the item                        |
| Scaling        | Changing the size of the item          |
| Rotation       | Rotating the item                      |
| Shear          | Turning rectangles into parallelograms |

[^affine-cheat]:
    There are tricks you can use to beat the affine transformations and have non-affine transformations.
    The [3d plane](https://agbrs.dev/examples/dma_effect_affine_background_3d_plane) and [pipe background](https://agbrs.dev/examples/dma_effect_affine_background_pipe) examples show what you can do if you change the transformation matrix on every single scanline.

You can see all these transformations in action in the [affine transformations](https://agbrs.dev/examples/affine_transformations) example.

> The most important thing to note about transformation matrices in the Game Boy Advance is that they are inverted.

What we mean by this is that if you're looking to double the size of an object, you may construct a matrix like the following:

\\[
\begin{pmatrix}
2 & 0 \\\\
0 & 2
\end{pmatrix}
\\]

However, if you do this, your object will actually shrink to half its size.
This is because rather than mapping object locations to screen locations, the Game Boy Advance instead uses the transformation matrices to map screen locations to object locations.
So in order to double the size of the object, you'll need to use a matrix with `0.5` in the diagonal.

`agb` does not hide this from you and automatically invert the matrices because inverting a matrix in fixed point numbers still involves a division and can lose quite a lot of precision.
So you need to make sure that any matrix you pass is thought of as mapping pixels on the screen to the location in your object / background rather than from your object / background to pixels on the screen.

## Affine matrices in `agb`

The key affine matrix type provided by `agb` is the [`AffineMatrix`](https://docs.rs/agb/latest/agb/display/struct.AffineMatrix.html).
This represents the full affine transformation including translation, and provides a multiplication overload which you use to combine transformations.

For example, if we want to do both a rotation and a scale, you could use something like this:

```rust
use agb::{
    display::AffineMatrix,
    fixnum::{Num, num}
};

let rot_mat: AffineMatrix<Num<i32, 8>> =
    AffineMatrix::from_rotation::<8>(num!(0.25));
let scale_mat: AffineMatrix<Num<i32, 8>> =
    AffineMatrix::from_scale(vec2(num!(0.5), num!(0.5)));

let final_transform: AffineMatrix<Num<i32, 8>> = rot_mat * scale_mat;
```

Remember that the transform is transforming _screen_ coordinates to _object_ coordinates.
So this will first halve the size of the screen and then rotate it (effectively showing it at double the size).

You can construct an `AffineMatrix` for each of the basic transformations above.
All the fields are `pub`, so you can also construct one using:

```rust
use agb::display::AffineMatrix;

let mat = AffineMatrix {
    a, b, c, d, x, y
};
```

which is the matrix:

\\[
\begin{pmatrix}
a & b & x \\\\
c & d & y \\\\
0 & 0 & 0
\end{pmatrix}
\\]

## Affine backgrounds

To create affine backgrounds, please see the relevant section in the [backgrounds deep dive](./backgrounds.md#affine-backgrounds).

You can apply a transformation matrix to an affine background using the [`.set_transform()`](https://docs.rs/agb/latest/agb/display/tiled/struct.AffineBackground.html#method.set_transform) method and passing in the desired affine matrix.
`set_transform()` takes an [`AffineMatrixBackground`](https://docs.rs/agb/latest/agb/display/tiled/struct.AffineMatrixBackground.html) rather than an `AffineMatrix` directly because they have different size requirements.

You can convert from an `AffineMatrix` to an `AffineMatrixBackground` by using the `from_affine()` constructor or the `.into()` method.

## Affine objects

Affine objects behave slightly differently to backgrounds.
They only use the `a`, `b`, `c` and `d` components of the matrix and ignore the transformation part of it.
So you'll also need to set the position of the sprite separately.

```rust
let affine_matrix = calculate_affine_matrix();
let affine_matrix_instance = AffineMatrixObject::new(affine_matrix);

ObjectAffine::new(sprite, affine_matrix_instance, AffineMode::Affine)
    .set_position(affine_matrix.position().round())
    .show(frame);
```

See the [affine section](./objects_deep_dive.md#affine-objects) of the object deep dive for more details.
