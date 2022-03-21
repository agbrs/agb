#![no_std]
#![no_main]

use core::cell::RefCell;

use bare_metal::{CriticalSection, Mutex};

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    let count = Mutex::new(RefCell::new(0));
    let _a = agb::interrupt::add_interrupt_handler(
        agb::interrupt::Interrupt::VBlank,
        |key: &CriticalSection| {
            let mut count = count.borrow(*key).borrow_mut();
            agb::println!("Hello, world, frame = {}", *count);
            *count += 1;
        },
    );
    loop {}
}
