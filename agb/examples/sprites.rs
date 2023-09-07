#![no_std]
#![no_main]

extern crate alloc;

use agb::fixnum::num;
use agb::input::Button;
use agb::{
    display::{
        affine::AffineMatrix,
        object::{
            self, AffineMode, Graphics, OamUnmanaged, ObjectUnmanaged, Sprite, SpriteLoader, TagMap,
        },
    },
    input::ButtonController,
    interrupt::VBlank,
};
use agb_fixnum::Num;
use alloc::vec::Vec;

static GRAPHICS: &Graphics = agb::include_aseprite!(
    "examples/gfx/objects.aseprite",
    "examples/gfx/boss.aseprite",
    "examples/gfx/wide.aseprite",
    "examples/gfx/tall.aseprite"
);
static SPRITES: &[Sprite] = GRAPHICS.sprites();
static TAG_MAP: &TagMap = GRAPHICS.tags();

fn all_sprites(
    sprite_loader: &mut SpriteLoader,
    rotation: Num<i32, 16>,
    offset: usize,
) -> Vec<ObjectUnmanaged> {
    let rotation_matrix = AffineMatrix::from_rotation(rotation);
    let matrix = object::AffineMatrixInstance::new(rotation_matrix.to_object_wrapping());

    (0..(9 * 14))
        .map(|idx| {
            let x = idx % 14;
            let y = idx / 14;

            (idx, x as i32, y as i32)
        })
        .map(move |(idx, x, y)| {
            let sprite_vram =
                sprite_loader.get_vram_sprite(&SPRITES[(idx + offset) % SPRITES.len()]);
            ObjectUnmanaged::new(sprite_vram)
                .set_affine_matrix(matrix.clone())
                .show_affine(AffineMode::Affine)
                .set_position((x * 16 + 8, y * 16 + 8).into())
        })
        .collect()
}

fn all_tags(sprite_loader: &mut SpriteLoader, image: usize) -> Vec<ObjectUnmanaged> {
    TAG_MAP
        .values()
        .enumerate()
        .map(move |(i, v)| {
            let x = (i % 7) as i32;
            let y = (i / 7) as i32;
            let sprite = v.animation_sprite(image);
            let (size_x, size_y) = sprite.size().to_width_height();
            let (size_x, size_y) = (size_x as i32, size_y as i32);
            let sprite_vram = sprite_loader.get_vram_sprite(sprite);
            ObjectUnmanaged::new(sprite_vram)
                .set_position((x * 32 + 16 - size_x / 2, y * 32 + 16 - size_y / 2).into())
        })
        .collect()
}

fn run_with_sprite_generating_fn<F>(
    oam: &mut OamUnmanaged,
    sprite_loader: &mut SpriteLoader,
    mut f: F,
) where
    F: FnMut(&mut SpriteLoader) -> Vec<ObjectUnmanaged>,
{
    let mut button = ButtonController::new();
    let vblank = VBlank::get();

    loop {
        let sprites = f(sprite_loader);
        vblank.wait_for_vblank();
        button.update();
        if button.is_just_pressed(Button::A) {
            break;
        }
        oam.iter().set(sprites);
    }
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (mut oam_manager, mut sprite_loader) = gba.display.object.get();

    loop {
        let mut count = 0;
        run_with_sprite_generating_fn(&mut oam_manager, &mut sprite_loader, |sprite_loader| {
            count += 1;
            all_tags(sprite_loader, count / 5)
        });
        run_with_sprite_generating_fn(&mut oam_manager, &mut sprite_loader, |sprite_loader| {
            count += 1;
            all_sprites(sprite_loader, num!(0.), count / 5)
        });
        let mut rotation = num!(0.);
        run_with_sprite_generating_fn(&mut oam_manager, &mut sprite_loader, |sprite_loader| {
            count += 1;
            rotation += num!(0.01);
            all_sprites(sprite_loader, rotation, count / 5)
        });
    }
}
