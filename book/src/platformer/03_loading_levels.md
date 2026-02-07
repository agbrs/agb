# Loading levels at build time

We want to parse our Tiled level at build time so the GBA doesn't have to do any parsing at runtime.
To achieve this, we'll use a Rust [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html) (`build.rs`) that runs on your computer during `cargo build`, reads the `.tmx` files, and outputs Rust source code that gets compiled into the game.

# What is `build.rs`?

A `build.rs` file is a Rust program that Cargo runs before compiling your crate.
It runs on your build machine (not on the GBA), so it has access to the full standard library, file system, and any crates listed under `[build-dependencies]` in your `Cargo.toml`.

The build script's job is to generate code or data that your main crate can then include.
For us, this means reading Tiled `.tmx` files and outputting efficient Rust data structures.

Create a file called `build.rs` in the root of your project (next to `Cargo.toml`).

# Some Tiled boilerplate

Working with the `tiled` library isn't ideal.
For instance, we've given our layers nice names, and it would be convenient to look up a layer by name.
The `tiled` library doesn't provide a method for this, so we'll use the normal trick of making a trait that we implement on the foreign type.

```rust
use tiled::{
    FilesystemResourceReader, FiniteTileLayer, Layer, Map, ObjectLayer,
    PropertyValue, ResourceReader, TileLayer,
};

trait GetLayer {
    fn get_layer_by_name(&self, name: &str) -> Layer<'_>;
    fn get_tile_layer(&self, name: &str) -> FiniteTileLayer<'_>;
    fn get_object_layer(&self, name: &str) -> ObjectLayer<'_>;
}

impl GetLayer for Map {
    fn get_layer_by_name(&self, name: &str) -> Layer<'_> {
        self.layers().find(|x| x.name == name).unwrap()
    }
    fn get_tile_layer(&self, name: &str) -> FiniteTileLayer<'_> {
        match self.get_layer_by_name(name).as_tile_layer().unwrap() {
            TileLayer::Finite(finite_tile_layer) => finite_tile_layer,
            TileLayer::Infinite(_) => panic!("Infinite tile layer not supported"),
        }
    }

    fn get_object_layer(&self, name: &str) -> ObjectLayer<'_> {
        self.get_layer_by_name(name).as_object_layer().unwrap()
    }
}
```

# Telling Cargo about our dependencies

A `build.rs` file only runs when it changes or a dependency changes.
What counts as a dependency?
You have to tell Cargo each file you depend on by using [`cargo::rerun-if-changed`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed).

We can add this capability in the `tiled` library by using their `ResourceReader` trait.

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

This adds a reader that passes through to the existing `FilesystemResourceReader` but intercepts each read to tell Cargo to depend on the file being accessed.
The whole reason for doing this is that loading a Tiled map could mean we also need to load the various files it references, like the tilesets.
If we were to change the tileset, maybe adding tiles or changing the tags, we would like that to be reflected in the next build of our game.
Make sure to properly tell Cargo about your dependencies as it will annoy you otherwise!

# An intermediate representation

It is best practice and easier to maintain to import the level into an intermediate representation and then convert that into your game representation.
The intermediate representation can be inefficient as it's going to use your powerful build machine rather than the underpowered GBA.

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

`TileInfo` stores the tile's ID (or `None` for empty tiles) and its boolean properties.
`Level` stores the map dimensions, a flat list of tiles (row by row), and the player's starting position.

# Loading the level

Now we can write the function that reads a `.tmx` file and returns our intermediate `Level`.
We'll need `std::error::Error` for the return type — add this import to the top of `build.rs`:

```rust
use std::error::Error;
```

```rust
fn import_level(level: &str) -> Result<Level, Box<dyn Error>> {
    let level = tiled::Loader::with_reader(BuildResourceReader).load_tmx_map(level)?;

    let map = level.get_tile_layer("Level");
    let objs = level.get_object_layer("Objects");

    let width = map.width();
    let height = map.height();

    let mut tiles = Vec::new();

    for y in 0..height {
        for x in 0..width {
            let tile = match map.get_tile(x as i32, y as i32) {
                Some(tile) => {
                    let properties = &tile.get_tile().unwrap().properties;

                    let colliding = properties["COLLISION"] == PropertyValue::BoolValue(true);
                    let win = properties["WIN"] == PropertyValue::BoolValue(true);
                    TileInfo {
                        colliding,
                        win,
                        id: Some(tile.id()),
                    }
                }
                None => TileInfo {
                    colliding: false,
                    win: false,
                    id: None,
                },
            };

            tiles.push(tile);
        }
    }

    let player = objs
        .objects()
        .find(|x| x.name == "PLAYER")
        .expect("Should be able to find the player");

    let player_x = player.x as i32;
    let player_y = player.y as i32;

    Ok(Level {
        size: (width, height),
        tiles,
        player_start: (player_x, player_y),
    })
}
```

Let's break this down:

- We load the map using our `BuildResourceReader` so that Cargo knows about all the files.
- We get the tile layer called `"Level"` (the one we named in Tiled) and the object layer called `"Objects"`.
- We iterate over every tile position, reading its properties. For empty tiles, we use default values.
- We find the `PLAYER` point object to determine the starting position.

# A skeleton `main()`

To verify everything works, add a temporary `main()` function:

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let level = import_level("tiled/level_01.tmx")?;
    println!("cargo::warning=Loaded level: {}x{}", level.size.0, level.size.1);
    println!("cargo::warning=Player starts at: {:?}", level.player_start);
    Ok(())
}
```

Run `cargo build` and you should see these messages as warnings in the build output, confirming the level loads correctly.

# What we did

We've written the first half of our build script: loading Tiled levels into an intermediate representation.
We can now load our level data.
In the next chapter, we'll convert this into a format our game can use.

# Exercise

Add a `cargo::warning` message in your `main()` to print the total number of colliding tiles.
Run `cargo build` and check the output.

If you added a `SPIKE` property to your tileset in the previous chapter's exercise, extend `TileInfo` and `import_level` to pass it through as well.
