#![no_std]
#![no_main]

use core::cell::RefCell;

use agb::{
    display::{
        example_logo,
        tiled::{RegularBackgroundSize, TileFormat},
    },
    fixnum::FixedNum,
    interrupt::{free, Interrupt},
};
use bare_metal::{CriticalSection, Mutex};

struct BackCosines {
    cosines: [u16; 32],
    row: usize,
}

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    let (gfx, mut vram) = gba.display.video.tiled0();

    let mut background = gfx.background(
        agb::display::Priority::P0,
        RegularBackgroundSize::Background32x32,
        TileFormat::FourBpp,
    );

    example_logo::display_logo(&mut background, &mut vram);

    let mut time = 0;
    let cosines = [0_u16; 32];

    let back = Mutex::new(RefCell::new(BackCosines { cosines, row: 0 }));

    let _a = agb::interrupt::add_interrupt_handler(Interrupt::HBlank, |key: CriticalSection| {
        let mut back = back.borrow(key).borrow_mut();
        let deflection = back.cosines[back.row % 32];
        unsafe { ((0x0400_0010) as *mut u16).write_volatile(deflection) }
        back.row += 1;
    });

    let vblank = agb::interrupt::VBlank::get();

    loop {
        vblank.wait_for_vblank();
        free(|key| {
            let mut back = back.borrow(key).borrow_mut();
            back.row = 0;
            time += 1;
            for (r, a) in back.cosines.iter_mut().enumerate() {
                let n: FixedNum<8> = (FixedNum::new(r as i32) / 32 + FixedNum::new(time) / 128)
                    .cos()
                    * (256 * 4 - 1)
                    / 256;
                *a = (n.trunc() % (32 * 8)) as u16;
            }
        })
    }
}
