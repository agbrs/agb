#![no_std]
#![no_main]

use core::cell::RefCell;

use agb::{
    display::example_logo,
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
    let mut gfx = gba.display.video.tiled0();

    example_logo::display_logo(&mut gfx);

    let mut time = 0;
    let cosines = [0_u16; 32];

    let back = Mutex::new(RefCell::new(BackCosines { cosines, row: 0 }));

    agb::add_interrupt_handler!(Interrupt::HBlank, |key: &CriticalSection| {
        let mut backc = back.borrow(*key).borrow_mut();
        let deflection = backc.cosines[backc.row % 32];
        unsafe { ((0x0400_0010) as *mut u16).write_volatile(deflection) }
        backc.row += 1;
    });

    let vblank = agb::interrupt::VBlank::get();

    loop {
        vblank.wait_for_vblank();
        free(|key| {
            let mut backc = back.borrow(*key).borrow_mut();
            backc.row = 0;
            time += 1;
            for (r, a) in backc.cosines.iter_mut().enumerate() {
                let n: FixedNum<8> = (FixedNum::new(r as i32) / 32 + FixedNum::new(time) / 128)
                    .cos()
                    * (256 * 4 - 1)
                    / 256;
                *a = (n.trunc() % (32 * 8)) as u16;
            }
        })
    }
}
