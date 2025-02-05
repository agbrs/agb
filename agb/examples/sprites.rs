#![no_std]
#![no_main]

extern crate alloc;

use agb::display::{
    affine::AffineMatrix,
    object::{self, AffineMode, Object, ObjectAffine, Sprite, TagMap},
    Graphics,
};
use agb::fixnum::num;
use agb_fixnum::Num;
use alloc::vec::Vec;

static GRAPHICS: &object::Graphics = agb::include_aseprite!(
    "examples/gfx/objects.aseprite",
    "examples/gfx/boss.aseprite",
    "examples/gfx/wide.aseprite",
    "examples/gfx/tall.aseprite"
);
static SPRITES: &[Sprite] = GRAPHICS.sprites();
static TAG_MAP: &TagMap = GRAPHICS.tags();

fn all_sprites(oam: &mut Graphics, rotation_speed: Num<i32, 16>) {
    let mut input = agb::input::ButtonController::new();
    let mut objs = Vec::new();

    let mut rotation: Num<i32, 16> = num!(0.);

    let rotation_matrix = AffineMatrix::from_rotation(rotation);
    let matrix = object::AffineMatrixInstance::new(rotation_matrix.to_object_wrapping());

    for y in 0..9 {
        for x in 0..14 {
            let mut obj = ObjectAffine::new(&SPRITES[0], matrix.clone(), AffineMode::Affine);
            obj.set_position((x * 16 + 8, y * 16 + 8));
            objs.push(obj);
        }
    }

    let mut count = 0;
    let mut image = 0;

    let vblank = agb::interrupt::VBlank::get();

    loop {
        let mut frame = oam.frame();
        input.update();

        if input.is_just_pressed(agb::input::Button::A) {
            break;
        }

        rotation += rotation_speed;
        let rotation_matrix = AffineMatrix::from_rotation(rotation);

        let matrix = object::AffineMatrixInstance::new(rotation_matrix.to_object_wrapping());

        for obj in objs.iter_mut() {
            obj.set_affine_matrix(matrix.clone());
        }

        count += 1;

        if count % 5 == 0 {
            image += 1;
            image %= SPRITES.len();
            for (i, obj) in objs.iter_mut().enumerate() {
                let this_image = (image + i) % SPRITES.len();
                obj.set_sprite(&SPRITES[this_image]);
            }
        }

        for obj in objs.iter() {
            obj.show(&mut frame);
        }

        vblank.wait_for_vblank();

        frame.commit();
    }
}

fn all_tags(gfx: &mut Graphics) {
    let mut input = agb::input::ButtonController::new();
    let mut objs = Vec::new();

    for (i, v) in TAG_MAP.values().enumerate() {
        let x = (i % 7) as i32;
        let y = (i / 7) as i32;
        let sprite = v.sprite(0);
        let (size_x, size_y) = sprite.size().to_width_height();
        let (size_x, size_y) = (size_x as i32, size_y as i32);
        let mut obj = Object::new(sprite);
        obj.set_position((x * 32 + 16 - size_x / 2, y * 32 + 16 - size_y / 2));
        objs.push((obj, v));
    }

    let mut count = 0;
    let mut image = 0;

    let vblank = agb::interrupt::VBlank::get();

    loop {
        let mut frame = gfx.frame();

        input.update();

        if input.is_just_pressed(agb::input::Button::A) {
            break;
        }

        count += 1;

        if count % 5 == 0 {
            image += 1;
            for (obj, tag) in objs.iter_mut() {
                obj.set_sprite(tag.animation_sprite(image));
            }
        }

        for (obj, _) in objs.iter() {
            obj.show(&mut frame);
        }

        vblank.wait_for_vblank();

        frame.commit();
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let mut gfx = gba.display.graphics.get();

    loop {
        all_tags(&mut gfx);
        all_sprites(&mut gfx, num!(0.));
        all_sprites(&mut gfx, num!(0.01));
    }
}
