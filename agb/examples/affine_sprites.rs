#![no_std]
#![no_main]

/// This example shows how to apply affine transformations
/// (just rotation and scaling in this example) to sprites.
/// Note that these transformations will cause any content outside the
/// sprites size to be clipped.
extern crate alloc;

use agb::display::{
    affine::AffineMatrix,
    object::{Graphics, Tag, TagMap},
};
use agb_fixnum::{num, Num, Vector2D};

const GRAPHICS: &Graphics = agb::include_aseprite!("examples/gfx/tall.aseprite");
const TAG_MAP: &TagMap = GRAPHICS.tags();

const SPRITE: &Tag = TAG_MAP.get("Heart");

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let gfx = gba.display.object.get();

    let mut rot_sprite = gfx.object(gfx.sprite(SPRITE.sprite(0)));
    let mut scale_sprite = gfx.object(gfx.sprite(SPRITE.sprite(0)));
    scale_sprite.set_affine();
    rot_sprite.set_affine();

    let rot_matrix_idx = 0;
    rot_sprite.set_affine_matrix(rot_matrix_idx);

    let scale_matrix_idx = 1;
    scale_sprite.set_affine_matrix(scale_matrix_idx);

    scale_sprite.set_position(Vector2D { x: 130, y: 50 });
    rot_sprite.set_position(Vector2D { x: 30, y: 50 });

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
            if scale < num!(1.) {
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
