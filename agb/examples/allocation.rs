#![no_std]
#![no_main]
#![feature(allocator_api)]

extern crate alloc;

use alloc::boxed::Box;

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    loop {
        let a = Box::new_in(1, agb::ExternalAllocator);
        let b = Box::new(1);
        let c = Box::new_in(3, agb::InternalAllocator);
        agb::println!("ewram allocation made to {:?}", &*a as *const _);
        agb::println!("global allocation made to {:?}", &*b as *const _);
        agb::println!("iwram allocation made to {:?}", &*c as *const _);
    }
}
