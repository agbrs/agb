#![no_std]
#![no_main]

use agb::sync::Static;

#[agb::entry]
fn main(_gba: agb::Gba) -> ! {
    let count = Static::new(0);
    let _a = agb::interrupt::add_interrupt_handler(
        agb::interrupt::Interrupt::VBlank,
        |_| {
            let cur_count = count.read();
            agb::println!("Hello, world, frame = {}", cur_count);
            count.write(cur_count + 1);
        },
    );
    loop {}
}
