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

use agb::display::object::Size;
use agb::Gba;

// Put all the graphics related code in the gfx module
mod gfx {
    use agb::display::object::ObjectControl;

    // Import the sprites into this module. This will create a `sprites` module
    // and within that will be a constant called `sprites` which houses all the
    // palette and tile data.
    agb::include_gfx!("gfx/sprites.toml");

    // Loads the sprites tile data and palette data into VRAM
    pub fn load_sprite_data(object: &mut ObjectControl) {
        object.set_sprite_palettes(sprites::sprites.palettes);
        object.set_sprite_tilemap(sprites::sprites.tiles);
    }
}

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
#[agb::entry]
fn main() -> ! {
    let mut gba = Gba::new();

    let _tiled = gba.display.video.tiled0();
    let mut object = gba.display.object.get();
    gfx::load_sprite_data(&mut object);
    object.enable();

    let mut ball = object.get_object_standard();

    ball.set_x(50)
        .set_y(50)
        .set_sprite_size(Size::S16x16)
        .set_tile_id(4 * 2)
        .show()
        .commit();

    loop {}
}
