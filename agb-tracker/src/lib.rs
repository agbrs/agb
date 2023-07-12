#![no_std]
#![no_main]
// This is required to allow writing tests
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]

#[cfg(test)]
mod tests {
    #[test_case]
    fn it_works(_gba: &mut agb::Gba) {
        assert_eq!(1, 1);
    }
}

#[cfg(test)]
#[agb::entry]
fn main(gba: agb::Gba) -> ! {
    loop {}
}
