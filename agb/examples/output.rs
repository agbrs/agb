#![no_std]
#![no_main]

use agb::syscall;
use portable_atomic::{AtomicU32, Ordering};

static COUNT: AtomicU32 = AtomicU32::new(0);

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    let _a = unsafe {
        agb::interrupt::add_interrupt_handler(agb::interrupt::Interrupt::VBlank, |_| {
            let cur_count = COUNT.load(Ordering::SeqCst);
            agb::println!("Hello, world, frame = {}", cur_count);
            COUNT.store(cur_count + 1, Ordering::SeqCst);
        })
    };
    loop {
        syscall::halt();
    }
}
