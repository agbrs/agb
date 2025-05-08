# Fixed point numbers

When writing games you'll often find yourself needing to represent fractional numbers.
For example your player might not want to accelerate at whole numbers of pixels per second.
Commonly on desktop computers and modern games consoles, to do this you would use
[floating point numbers](https://en.wikipedia.org/wiki/Floating-point_arithmetic).
However, floating point numbers are complex and require special hardware to work efficiently.
The Game Boy Advance doesn't have a [floating point unit](https://en.wikipedia.org/wiki/Floating-point_unit)
(FPU) so any use of floating point numbers in your game will have to be emulated in software.
This is very slow, and will reduce the speed of your game to a crawl.

The solution to this problem is to have a fixed point number rather than a floating point number.
These store numbers with a fixed number of bits of precision instead of allowing the precision to vary.
While you lose the ability to store very small and very large numbers, arithmetic operations like `+` and `*` become as fast (or almost as fast) as working with integers.

Storing the coordinates as a fixed point integer is often referred to as the `sub-pixel value` in retro games communities.
But this book refers to them as 'fixed point integers' or 'fixnums'.

## Fixnums - `Num<T, N>`

The main type for interacting with fixnums is the [`Num<T, N>`](https://docs.rs/agb/latest/agb/fixnum/struct.Num.html) struct found in `agb::fixnum::Num`.
`T` is underlying integer type for your fixed point number (e.g. `i32`, `u32`, `i16`) and `N` is how many bits you want to use for the fractional component.
So you'll often see it written as something like `Num<i32, 8>`.

We recommend using `i32` (or `u32` if you never need the number to be negative) as the primitive integer type unless you have good reason not to.

It is harder to provide general advice for how many fractional bits you will need.
The larger `N` is, the more precise your numbers can be but it also reduces the maximum possible value.
The smallest positive number that can be represented for a given `N` will be `1 / 2^N`, and the maximum number will be `type::MAX / 2^N`.
You should use an `N` that is less than or equal to half the number of bits in the underlying integer type, so for an `i32` you should use an `N` of _at most_ `16`.

In general, `8` bits offers a good middle ground, allowing for `1/256` precision while still allowing for a range of `-8388608..8388608` with `i32`.

The original Super Mario Bros has 16 sub pixels, which would correspond to an `N` of `4`.
That was designed to run on a 8-bit processor, so you might as well use a few more.

### Creating fixnums in code

#### The `num!` macro

The `num!` macro is useful if you want to represent your fixnum as a floating point number in your code.
You'll often see it when you want to pretend that the code you're working with is actually working as a floating point.
For example:

```rust
use agb::fixnum::num;

fn do_some_calculation(input: Num<i32, 8>) -> Num<i32, 8> {
    input * num!(1.4)
}
```

will multiply the input by 1.4 (as a fixnum value).

You must specify the type being produced, or it must be inferred elsewhere.
So in the example above, it is inferred that the type is `Num<i32, 8>` but the following example is incorrect:

```rust,compile_fail
# use agb::fixnum::num;
#
let jump_speed = num!(1.5); // this is incorrect
```

Instead, you should specify the type:

```rust
# use agb::fixnum::{num, Num};
#
let jump_speed: Num<i32, 8> = num!(1.5); // this is now correct
```

#### `Num::new()`

You can also call the `new()` method on `Num` which accepts an integer.
This is mainly useful if you want a fixnum with that value.

```rust
# use agb::fixnum::Num;
let jump_speed: Num<i32, 8> = Num::new(5);
```

Note that you will either need to have the number of fractional bits be inferred, or use the turbo fish operator

```rust
# use agb::fixnum::Num;
let jump_speed = Num::<i32, 8>::new(5);
```

#### `.into()`

You can also create integer valued fixnums using `.into()`.

```rust
# use agb::fixnum::Num;
// these are equivalent
let jump_speed: Num<i32, 8> = 5.into();
let jump_speed: Num<i32, 8> = Num::from(5);
```

### Arithmetic

Fixnums can be used in arithmetic in the ways you would expect.
They also work with integers directly, so the following examples are all valid:

```rust
# use agb::fixnum::{num, Num};
let speed: Num<i32, 8> = num!(5);
let distance: Num<i32, 8> = num!(1);

let position = distance + speed * 3;
let position = distance + speed * num!(1.5);
```

There are many more useful methods on `Num` (like `.sqrt()`, `.abs()`), so check out the
[documentation](https://docs.rs/agb/latest/agb/fixnum/struct.Num.html) for the full list.

### Division with fixnums

The `Num` type supports standard division using the `/` operator with both other fixnums, or integers.

```rust
 use agb::fixnum::{Num, num};

let distance: Num<i32, 8> = num!(25.0);
let time: Num<i32, 8> = num!(2.0);
let speed = distance / time;

agb::println!("Distance: {}", distance);
agb::println!("Time: {}", time);
agb::println!("Speed: {}", speed);

let scaled_distance = distance / 4; // Dividing by an integer
agb::println!("Distance divided by 4: {}", scaled_distance);
```

When performing division, it is most efficient to do the division with a power of 2 (2, 4, 8, 16, 32, ...) rather than any other value.
Division by a power of 2 will be optimised by the rust compiler to a simple shift, whereas other constants are much more computationally intensive.
It is okay to do a few divisions by non-power of 2 values per frame, but it is around 100 times less efficient, so keep them to a small number per frame.

Therefore, when designing your game logic, especially performance-critical sections that many times every frame, try to structure calculations such that divisions are primarily done using powers of two.
For instance, if you need to scale a value by a factor that isn't a power of two, consider whether you can achieve a similar effect by dividing by a near power of 2 (so if you wanted to divide by 10, can you get away with dividing by 8 instead?).
You can also multiply by the inverse if the precision is okay. Rather than dividing by `10`, instead multiply by `num!(0.1)`.

## Bridging back into integers

Often you'll have something calculated using fixed points (like the position of a player character) and you'll want to now display something on the screen.
Because the screen works in whole pixel coordinates, you'll need to convert your fixnums into integers.
The best method for this is the [`.round()`](https://docs.rs/agb/latest/agb/fixnum/struct.Num.html#method.round) method because it has better behaviour when approaching the target integer.

## `Vector2D` and `Rect`

In addition to the `Num` type, `agb::fixnum` also includes a few additional types which will be useful in many applications.
`Vector2D<T>` works with both `Num` and primitive integer types and provides a 2 dimensional vector.
`Rect<T>` similarly works with both `Num` and primitive integer types and represents an axis aligned rectangle.

### `Vector2D<T>`

`Vector2D` can be used to represent positions, velocities, points etc.
It implements the arithmetic operations addition and multiplication by a constant, allowing you to write simpler code when dealing with 2d coordinates.

The main way to construct a `Vector2D<T>` is via the `vec2` helper method in `agb::fixnum::vec2`, but `Vector2D` also implements `From<(T, T)>` which means that you can pass 2-tuples to methods which require `impl Into<Vector2D<T>>`.

As an example, here is some code for calculating the final location of an object at a time `time` given a starting location and a velocity.

```rust
use agb::fixnum::{Num, num, Vector2D, vec2};

fn calculate_position(
    initial_position: Vector2D<Num<i32, 8>>,
    velocity: Vector2D<Num<i32, 8>>,
    time: Num<i32, 8>
) -> Vector2D<Num<i32, 8>> {
    initial_position + velocity * time
}

assert_eq!(
    calculate_position(
        vec2(num!(5), num!(5)), // you can use the vec2 constructor
        (num!(0.5), num!(-0.5)).into(), // or `.into()` on 2-tuples
        num!(10)
    ),
    vec2(num!(10), num!(0))
);
```

Like `Num`, `Vector2D` also provides the [`.round()`](https://docs.rs/agb/lastest/agb/fixnum/struct.Vector2D.html#method.round) which is the method you should use if converting back into integer coordinates (like setting the position of an object given fixnum positions before).

See the [`Vector2D` documentation](https://docs.rs/agb/latest/agb/fixnum/struct.Vector2D.html) for more details.

### `Rect<T>`

`Rect<T>` is an axis aligned rectangle that can be used to represent hit boxes.
It is represented as a position and a size.

For the purpose of hit boxes, the most useful method is the `touches` method that is true if the two rectangles are overlapping.

```rust
use agb::fixnum::{Rect, vec2};

let r1 = Rect::new(vec2(1, 1), vec2(3, 3));
let r2 = Rect::new(vec2(2, 2), vec2(3, 3));
let r3 = Rect::new(vec2(-10, 2), vec2(3, 3));

assert!(r1.touches(r2));
assert!(!r1.touches(r3));
```

See the [`Rect` documentation](https://docs.rs/agb/latest/agb/fixnum/struct.Rect.html) for more details.

## See also

- [Agb fixnum documentation](https://docs.rs/agb/latest/agb/fixnum/index.html)
- [Tonc's fixed point article](https://gbadev.net/tonc/fixed.html)
- [Wikipedia](https://en.wikipedia.org/wiki/Fixed-point_arithmetic)
