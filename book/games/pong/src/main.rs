// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
//
// Provided you haven't disabled it, agb does provide an allocator, so it is possible
// to use both the `core` and the `alloc` built in crates.
#![no_std]
// `agb` defines its own `main` function, so you must declare your game's main function
// using the #[agb::entry] proc macro. Failing to do so will cause failure in linking
// which won't be a particularly clear error message.
#![no_main]

// This is required in order to ensure that the panic handler defined in `agb` is set
// up correctly.
extern crate agb;

mod gfx {
    agb::include_gfx!("gfx/sprites.toml");
}

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
#[agb::entry]
fn main() -> ! {
    let mut gba = agb::Gba::new();

    let mut object = gba.display.object.get();
    object.set_sprite_palettes(gfx::sprites::sprites.palettes);
    object.set_sprite_tilemap(gfx::sprites::sprites.tiles);

    loop {}
}
