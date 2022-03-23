#![no_std]
#![no_main]

extern crate alloc;

use agb::display::object::{Graphics, ObjectController, Sprite, TagMap};
use alloc::vec::Vec;


const GRAPHICS: &Graphics = agb::include_aseprite!(
    "../examples/the-purple-night/gfx/objects.aseprite",
    "../examples/the-purple-night/gfx/boss.aseprite"
);
const SPRITES: &[Sprite] = GRAPHICS.sprites();
const TAG_MAP: &TagMap = GRAPHICS.tags();

fn all_sprites(gfx: &ObjectController) {
    let mut input = agb::input::ButtonController::new();
    let mut objs = Vec::new();

    for y in 0..9 {
        for x in 0..14 {
            let mut obj = gfx.object(gfx.sprite(&SPRITES[0]));
            obj.show();
            obj.set_position((x * 16 + 8, y * 16 + 8).into());
            objs.push(obj);
        }
    }

    let mut count = 0;
    let mut image = 0;

    let vblank = agb::interrupt::VBlank::get();

    loop {
        vblank.wait_for_vblank();
        input.update();

        if input.is_just_pressed(agb::input::Button::A) {
            break;
        }

        count += 1;

        if count % 5 == 0 {
            image += 1;
            image %= SPRITES.len();
            let objs_len = objs.len();
            for (i, obj) in objs.iter_mut().enumerate() {
                let this_image = (image + i * SPRITES.len() / objs_len) % SPRITES.len();
                obj.set_sprite(gfx.sprite(&SPRITES[this_image]));
                obj.commit();
            }
        }
    }
}

fn all_tags(gfx: &ObjectController) {
    let mut input = agb::input::ButtonController::new();
    let mut objs = Vec::new();

    for (i, v) in TAG_MAP.values().enumerate() {
        let x = (i % 7) as i32;
        let y = (i / 7) as i32;
        let sprite = v.sprite(0);
        let (size_x, size_y) = sprite.size().to_width_height();
        let (size_x, size_y) = (size_x as i32, size_y as i32);
        let mut obj = gfx.object(gfx.sprite(sprite));
        obj.show();
        obj.set_position((x * 32 + 16 - size_x / 2, y * 32 + 16 - size_y / 2).into());
        objs.push((obj, v));
    }

    let mut count = 0;
    let mut image = 0;

    let vblank = agb::interrupt::VBlank::get();

    loop {
        vblank.wait_for_vblank();

        input.update();

        if input.is_just_pressed(agb::input::Button::A) {
            break;
        }

        count += 1;

        if count % 5 == 0 {
            image += 1;
            for (obj, tag) in objs.iter_mut() {
                obj.set_sprite(gfx.sprite(tag.animation_sprite(image)));
                obj.commit();
            }
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();

    let mut timers = gba.timers.timers();

    let _a = agb::interrupt::profiler(&mut timers.timer0, 5000);

    loop {
        all_tags(&gfx);
        all_sprites(&gfx);
    }
}
