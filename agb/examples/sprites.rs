#![no_std]
#![no_main]

extern crate alloc;

use agb::display::object::{ObjectController, Sprite, TagMap};
use alloc::vec::Vec;

const SPRITE_TAGS: (&[Sprite], &TagMap) =
    agb::include_aseprite!("../examples/the-purple-night/gfx/objects.aseprite");
const SPRITES: &[Sprite] = SPRITE_TAGS.0;
const TAG_MAP: &TagMap = SPRITE_TAGS.1;

fn all_sprites(gfx: &ObjectController) {
    let mut input = agb::input::ButtonController::new();
    let mut objs = Vec::new();

    for y in 0..9 {
        for x in 0..14 {
            let mut obj = gfx
                .get_object(gfx.get_sprite(&SPRITES[0]).unwrap())
                .unwrap();
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
                obj.set_sprite(gfx.get_sprite(&SPRITES[this_image]).unwrap());
                obj.commit();
            }
        }
    }
}

fn all_tags(gfx: &ObjectController) {
    let mut input = agb::input::ButtonController::new();
    let mut objs = Vec::new();

    for (i, v) in TAG_MAP.values().enumerate() {
        let x = (i % 14) as i32;
        let y = (i / 14) as i32;
        let mut obj = gfx
            .get_object(gfx.get_sprite(v.get_sprite(0)).unwrap())
            .unwrap();
        obj.show();
        obj.set_position((x * 16 + 8, y * 16 + 8).into());
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
                obj.set_sprite(gfx.get_sprite(tag.get_animation_sprite(image)).unwrap());
                obj.commit();
            }
        }
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();

    loop {
        all_tags(&gfx);
        all_sprites(&gfx);
    }
}
