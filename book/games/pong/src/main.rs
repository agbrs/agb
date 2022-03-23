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

use agb::display::object::{Graphics, Tag};
use agb::Gba;

const GRAPHICS: &Graphics = agb::include_aseprite!("gfx/sprites.aseprite");

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
#[agb::entry]
fn main(mut gba: Gba) -> ! {
    let object = gba.display.object.get();

    const BALL: &Tag = GRAPHICS.tags().get("Ball");
    let ball_sprite = object.sprite(BALL.sprite(0));
    let mut ball = object.get_object(ball_sprite);

    ball.set_x(50).set_y(50).show();

    let mut ball_x = 50;
    let mut ball_y = 50;
    let mut x_velocity = 1;
    let mut y_velocity = 1;

    loop {
        ball_x = (ball_x + x_velocity).clamp(0, agb::display::WIDTH - 16);
        ball_y = (ball_y + y_velocity).clamp(0, agb::display::HEIGHT - 16);

        if ball_x == 0 || ball_x == agb::display::WIDTH - 16 {
            x_velocity = -x_velocity;
        }

        if ball_y == 0 || ball_y == agb::display::HEIGHT - 16 {
            y_velocity = -y_velocity;
        }

        ball.set_x(ball_x as u16).set_y(ball_y as u16);

        agb::display::busy_wait_for_vblank();
        ball.commit();
    }
}
