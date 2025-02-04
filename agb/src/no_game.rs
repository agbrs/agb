//! The no game screen is what is displayed if there isn't a game made yet.

use agb_fixnum::{num, Num, Vector2D};
use alloc::vec::Vec;
use alloc::{boxed::Box, vec};

use crate::display::object::{DynamicSprite, PaletteVramSingle, Size, SpriteVram};
use crate::display::palette16::Palette16;
use crate::{
    display::{object::Object, HEIGHT, WIDTH},
    include_palette,
    interrupt::VBlank,
};

static PALETTE: &[u16] = &include_palette!("gfx/pastel.png");

fn letters() -> Vec<Vec<Vector2D<Num<i32, 8>>>> {
    vec![
        // N
        vec![
            (num!(0.), num!(0.)).into(),
            (num!(1.), num!(1.)).into(),
            (num!(2.), num!(2.)).into(),
            (num!(3.), num!(3.)).into(),
            (num!(0.), num!(1.)).into(),
            (num!(0.), num!(2.)).into(),
            (num!(0.), num!(3.)).into(),
            (num!(3.), num!(0.)).into(),
            (num!(3.), num!(1.)).into(),
            (num!(3.), num!(2.)).into(),
            (num!(3.), num!(3.)).into(),
        ],
        // O
        vec![
            (num!(0.), num!(0.)).into(),
            (num!(0.), num!(1.)).into(),
            (num!(0.), num!(2.)).into(),
            (num!(0.), num!(3.)).into(),
            (num!(1.), num!(3.)).into(),
            (num!(2.), num!(3.)).into(),
            (num!(3.), num!(3.)).into(),
            (num!(3.), num!(2.)).into(),
            (num!(3.), num!(1.)).into(),
            (num!(3.), num!(0.)).into(),
            (num!(2.), num!(0.)).into(),
            (num!(1.), num!(0.)).into(),
        ],
        // G
        vec![
            (num!(3.), num!(0.)).into(),
            (num!(2.), num!(0.)).into(),
            (num!(1.), num!(0.)).into(),
            (num!(0.), num!(0.)).into(),
            (num!(0.), num!(1.)).into(),
            (num!(0.), num!(2.)).into(),
            (num!(0.), num!(3.)).into(),
            (num!(1.), num!(3.)).into(),
            (num!(2.), num!(3.)).into(),
            (num!(3.), num!(3.)).into(),
            (num!(3.), num!(2.25)).into(),
            (num!(3.), num!(1.5)).into(),
            (num!(2.), num!(1.5)).into(),
        ],
        // A
        vec![
            (num!(0.), num!(0.)).into(),
            (num!(0.), num!(1.)).into(),
            (num!(0.), num!(2.)).into(),
            (num!(0.), num!(3.)).into(),
            (num!(3.), num!(3.)).into(),
            (num!(3.), num!(2.)).into(),
            (num!(3.), num!(1.)).into(),
            (num!(3.), num!(0.)).into(),
            (num!(2.), num!(0.)).into(),
            (num!(1.), num!(0.)).into(),
            (num!(1.), num!(1.5)).into(),
            (num!(2.), num!(1.5)).into(),
        ],
        // M
        vec![
            (num!(0.), num!(0.)).into(),
            (num!(0.), num!(1.)).into(),
            (num!(0.), num!(2.)).into(),
            (num!(0.), num!(3.)).into(),
            (num!(3.), num!(3.)).into(),
            (num!(3.), num!(2.)).into(),
            (num!(3.), num!(1.)).into(),
            (num!(3.), num!(0.)).into(),
            (num!(1.5), num!(1.5)).into(),
            (num!(0.75), num!(0.75)).into(),
            (num!(2.25), num!(0.75)).into(),
        ],
        // E
        vec![
            (num!(0.), num!(0.)).into(),
            (num!(0.), num!(1.)).into(),
            (num!(0.), num!(2.)).into(),
            (num!(0.), num!(3.)).into(),
            (num!(1.), num!(3.)).into(),
            (num!(2.), num!(3.)).into(),
            (num!(3.), num!(3.)).into(),
            (num!(3.), num!(0.)).into(),
            (num!(2.), num!(0.)).into(),
            (num!(1.), num!(0.)).into(),
            (num!(1.), num!(1.5)).into(),
            (num!(2.), num!(1.5)).into(),
        ],
    ]
}

fn generate_sprites() -> Box<[SpriteVram]> {
    let mut sprites = Vec::new();

    // generate palettes

    let palettes: Vec<PaletteVramSingle> = PALETTE
        .chunks(15)
        .map(|x| {
            core::iter::once(0)
                .chain(x.iter().copied())
                .chain(core::iter::repeat(0))
                .take(16)
                .collect::<Vec<_>>()
        })
        .map(|palette| {
            let palette = Palette16::new(palette.try_into().unwrap());
            PaletteVramSingle::new(&palette).unwrap()
        })
        .collect();

    // generate sprites

    for (palette, colour) in (0..PALETTE.len()).map(|x| (x / 15, x % 15)) {
        let mut sprite = DynamicSprite::new(Size::S8x8);
        sprite.clear(colour + 1);
        sprites.push(sprite.to_vram(palettes[palette].clone()));
    }

    sprites.into_boxed_slice()
}

pub fn no_game(mut gba: crate::Gba) -> ! {
    let mut gfx: crate::display::Graphics<'_> = gba.display.graphics.get();

    let squares = generate_sprites();

    let mut letter_positons = Vec::new();

    let square_positions = {
        let mut s = letters();
        for letter in s.iter_mut() {
            letter.sort_by_key(|a| a.magnitude_squared());
        }
        s
    };
    for (letter_idx, letter_parts) in square_positions.iter().enumerate() {
        for part in letter_parts.iter() {
            let position = part
                .hadamard((8, 10).into())
                .hadamard((num!(3.) / 2, num!(3.) / 2).into());

            let letter_pos = Vector2D::new(
                60 * (1 + letter_idx as i32 - ((letter_idx >= 2) as i32 * 3)),
                70 * ((letter_idx >= 2) as i32),
            );

            letter_positons.push(position + letter_pos.change_base());
        }
    }

    let bottom_right = letter_positons
        .iter()
        .copied()
        .max_by_key(|x| x.manhattan_distance())
        .unwrap();

    let difference = (Vector2D::new(WIDTH - 8, HEIGHT - 8).change_base() - bottom_right) / 2;

    for pos in letter_positons.iter_mut() {
        *pos += difference;
    }

    let mut time: Num<i32, 8> = num!(0.);
    let time_delta: Num<i32, 8> = num!(0.025);

    let vblank = VBlank::get();

    loop {
        time += time_delta;
        time %= 1;
        let letters: Vec<Object> = letter_positons
            .iter()
            .enumerate()
            .map(|(idx, position)| {
                let time = time + Num::<i32, 8>::new(idx as i32) / 128;
                (idx, *position + Vector2D::new(time.sin(), time.cos()) * 10)
            })
            .map(|(idx, pos)| {
                let mut obj = Object::new(squares[idx % squares.len()].clone());
                obj.set_position(pos.floor());
                obj
            })
            .collect();

        let mut frame = gfx.frame();

        for obj in letters.iter() {
            obj.show(&mut frame);
        }

        vblank.wait_for_vblank();

        frame.commit();
    }
}
