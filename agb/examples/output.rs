#![no_std]
#![no_main]

use agb::sync::Static;

static COUNT: Static<u32> = Static::new(0);

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    let _a = unsafe {
        agb::interrupt::add_interrupt_handler(agb::interrupt::Interrupt::VBlank, || {
            let cur_count = COUNT.read();
            agb::println!("Hello, world, frame = {}", cur_count);
            COUNT.write(cur_count + 1);
        })
    };
    loop {}
}
