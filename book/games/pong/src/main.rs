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
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

use agb::{
    display::{object::Object, GraphicsFrame},
    include_aseprite,
};

// Import the sprites in to this static. This holds the sprite
// and palette data in a way that is manageable by agb.
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);

struct Paddle {
    start: Object,
    mid: Object,
    end: Object,
}

impl Paddle {
    fn new(start_x: i32, start_y: i32) -> Self {
        let paddle_start = Object::new(sprites::PADDLE_END.sprite(0));
        let paddle_mid = Object::new(sprites::PADDLE_MID.sprite(0));
        let mut paddle_end = Object::new(sprites::PADDLE_END.sprite(0));

        paddle_end.set_vflip(true);

        let mut paddle = Self {
            start: paddle_start,
            mid: paddle_mid,
            end: paddle_end,
        };

        paddle.set_position(start_x, start_y);

        paddle
    }

    fn set_position(&mut self, x: i32, y: i32) {
        // new! use of the `set_position` method. This is a helper feature using
        // agb's vector types. For now we can just use it to avoid adding them
        // separately
        self.start.set_position((x, y));
        self.mid.set_position((x, y + 16));
        self.end.set_position((x, y + 32));
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        self.start.show(frame);
        self.mid.show(frame);
        self.end.show(frame);
    }
}

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Get the OAM manager
    let mut gfx = gba.display.graphics.get();

    // Create an object with the ball sprite
    let mut ball = Object::new(sprites::BALL.sprite(0));

    // Place this at some point on the screen, (50, 50) for example
    ball.set_position((50, 50));

    let paddle_a = Paddle::new(8, 8);
    let paddle_b = Paddle::new(240 - 16 - 8, 8);

    let mut ball_x = 50;
    let mut ball_y = 50;
    let mut x_velocity = 1;
    let mut y_velocity = 1;

    loop {
        // This will calculate the new position and enforce the position
        // of the ball remains within the screen
        ball_x = (ball_x + x_velocity).clamp(0, agb::display::WIDTH - 16);
        ball_y = (ball_y + y_velocity).clamp(0, agb::display::HEIGHT - 16);

        // We check if the ball reaches the edge of the screen and reverse it's direction
        if ball_x == 0 || ball_x == agb::display::WIDTH - 16 {
            x_velocity = -x_velocity;
        }

        if ball_y == 0 || ball_y == agb::display::HEIGHT - 16 {
            y_velocity = -y_velocity;
        }

        // Set the position of the ball to match our new calculated position
        ball.set_position((ball_x, ball_y));

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame);
        paddle_b.show(&mut frame);

        // Wait for vblank, then commit the objects to the screen
        agb::display::busy_wait_for_vblank();

        frame.commit();
    }
}
