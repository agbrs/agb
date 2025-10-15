#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(agb::test_runner::test_runner)]

use agb::{
    fixnum::{Num, Vector2D},
    save::Save,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Eq, PartialEq, Debug)]
struct MySaveGame {
    a: Vector2D<Num<i32, 8>>,
}

#[test_case]
fn test_save_serde(gba: &mut agb::Gba) {
    let enigne = gba.save.init_sram();

    let mut save =
        Save::new(gba, enigne, "my_id".as_bytes()).expect("Should be able to initialise save");
    let game = MySaveGame {
        a: (127, 23).into(),
    };

    assert!(save.load().is_err());

    save.save(&game).expect("Save should work");
    assert_eq!(save.load(), Ok(game));
}

#[agb::entry]
fn entry(_gba: agb::Gba) -> ! {
    loop {
        agb::syscall::halt();
    }
}
