//! The no game screen is what is displayed if there isn't a game made yet.

use agb_fixnum::{num, Num, Vector2D};
use alloc::vec;
use alloc::vec::Vec;

use crate::{
    display::{
        object::{OamIterator, ObjectUnmanaged, Sprite},
        HEIGHT, WIDTH,
    },
    include_aseprite,
    interrupt::VBlank,
};

const SQUARES: &[Sprite] = include_aseprite!("gfx/square.aseprite").sprites();

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
            (num!(0.5), num!(0.5)).into(),
            (num!(2.5), num!(0.5)).into(),
            (num!(1.), num!(1.)).into(),
            (num!(2.), num!(1.)).into(),
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

trait Renderable {
    fn render(&self, slots: &mut OamIterator) -> Option<()>;
}

impl Renderable for ObjectUnmanaged {
    fn render(&self, slots: &mut OamIterator) -> Option<()> {
        slots.next()?.set(self);
        Some(())
    }
}

impl<T: Renderable> Renderable for &[T] {
    fn render(&self, slots: &mut OamIterator) -> Option<()> {
        for r in self.iter() {
            r.render(slots)?;
        }

        Some(())
    }
}

pub fn no_game(mut gba: crate::Gba) -> ! {
    let (mut oam, mut loader) = gba.display.object.get_unmanaged();

    let squares: Vec<_> = SQUARES
        .iter()
        .map(|sprite| loader.get_vram_sprite(sprite))
        .collect();

    let mut letter_positons = Vec::new();

    let square_positions = {
        let mut s = letters();
        for letter in s.iter_mut() {
            letter.sort_by_key(|a| a.manhattan_distance());
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

    // let (_background, mut vram) = gba.display.video.tiled0();

    // vram.set_background_palettes(&[Palette16::new([u16::MAX; 16])]);

    let vblank = VBlank::get();

    loop {
        let mut rng = crate::rng::RandomNumberGenerator::new();
        time += time_delta;
        time %= 1;
        let letters: Vec<ObjectUnmanaged> = letter_positons
            .iter()
            .enumerate()
            .map(|(idx, position)| {
                let time = time + Num::<i32, 8>::new(idx as i32) / 128;
                *position + Vector2D::new(time.sin(), time.cos()) * 10
            })
            .map(|pos| {
                let mut obj =
                    ObjectUnmanaged::new(squares[rng.gen() as usize % squares.len()].clone());
                obj.show().set_position(pos.floor());
                obj
            })
            .collect();

        vblank.wait_for_vblank();
        for (obj, slot) in letters.iter().zip(oam.iter()) {
            slot.set(obj);
        }
    }
}
