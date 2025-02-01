#![no_std]
#![no_main]

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    let mut input = agb::input::ButtonController::new();

    loop {
        input.update();
        // if A is pressed, draw out of range
        if input.is_just_pressed(agb::input::Button::A) {
            do_some_panic();
        }
        if input.is_just_pressed(agb::input::Button::B) {
            #[allow(arithmetic_overflow)]
            let _p = i32::MAX + 1;
        }
    }
}

#[inline(never)]
fn do_some_panic() {
    panic!("This is an example panic");
}
