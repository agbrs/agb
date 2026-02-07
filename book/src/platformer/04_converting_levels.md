# Converting levels to game data

We now have our level loaded into an intermediate representation.
We need to convert this into something efficient for the GBA.

# Why two `Level` structs?

The build-time `Level` (in `build.rs`) is convenient to work with but wasteful — it uses `Vec`s and `Option`s.
We don't want any of that on the GBA.
The runtime `Level` (in `main.rs`) uses flat arrays and bit-packed maps so it takes up as little space as possible.

# The runtime `Level` struct

Open `src/main.rs` and replace the template code with the following.
First, we need to import our background tiles:

```rust
use agb::{
    display::tiled::TileSetting,
    include_background_gfx,
};

extern crate alloc;

include_background_gfx!(mod tiles, "2ce8f4", TILES => "gfx/tileset.png");
```

The `include_background_gfx!` macro imports a PNG as a set of background tiles.
The `"2ce8f4"` is the transparency colour — pixels of this colour become transparent.
This is also used as the background colour where there are no tiles.
`TILES` is the name we give the resulting tileset.
For more detail, see the [Backgrounds](../pong/06_background.md) chapter of the pong tutorial and the [Backgrounds deep dive](../articles/backgrounds.md).

Now define the runtime `Level` struct:

```rust
struct Level {
    width: u32,
    height: u32,
    background: &'static [TileSetting],
    collision_map: &'static [u8],
    winning_map: &'static [u8],
    player_start: (i32, i32),
}
```

Each field serves a specific purpose:

- `width` and `height`: the level dimensions in tiles.
- `background`: a flat array of `TileSetting` values, one per tile, used to display the level.
- `collision_map`: a bit-packed array where each bit represents whether a tile blocks movement.
- `winning_map`: a bit-packed array where each bit represents whether a tile triggers a level win.
- `player_start`: the pixel coordinates where the player spawns.

# Including the generated levels

The build script will output a Rust file into Cargo's `OUT_DIR`.
We include it using a `levels` module:

```rust
mod levels {
    use super::Level;
    use agb::display::tiled::TileSetting;
    static TILES: &[TileSetting] = super::tiles::TILES.tile_settings;

    include!(concat!(env!("OUT_DIR"), "/levels.rs"));
}
```

The critical mapping here: the tile IDs from Tiled correspond to indices into the `TILES.tile_settings` array.
So `TILES[tile_id]` gives us the `TileSetting` for that tile.
We define `TILES` inside the `levels` module so the generated code can reference it directly.

Finally, add a placeholder entry point so the game compiles:

```rust
#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    loop {
        agb::halt();
    }
}
```

# Generating the output with `quote`

Back in `build.rs`, we need to output Rust source code for our levels.
The [`quote`](https://docs.rs/quote) crate lets us write Rust code that generates Rust code.
It's widely used in procedural macros, but we can use it in our build script too.

Add these imports to the top of `build.rs`:

```rust
use std::io::Write;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
```

# Implementing `ToTokens` for `Level`

We implement the `ToTokens` trait on our build-time `Level` to define how it converts to Rust source code:

```rust
impl ToTokens for Level {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let background_tiles = self.tiles.iter().map(|x| match x.id {
            // if the tile is defined, look up the TileSetting in the background tiles
            Some(x) => quote! { TILES[#x as usize] },
            // otherwise, use the blank tile
            None => quote! { TileSetting::BLANK },
        });

        // this creates the bit-array from the booleans on the tiles.
        let collision_map = self.tiles.chunks(8).map(|x| {
            x.iter()
                .map(|x| x.colliding as u8)
                .fold(0u8, |a, b| (a >> 1) | (b << 7))
        });

        let winning_map = self.tiles.chunks(8).map(|x| {
            x.iter()
                .map(|x| x.win as u8)
                .fold(0u8, |a, b| (a >> 1) | (b << 7))
        });

        let (player_x, player_y) = self.player_start;
        let (width, height) = self.size;

        quote! {
            Level {
                width: #width,
                height: #height,
                background: &[#(#background_tiles),*],
                collision_map: &[#(#collision_map),*],
                winning_map: &[#(#winning_map),*],
                player_start: (#player_x, #player_y),
            }
        }
        .to_tokens(tokens)
    }
}
```

The `quote!` macro uses `#variable` to splice in values and `#(#iter),*` to expand iterators with commas between elements.

The bit-packing deserves some explanation.
We pack 8 booleans into each byte.
For each chunk of 8 tiles, we fold them together: shift the existing bits right by one position and place the new bit in the top position (bit 7).
After processing all 8 tiles in a chunk, bit 0 of the byte corresponds to the first tile in the chunk, bit 1 to the second, and so on.

# The complete `main()` for build.rs

Replace the skeleton `main()` we wrote in the previous chapter with the real one:

```rust
static LEVELS: &[&str] = &["level_01.tmx"];

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_file_name = format!("{out_dir}/levels.rs");
    let mut file = std::fs::File::create(out_file_name)?;

    for (number, level) in LEVELS.iter().enumerate() {
        let ident = quote::format_ident!("LEVEL_{}", number);
        let level = import_level(&format!("tiled/{level}"))?;
        let content = quote! {
            static #ident: Level = #level;
        };
        writeln!(file, "{content}")?;
    }

    let levels = (0..LEVELS.len()).map(|x| quote::format_ident!("LEVEL_{}", x));
    writeln!(
        file,
        "{}",
        quote! {
            pub static LEVELS: &[&Level] = &[#(&#levels),*];
        }
    )?;

    Ok(())
}
```

This iterates over each level file, loads it, converts it to tokens, and writes it to `levels.rs` in the output directory.
It also creates a `LEVELS` array that holds references to all levels, which we'll use later to iterate through them.

# What we did

We've completed the build script.
It loads Tiled levels, converts them into efficient bit-packed data, and outputs Rust code that gets compiled directly into the ROM.
Run `cargo build` and take a look at the generated `levels.rs` file in your target's output directory (check the `OUT_DIR` path in your build output) to see what the build script produces.
In the next chapter, we'll display this level on screen.

# Exercise

If you completed the previous exercises and added a `spike` field to `TileInfo`, add a `spike_map` to the runtime `Level` struct and output it from the build script, using the same bit-packing approach as `collision_map`.
