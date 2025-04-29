// This file exists to be `core::include!`ed into a doctest to allow it to
// actually run. You'll need to define a function called `test` with the signature
// fn(gba: agb::Gba) -> () and then that will run as part of the test.

#[agb::entry]
fn doctest_entry(gba: agb::Gba) -> ! {
    test(gba);

    agb::println!("Tests finished successfully");
    unreachable!("Should have stopped the runner by now");
}
