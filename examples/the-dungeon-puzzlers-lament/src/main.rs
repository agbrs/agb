#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    the_dungeon_puzzlers_lament::entry(gba);
}
