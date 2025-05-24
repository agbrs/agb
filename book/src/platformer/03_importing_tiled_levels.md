# Importing Tiled levels

What we want to write is something that parses the Tiled level and creates a representation in Rust.
This will be run at build time.
We do this at build time to avoid doing any parsing on the GBA itself.

There is a Rust crate for parsing Tiled levels called [`tiled`](https://crates.io/crates/tiled).
To easily write out the Rust code for the levels, we will use the [`quote`](https://crates.io/crates/quote) crate.
In order to have access to the `TokenStream` type, we will also need [`proc-macro2`](https://crates.io/crates/proc-macro2).
Include this in your `Cargo.toml`

```toml
[build-dependencies]
quote = "1"
proc-macro2 = "1"
tiled = "0.14.0"
```

# `build.rs`

The `build.rs` file placed in the manifest directory will be run before the compilation of your game.
It can do whatever it wants in this time and can output content to be used in your game.
In a larger game you may want to make your own crate for the logic of your `build.rs` file for being able to split things in logical parts and for testability.
We'll not do this.

# Some tiled boilerplate

Working with the tiled library isn't ideal.
For instance, we've used two layers that we've given nice names to, it would be nice to use these names to access the layers themselves.
The library we will be using doesn't support this.
So we will use the normal trick of making a trait that we implement on the foreign type.

```rust
trait GetLayer {
    fn get_layer_by_name(&self, name: &str) -> Layer;
    fn get_tile_layer(&self, name: &str) -> FiniteTileLayer;
    fn get_object_layer(&self, name: &str) -> ObjectLayer;
}

impl GetLayer for Map {
    fn get_layer_by_name(&self, name: &str) -> Layer {
        self.layers().find(|x| x.name == name).unwrap()
    }
    fn get_tile_layer(&self, name: &str) -> FiniteTileLayer {
        match self.get_layer_by_name(name).as_tile_layer().unwrap() {
            TileLayer::Finite(finite_tile_layer) => finite_tile_layer,
            TileLayer::Infinite(_) => panic!("Infinite tile layer not supported"),
        }
    }

    fn get_object_layer(&self, name: &str) -> ObjectLayer {
        self.get_layer_by_name(name).as_object_layer().unwrap()
    }
}
```

A `build.rs` file only runs when it changes or a dependency changes.
What counts as a dependency?
You have to tell `Cargo` each file you depend on by using [`rerun-if-changed`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed).
We can add this capability in the tiled library by using their `Reader` trait.

```rust
struct BuildResourceReader;

impl ResourceReader for BuildResourceReader {
    type Resource = <FilesystemResourceReader as ResourceReader>::Resource;

    type Error = <FilesystemResourceReader as ResourceReader>::Error;

    fn read_from(
        &mut self,
        path: &std::path::Path,
    ) -> std::result::Result<Self::Resource, Self::Error> {
        println!("cargo::rerun-if-changed={}", path.to_string_lossy());
        FilesystemResourceReader.read_from(path)
    }
}
```

This adds a reader that passes through to the existing `FilesystemResourceReader` but intercepts each one to tell `cargo` to depend on the file being read.
The whole reason for doing this is that loading a tiled map could mean we also need to load the various files it references, like the tilesets.
If we were to change the tileset, maybe adding tiles or changing the tags, we would like that to be reflected in the next build of our game.
Make sure to properly tell `cargo` about your dependencies as it will annoy you otherwise!


# Loading the level into an internal representation

It is best practice and easier to maintain to import the level into an internal representation and then convert that into your game representation.
The internal representation can be inefficient as it's going to use your powerful build machine rather than the underpowered GBA.
This will be the representation we use.

```rust
#[derive(Debug, Clone, Copy)]
struct TileInfo {
    id: Option<u32>,
    colliding: bool,
    win: bool,
}

#[derive(Debug, Clone)]
struct Level {
    size: (u32, u32),
    tiles: Vec<TileInfo>,
    player_start: (i32, i32),
}
```

Then we can make our level loading function

```rust
fn load_level(level: &str) -> Result<Level, Box<dyn Error>> {
    let level = tiled::Loader::with_reader(BuildResourceReader).load_tmx_map(level)?;

    // ...

    Ok(Level{
        size: (width, height),
        player_start: (player_x, player_y),
        tiles,
    })
}
```

Lets break down how we load each one

## Size

By far the easiest to do, the level has width and height getters.

```rust
let width = map.width();
let height = map.height();
```

## Player position

We get the object layer and then find the first object with the name of `PLAYER` which is what we set the object to be called when we made the level.

```rust
let objs = level.get_object_layer("Objects");

let player = objs
    .objects()
    .find(|x| x.name == "PLAYER")
    .expect("Should be able to find the player");

let player_x = player.x as i32;
let player_y = player.y as i32;
```

## Tiles

All we have on a tile layer is a `get_tile` method.
We have to iterate over all tiles ourselves.
The easiest is to use nested for loops.

```rust
let mut tiles = Vec::new();

for y in 0..height {
    for x in 0..width {
        let tile = match map.get_tile(x as i32, y as i32) {
            // no tile, use standard settings for the transparent tile
            None => TileInfo {
                colliding: false,
                win: false,
                id: None,
            },

            Some(tile) => {
                // get the properties we set on the tile
                let properties = &tile.get_tile().unwrap().properties;

                // check the properties on the tile
                let colliding = properties["COLLISION"] == PropertyValue::BoolValue(true);
                let win = properties["WIN"] == PropertyValue::BoolValue(true);
                TileInfo {
                    colliding,
                    win,
                    id: Some(tile.id()),
                }
            }
        };

        tiles.push(tile);
    }
}
```

# Making our game representation

What we're going to do here is output a Rust file that we will include in our game that contains various `static`s for each of our levels.
We need to do some work in our `main.rs` file now to enable us to output the levels.

The first is to include the background tiles, we do this because the `TileSettings` we refer to will be in these tiles.
```rust
include_background_gfx!(mod tiles, "2ce8f4", TILES => "gfx/tilesheet.png");
```

Now we want to define the representation of the level that is used in the game itself.
This will be part of the ROM and we will have some pointer to the current level that will drive our display and game logic.
This means we want to prioritise direct access to the tiles and collision data.
We will have the background stored as a flattened list of tiles and the collision and win maps stored as bit arrays where each bit corresponds to a tile and whether the flag is set.

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

Now we want a place to include the levels that will be output by the `build.rs`.

```rust
mod levels {
    // It's a matter of style whether you want to include these here or output them as part of your `build.rs` file.
    // I prefer to include as little as possible in the `build.rs` for no particular reason.
    use super::Level;
    use agb::display::tiled::TileSetting;
    static TILES: &[TileSetting] = super::tiles::TILES.tile_settings;

    // This will include the referenced file in our current file _as is_.
    // As we've not made it yet this won't work, but we will come to making it...
    include!(concat!(env!("OUT_DIR"), "/levels.rs"));
}
```

# Outputting our game representation

Back in our `build.rs` file we need to output the levels.
To create the output for our level, we will use `quote`.
This crate makes it very easy to define code that can be made into a string.
It's widely used in proc-macro crates, but can be used outside of them.

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
                .fold(0u8, |a, b| (a << 1) | b)
        });

        let winning_map = self
            .tiles
            .chunks(8)
            .map(|x| x.iter().map(|x| x.win as u8).fold(0u8, |a, b| (a << 1) | b));

        let (player_x, player_y) = self.player_start;
        let (width, height) = self.size;

        // see how easy it is to define Rust code! See the quote documentation for more details.
        quote! {
            Level {
                // just puts the width in
                width: #width,
                height: #height,
                // this includes every element of the iterator separated by commas
                background: &[#(#background_tiles),*],
                collision_map: &[#(#collision_map),*],
                winning_map: &[#(#winning_map),*],
                player_start: (#player_x, #player_y),
            }
        }.to_tokens(tokens)
    }
}
```

# Driving the level output

In our `build.rs` file, we need to include a main function to be run by `cargo`.

```rust
// the levels we should load
static LEVELS: &[&str] = &["level_01.tmx"];

fn main() -> Result<(), Box<dyn Error>> {
    // the file we output to which is in the `OUT_DIR` directory. This is a directory
    // provided by cargo designed to be used by the build script to include output into
    let mut file =
        std::fs::File::create(format!("{}/levels.rs", std::env::var("OUT_DIR").unwrap()))?;

    // make and write each level to the output
    for (number, level) in LEVELS.iter().enumerate() {
        let ident = quote::format_ident!("LEVEL_{}", number);
        let level = import_level(&format!("tiled/{level}"))?;
        let content = quote! {
            static #ident: Level = #level;
        };
        writeln!(file, "{content}")?;
    }

    // define an array of all the levels to be used by the game, therefore make the array `pub`.
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

# Summary

Here we've made a level import system for our game.
We've shown how you can use the basic Tiled features along with a more advanced one (custom properties).
How you use tiled is highly individualised and as your projects grow so too will your use of tiled and the various features you use and have to write support for in your import system.
To make levels quickly and efficiently, you will want to individualise it and make it work for you.
