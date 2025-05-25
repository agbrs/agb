# Unit tests

It is possible to write unit tests using `agb`.

## Installing the `mgba-test-runner` (technically optional)

Firstly you'll need to install the `mgba-test-runner` which requires `cmake` and `libelf` installed via whichever mechanism you use to manage software, along with a C and C++ compiler.

Then run

```sh
cargo install --git https://github.com/agbrs/agb.git mgba-test-runner
```

## Running unit tests

Running just `cargo test` will launch the test in `mgba`, which isn't particularly useful.
To run the test using the test runner installed in step 1, use the following command:

```sh
CARGO_TARGET_THUMBV4T_NONE_EABI_RUNNER=mgba-test-runner cargo test
```

## Writing unit tests

If you don't already have this in your `main.rs` file from the template, you need the following at the top to enable the custom test framework.

```rust
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
```

Then, you can write tests using the `#[test_case]` attribute:

```rust
#[test_case]
fn dummy_test(_gba: &mut Gba) {
    assert_eq!(1, 1);
}
```

Tests take a mutable reference to the `Gba` struct.
There is no equivalent to `#[should_panic]` so these style of tests are not (currently) possible to write using `agb`.

If you want to check that the screen has pixels as expected, you can use the [`assert_image_output`](https://docs.rs/agb/latest/agb/test_runner/fn.assert_image_output.html) method.
This only works when running the unit test under the `mgba-test-runner` installed in step 1.

```rust
#[test_case]
fn test_background_shows_correctly(gba: &mut Gba) {
    // you should never assume that the palettes are set up correctly in a test
    VRAM_MANAGER.set_background_palettes(...);

    let mut gfx = gba.graphics.get();
    let mut frame = gfx.frame();
    show_some_background(&mut frame);
    frame.commit();

    assert_image_output("src/tests/test_background_shows_correctly.png");
}
```

If the mentioned `png` file doesn't exist, then the `mgba-test-runner` will create the file.
Subsequent runs will compare the current screen with the expected result and fail the test if it doesn't match.

## Interpreting test output

When running unit tests, you'll get output that looks similar to this:

```
agb::display::blend::test::can_blend_affine_backgrounds...[ok: 1757950c ≈ 0.1s ≈ 625% frame]
agb::display::blend::test::can_blend_affine_object_to_black...[ok: 1656122c ≈ 0.1s ≈ 589% frame]
agb::display::blend::test::can_blend_object_shape_to_black...[ok: 1392767c ≈ 0.08s ≈ 496% frame]
agb::display::blend::test::can_blend_object_to_black...[ok: 1400985c ≈ 0.08s ≈ 498% frame]
agb::display::blend::test::can_blend_object_to_white...[ok: 1401004c ≈ 0.08s ≈ 498% frame]
```

It starts with the test name, and then lists something like `[ok: 1757950c ≈ 0.1s ≈ 625% frame]`.
This means it took `1,757,950` CPU cycles to run the test, or about `0.1` seconds or `6.25` frames.

Any test which uses `assert_image_output()` will automatically take a few frames.

## Doc tests

To write a doctest for your game in `agb`, create a function and mark it with the [`#[agb::doctest]`](https://docs.rs/agb/latest/agb/attr.doctest.html) attribute macro.

````rust
/// This is a cool rust function with some epic documentation which is checked
/// at compile time and the doctest will run when running the tests.
///
/// ```rust
/// # #![no_std]
/// # #![no_main]
/// #
/// # #[agb::doctest]
/// # fn test(gba: agb::Gba) {
/// assert_eq!(my_crate::my_cool_function(), 7);
/// # }
/// ```
fn my_cool_function() -> i32 {
    return 7;
}
````

You probably want to hide the boilerplate for the doctest as shown above to make it easier for your users to understand the relevant section.

These will run by default when running the `cargo test` command listed above, and you can run them explicitly with

```sh
CARGO_TARGET_THUMBV4T_NONE_EABI_RUNNER=mgba-test-runner cargo test --doc
```
