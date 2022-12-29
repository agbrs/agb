#![no_std]
#![no_main]

/// This example shows how to apply affine transformations
/// (just rotation and scaling in this example) to sprites.

extern crate alloc;

use agb::display::{
    affine::AffineMatrix,
    object::{Graphics, Tag, TagMap},
};
use agb_fixnum::{num, Num, Vector2D};

const GRAPHICS: &Graphics = agb::include_aseprite!("examples/gfx/heart.aseprite");
const TAG_MAP: &TagMap = GRAPHICS.tags();

const SPRITE: &Tag = TAG_MAP.get("Heart");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();

    let mut rot_sprite = gfx.object(gfx.sprite(SPRITE.sprite(0)));
    let mut scale_sprite = gfx.object(gfx.sprite(SPRITE.sprite(0)));
    let mut scale_sprite_double = gfx.object(gfx.sprite(SPRITE.sprite(0)));

    // The transformed version of these sprites are clipped
    // to the sprite size 
    scale_sprite.show_affine();
    rot_sprite.show_affine();

    // The transformed version of this sprite are clipped
    // to the 2x sprite size 
    scale_sprite_double.show_affine_double();


    let rot_matrix_idx = 0;
    rot_sprite.set_affine_matrix(rot_matrix_idx);

    let scale_matrix_idx = 1;
    scale_sprite.set_affine_matrix(scale_matrix_idx);
    scale_sprite_double.set_affine_matrix(scale_matrix_idx);

    rot_sprite.set_position(Vector2D { x: 30, y: 40 });
    scale_sprite.set_position(Vector2D { x: 150, y: 40 });
    scale_sprite_double.set_position(Vector2D { x: 90, y: 100 });

    let zero = num!(0.);
    let one = num!(1.);
    let mut angle: Num<i32, 8> = zero;

    // Note scale here is inverse, large values mean smaller sprite
    let mut scale: Num<i32, 8> = one;
    let mut scale_increasing = true;
    let vblank = agb::interrupt::VBlank::get();

    loop {
        angle += num!(0.01);

        let rot_matrix = AffineMatrix::from_rotation(angle).try_to_object().unwrap();
        rot_matrix.commit(rot_matrix_idx);

        if scale_increasing {
            scale += num!(0.04);
            if scale > num!(4.) {
                scale_increasing = false;
            }
        } else {
            scale -= num!(0.06);
            if scale < num!(0.5) {
                scale_increasing = true;
            }
        }

        let scale_matrix = AffineMatrix::from_scale(Vector2D { x: scale, y: scale })
            .try_to_object()
            .unwrap();
        scale_matrix.commit(scale_matrix_idx);

        vblank.wait_for_vblank();
        gfx.commit();
    }
}
