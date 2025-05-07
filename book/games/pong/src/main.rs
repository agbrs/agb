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

extern crate alloc;

use agb::{
    display::{
        GraphicsFrame, Priority, WIDTH,
        object::Object,
        tiled::{RegularBackgroundSize, RegularBackgroundTiles, TileFormat, VRAM_MANAGER},
    },
    fixnum::{Num, Rect, Vector2D, num, vec2},
    include_aseprite, include_background_gfx, include_wav,
    input::ButtonController,
    sound::mixer::{Frequency, Mixer, SoundChannel, SoundData},
};

use agb_tracker::{Track, Tracker, include_xm};

type Fixed = Num<i32, 8>;

// Import the sprites in to this static. This holds the sprite
// and palette data in a way that is manageable by agb.
include_aseprite!(
    mod sprites,
    "gfx/sprites.aseprite",
    "gfx/cpu-health.aseprite",
);

include_background_gfx!(
    mod background,
    PLAY_FIELD => deduplicate "gfx/background.aseprite",
    SCORE => deduplicate "gfx/player-health.aseprite",
);

static BALL_PADDLE_HIT: SoundData = include_wav!("sfx/ball-paddle-hit.wav");
static BGM: Track = include_xm!("sfx/bgm.xm");

struct Paddle {
    pos: Vector2D<Fixed>,
    health: i32,
}

impl Paddle {
    fn new(pos: Vector2D<Fixed>) -> Self {
        Self { pos, health: 3 }
    }

    fn move_by(&mut self, y: Fixed) {
        self.pos += vec2(num!(0), y);
    }

    fn collision_rect(&self) -> Rect<Fixed> {
        Rect::new(self.pos, vec2(num!(16), num!(16 * 3)))
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        let sprite_pos = self.pos.floor();

        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(sprite_pos)
            .set_priority(Priority::P1)
            .show(frame);
        Object::new(sprites::PADDLE_MID.sprite(0))
            .set_pos(sprite_pos + vec2(0, 16))
            .set_priority(Priority::P1)
            .show(frame);
        Object::new(sprites::PADDLE_END.sprite(0))
            .set_pos(sprite_pos + vec2(0, 32))
            .set_priority(Priority::P1)
            .set_vflip(true)
            .show(frame);
    }
}

struct Ball {
    pos: Vector2D<Fixed>,
    velocity: Vector2D<Fixed>,
}

impl Ball {
    fn new(pos: Vector2D<Fixed>, velocity: Vector2D<Fixed>) -> Self {
        Self { pos, velocity }
    }

    fn update(&mut self, paddle_a: &mut Paddle, paddle_b: &mut Paddle, mixer: &mut Mixer) {
        // Speculatively move the ball, we'll update the velocity if this causes it to intersect with either the
        // edge of the map or a paddle.
        let potential_ball_pos = self.pos + self.velocity;

        let ball_rect = Rect::new(potential_ball_pos, vec2(num!(16), num!(16)));

        if paddle_a.collision_rect().touches(ball_rect) {
            play_hit(mixer);

            self.velocity.x = self.velocity.x.abs();

            let y_difference = (ball_rect.centre().y - paddle_a.collision_rect().centre().y) / 32;
            self.velocity.y += y_difference;
        }

        if paddle_b.collision_rect().touches(ball_rect) {
            play_hit(mixer);

            self.velocity.x = -self.velocity.x.abs();

            let y_difference = (ball_rect.centre().y - paddle_b.collision_rect().centre().y) / 32;
            self.velocity.y += y_difference;
        }

        // We check if the ball reaches the edge of the screen and reverse it's direction
        if potential_ball_pos.x <= num!(0) {
            self.velocity.x *= -1;
            paddle_a.health -= 1;
        } else if potential_ball_pos.x >= num!(agb::display::WIDTH - 16) {
            self.velocity.x *= -1;
            paddle_b.health -= 1;
        }

        if potential_ball_pos.y <= num!(0)
            || potential_ball_pos.y >= num!(agb::display::HEIGHT - 16)
        {
            self.velocity.y *= -1;
        }

        self.pos += self.velocity;
    }

    fn show(&self, frame: &mut GraphicsFrame) {
        Object::new(sprites::BALL.sprite(0))
            .set_pos(self.pos.floor())
            .set_priority(Priority::P1)
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

    let mut mixer = gba.mixer.mixer(Frequency::Hz32768);

    // Make sure the background palettes are set up
    VRAM_MANAGER.set_background_palettes(background::PALETTES);

    let mut bg = RegularBackgroundTiles::new(
        Priority::P3,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );
    bg.fill_with(&background::PLAY_FIELD);

    let mut player_health_background = RegularBackgroundTiles::new(
        Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    for i in 0..4 {
        player_health_background.set_tile(
            (i, 0),
            &background::SCORE.tiles,
            background::SCORE.tile_settings[i as usize],
        );
    }

    player_health_background.set_scroll_pos((-4, -4));

    let mut button_controller = ButtonController::new();

    let mut ball = Ball::new(vec2(num!(50), num!(50)), vec2(num!(2), num!(0.5)));

    let mut paddle_a = Paddle::new(vec2(num!(8), num!(8)));
    let mut paddle_b = Paddle::new(vec2(num!(240 - 16 - 8), num!(8)));

    let mut tracker = Tracker::new(&BGM);
    mixer.enable();

    loop {
        button_controller.update();

        paddle_a.move_by(Fixed::from(button_controller.y_tri() as i32));
        ball.update(&mut paddle_a, &mut paddle_b, &mut mixer);

        for i in 0..3 {
            let tile_index = if i < paddle_a.health { 4 } else { 5 };
            player_health_background.set_tile(
                (i + 4, 0),
                &background::SCORE.tiles,
                background::SCORE.tile_settings[tile_index],
            );
        }

        let mut frame = gfx.frame();

        ball.show(&mut frame);
        paddle_a.show(&mut frame);
        paddle_b.show(&mut frame);

        bg.show(&mut frame);
        player_health_background.show(&mut frame);
        show_cpu_health(&paddle_b, &mut frame);

        tracker.step(&mut mixer);
        mixer.frame();
        frame.commit();
    }
}

fn play_hit(mixer: &mut Mixer) {
    let hit_sound = SoundChannel::new(BALL_PADDLE_HIT);
    mixer.play_sound(hit_sound);
}

fn show_cpu_health(paddle: &Paddle, frame: &mut GraphicsFrame) {
    // The text CPU: ends at exactly the edge of the sprite (which the player text doesn't).
    // so we add a 3 pixel gap between the text and the start of the hearts to make it look a bit nicer.
    const TEXT_HEART_GAP: i32 = 3;

    // The top left of the CPU health. The text is 2 tiles wide and the hearts are 3.
    // We also offset the y value by 4 pixels to keep it from the edge of the screen.
    let top_left = vec2(WIDTH - 4 - (2 + 3) * 8 - TEXT_HEART_GAP, 4);

    // Display the text `CPU:`
    Object::new(sprites::CPU.sprite(0))
        .set_pos(top_left)
        .show(frame);
    Object::new(sprites::CPU.sprite(1))
        .set_pos(top_left + vec2(8, 0))
        .show(frame);

    // For each heart frame, show that too
    for i in 0..3 {
        let heart_frame = if i < paddle.health { 0 } else { 1 };

        Object::new(sprites::HEART.sprite(heart_frame))
            .set_pos(top_left + vec2(16 + i * 8 + TEXT_HEART_GAP, 0))
            .show(frame);
    }
}
