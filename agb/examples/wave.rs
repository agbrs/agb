#![no_std]
#![no_main]

extern crate agb;

use agb::{
    display::example_logo,
    interrupt::{Interrupt, Mutex},
    number::FixedNum,
};

struct BackCosines {
    cosines: [u16; 32],
    row: usize,
}

#[no_mangle]
pub fn main() -> ! {
    let mut gba = agb::Gba::new();
    let mut gfx = gba.display.video.tiled0();

    gfx.set_background_palettes(example_logo::PALETTE_DATA);
    gfx.set_background_tilemap(0, example_logo::TILE_DATA);

    let mut back = gfx.get_background().unwrap();

    let mut entries: [u16; 30 * 20] = [0; 30 * 20];
    for tile_id in 0..(30 * 20) {
        let palette_entry = example_logo::PALETTE_ASSIGNMENT[tile_id as usize] as u16;
        entries[tile_id as usize] = tile_id | (palette_entry << 12);
    }

    back.draw_full_map(&entries, (30_u32, 20_u32).into(), 0);
    back.show();

    let mut time = 0;
    let cosines = [0_u16; 32];

    let back = Mutex::new(BackCosines { cosines, row: 0 });

    agb::add_interrupt_handler!(Interrupt::HBlank, |_| {
        let mut backc = back.lock();
        let deflection = backc.cosines[backc.row % 32];
        unsafe { ((0x0400_0010) as *mut u16).write_volatile(deflection) }
        backc.row += 1;
    });

    let vblank = agb::interrupt::VBlank::get();

    loop {
        vblank.wait_for_vblank();
        let mut backc = back.lock();
        backc.row = 0;
        time += 1;
        for (r, a) in backc.cosines.iter_mut().enumerate() {
            let n: FixedNum<8> = (FixedNum::new(r as i32) / 32 + FixedNum::new(time) / 128).cos()
                * (256 * 4 - 1)
                / 256;
            *a = (n.trunc() % (32 * 8)) as u16;
        }
    }
}
