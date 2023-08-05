#![no_std]
#![no_main]

use agb::save::Error;

extern crate alloc;
use alloc::vec::Vec;

#[agb::entry]
fn main(mut gba: agb::Gba) -> ! {
    test_save(gba).unwrap();
    panic!("example finished");
}

fn test_save(mut gba: agb::Gba) -> Result<(), Error> {
    gba.save.init_sram();
    let mut access = gba.save.access()?;

    let mut is_save = 0;
    access.read(0, core::slice::from_mut(&mut is_save))?;

    if is_save != 0 {
        access
            .prepare_write(0..128)?
            .write(0, &(0..128).collect::<Vec<_>>())?;
    }

    Ok(())
}
