#![no_std]
#![no_main]

extern crate agb;
#[no_mangle]
pub fn main() -> ! {
    let count = agb::interrupt::Mutex::new(0);
    agb::add_interrupt_handler!(agb::interrupt::Interrupt::VBlank, |key| {
        let mut count = count.lock_with_key(&key);
        agb::println!("Hello, world, frame = {}", *count);
        *count += 1;
    });
    loop {}
}
