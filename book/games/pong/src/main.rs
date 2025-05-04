// Games made using `agb` are no_std which means you don't have access to the standard
// rust library. This is because the game boy advance doesn't really have an operating
// system, so most of the content of the standard library doesn't apply.
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
    fixnum::{vec2, Rect, Vector2D},
    include_aseprite,
    input::ButtonController,
};

// Import the sprites in to this static. This holds the sprite
// and palette data in a way that is manageable by agb.
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite"
);

struct Paddle {
    pos: Vector2D<i32>,
}

impl Paddle {
    fn new(pos: Vector2D<i32>) -> Self {
        Self { pos }
    }

    fn set_pos(&mut self, pos: Vector2D<i32>) {
        self.pos = pos;
    }

    fn move_by(&mut self, y: i32) {
        self.pos += vec2(0, y);
    }

    fn collision_rect(&self) -> Rect<i32> {
        Rect::new(self.pos, vec2(16, 16 * 3))
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(self.pos)
            .show(frame);
        Object::new(sprites::PADDLE_MID.sprite(0))
            .set_pos(self.pos + vec2(0, 16))
            .show(frame);
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(self.pos + vec2(0, 32))
            .set_vflip(true)
            .show(frame);
    }
}

// The main function must take 0 arguments and never return. The agb::entry decorator
// ensures that everything is in order. `agb` will call this after setting up the stack
// and interrupt handlers correctly.
#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    // Get the graphics manager
    let mut gfx = gba.graphics.get();

    let mut button_controller = ButtonController::new();

    // Create an object with the ball sprite
    let mut ball = Object::new(sprites::BALL.sprite(0));

    // Place this at some point on the screen, (50, 50) for example
    ball.set_pos((50, 50));

    let mut paddle_a = Paddle::new(vec2(8, 8));
    let paddle_b = Paddle::new(vec2(240 - 16 - 8, 8));

    let mut ball_pos = vec2(50, 50);
    let mut ball_velocity = vec2(1, 1);

    loop {
        button_controller.update();

        paddle_a.move_by(button_controller.y_tri() as i32);

        // Speculatively move the ball, we'll update the velocity if this causes it to intersect with either the
        // edge of the map or a paddle.
        let potential_ball_pos = ball_pos + ball_velocity;

        let ball_rect = Rect::new(potential_ball_pos, vec2(16, 16));
        if paddle_a.collision_rect().touches(ball_rect) {
            ball_velocity.x = 1;
        }

        if paddle_b.collision_rect().touches(ball_rect) {
            ball_velocity.x = -1;
        }

        // We check if the ball reaches the edge of the screen and reverse it's direction
        if potential_ball_pos.x <= 0 || potential_ball_pos.x >= agb::display::WIDTH - 16 {
            ball_velocity.x *= -1;
        }

        if potential_ball_pos.y <= 0 || potential_ball_pos.y >= agb::display::HEIGHT - 16 {
            ball_velocity.y *= -1;
        }

        ball_pos += ball_velocity;

        // Set the position of the ball to match our new calculated position
        ball.set_pos(ball_pos);

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame);
        paddle_b.show(&mut frame);

        frame.commit();
    }
}
