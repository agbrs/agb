#![no_std]
#![no_main]

extern crate alloc;

use alloc::boxed::Box;

#[agb::entry]
fn main() -> ! {
    loop {
        let b = Box::new(1);
        agb::println!("dynamic allocation made to {:?}", &*b as *const _);
    }
}
