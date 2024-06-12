#![no_std]
#![no_main]

use agb::{
    display::{
        affine::AffineMatrix,
        object::{
            AffineMatrixInstance, AffineMode, ObjectTextRender, ObjectUnmanaged, PaletteVram, Size,
            SpriteVram, TextAlignment,
        },
        palette16::Palette16,
        Font, HEIGHT, WIDTH,
    },
    include_font,
};
use agb_fixnum::{num, Num, Vector2D};
use alloc::vec::Vec;

extern crate alloc;

const FONT: Font = include_font!("examples/font/yoster.ttf", 12);
#[agb::entry]
fn entry(gba: agb::Gba) -> ! {
    main(gba);
}

fn text_objects(
    font: &Font,
    sprite_size: Size,
    palette: PaletteVram,
    text_alignment: TextAlignment,
    width: i32,
    paragraph_spacing: i32,
    arguments: core::fmt::Arguments,
) -> Vec<(SpriteVram, Vector2D<i32>)> {
    let text = alloc::format!("{}\n", arguments);
    let mut wr = ObjectTextRender::new(text, font, sprite_size, palette, None);

    wr.layout(width, text_alignment, paragraph_spacing);
    wr.render_all();

    wr.letter_groups()
        .map(|x| (x.sprite().clone(), x.relative_position()))
        .collect()
}

fn main(mut gba: agb::Gba) -> ! {
    let (mut unmanaged, _sprites) = gba.display.object.get_unmanaged();

    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    palette[2] = 0x00_FF;
    let palette = Palette16::new(palette);
    let palette = PaletteVram::new(&palette).unwrap();

    let groups: Vec<_> = text_objects(
        &FONT,
        Size::S16x16,
        palette,
        TextAlignment::Center,
        WIDTH,
        0,
        format_args!("Woah, ROTATION!"),
    )
    .into_iter()
    .map(|x| (x.0, x.1 - (WIDTH / 2, 0).into() + (8, 4).into()))
    .collect();

    let vblank = agb::interrupt::VBlank::get();
    let mut angle: Num<i32, 8> = num!(0.);

    loop {
        angle += num!(0.01);
        if angle >= num!(1.) {
            angle -= num!(1.);
        }

        let rotation_matrix = AffineMatrix::from_rotation(angle);

        let letter_group_rotation_matrix_instance =
            AffineMatrixInstance::new(AffineMatrix::from_rotation(-angle).to_object_wrapping());

        let frame_positions: Vec<_> = groups
            .iter()
            .map(|x| {
                let mat = AffineMatrix::from_translation(x.1.change_base());

                let mat = rotation_matrix * mat;

                let position = mat.position() + (WIDTH / 2, HEIGHT / 2).into() - (16, 16).into();

                let mut object = ObjectUnmanaged::new(x.0.clone());
                object.set_affine_matrix(letter_group_rotation_matrix_instance.clone());
                object.show_affine(AffineMode::AffineDouble);
                object.set_position(position.floor());

                object
            })
            .collect();

        vblank.wait_for_vblank();
        let mut oam = unmanaged.iter();
        for (object, oam_slot) in frame_positions.into_iter().zip(&mut oam) {
            oam_slot.set(&object);
        }
    }
}
