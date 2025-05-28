# Miscellaneous

This article covers topics which aren't big enough to be their own section, but are worth covering in a smaller section here.

# Printing to the console

If your game is running under the [mGBA](https://mgba.io) emulator then you can print to the console using the [`agb::println!`](https://docs.rs/agb/latest/agb/macro.println.html) macro.
So to print the text `Hello, World!`, you would do the following:

```rust
agb::println!("Hello, World!");
```

The `println!` macro works the same way as the standard library `println!` macro.
However, you should note that formatting arguments is quite slow, so the main use for this is debugging, and you'll probably want to remove them when actually releasing your game as they can make it so that your game logic doesn't calculate within a frame any more.

# Random numbers

`agb` provides a simple random number generator in [`agb::rng`](https://docs.rs/agb/latest/agb/rng/index.html).
To generate a random number, you can either create your own instance of a `RandomNumberGenerator`, or use the global `next_i32()` method.

The Game Boy Advance has no easy way of seeding the random number generator, and creating random numbers which are different for each boot can be quite difficult.
One thing you can do to make random numbers harder to predict is to call the `next_i32()` method once per frame.

```rust
loop {
    let mut frame = gfx.frame();
    // do your game rendering

    frame.commit();

    // make the random number generator harder to predict
    let _ = agb::rng::next_i32();
}
```

# HashMaps

`alloc` does not provide a `HashMap` implementation, and although you can import the `no_std` version of [`hashbrown`](https://crates.io/crates/hashbrown), it can be quite slow on the Game Boy Advance due to its use of simd and other intrinsics which have to be emulated in software.

Therefore, `agb` provides its own `HashMap` and `HashSet` implementations which are optimised for use on 32-bit embedded devices which you can use from the [`agb::hash_map`](https://docs.rs/agb/latest/agb/hash_map/index.html) module.
These work exactly like the `HashMap` from the standard library, providing most of the same API, with only a few omissions for rarely used methods or ones which don't make sense with the different backing implementation.

# Allocators

By default, all allocations in `agb` go into the more plentiful, but slower `EWRAM` (Extended Working RAM).
`EWRAM` is 256kB in size, which is normally more than enough for even a moderately complex game.
Reading and writing to `EWRAM` takes 3 CPU cycles per access, so is fairly slow however.

If you have a collection you're reading and writing to a lot, you can instead move it to the faster, but smaller `IWRAM` (Internal Working RAM).
`IWRAM` is only 32kB in size and is used for the stack, along with some high performance code, so you should be careful with what you put there.
However, it does only take 1 CPU cycle to read or to write to, so it can be noticeably faster to use.

To allocate in `IWRAM`, use the `new_in()` methods on `Vec`, `Box` or `HashMap`.

```rust
use agb::InternalAllocator;
use alloc::vec::Vec;

let mut v = Vec::new_in(InternalAllocator);
v.push("Hello, World!");
```
