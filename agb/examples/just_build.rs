#![no_std]
#![no_main]

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

// implementation of tonc's "My first GBA demo"
// https://coranac.com/tonc/text/first.htm

#[no_mangle]
pub fn main() -> ! {
    unsafe {
        *(0x0400_0000 as *mut u32) = 0x0403;
        let video = 0x0600_0000 as *mut u16;
        *video.offset(120 + 80 * 240) = 0x001F;
        *video.offset(136 + 80 * 240) = 0x03E0;
        *video.offset(120 + 96 * 240) = 0x7C00;
    }
    loop {}
}
