use core::panic::PanicInfo;

use alloc::{format, vec};

use crate::{
    backtrace,
    display::{busy_wait_for_vblank, HEIGHT, WIDTH},
    dma::dma3_exclusive,
    interrupt, mgba, syscall,
};

pub fn render_backtrace(trace: &backtrace::Frames, info: &PanicInfo) -> ! {
    interrupt::free(|_cs| {
        dma3_exclusive(|| {
            // SAFETY: This is not fine, but we're crashing anyway. The loop at the end should stop anything bad happening
            let mut gba = unsafe { crate::Gba::new_in_entry() };

            gba.dma.dma().dma3.disable();

            let qrcode_string_data = format!("https://agbrs.dev/crash#v1-{trace}");
            crate::println!("Stack trace: {qrcode_string_data}");

            draw_qr_code(&mut gba, &qrcode_string_data);

            busy_wait_for_vblank();

            if let Some(mut mgba) = mgba::Mgba::new() {
                let _ = mgba.print(format_args!("Error: {info}"), mgba::DebugLevel::Fatal);
            }

            loop {
                syscall::halt();
            }
        })
    })
}

fn draw_qr_code(gba: &mut crate::Gba, qrcode_string_data: &str) {
    let mut gfx = gba.display.video.bitmap3();

    const MAX_VERSION: qrcodegen_no_heap::Version = qrcodegen_no_heap::Version::new(6);

    let mut temp_buffer = vec![0; MAX_VERSION.buffer_len()];
    let mut out_buffer = vec![0; MAX_VERSION.buffer_len()];

    let qr_code = match qrcodegen_no_heap::QrCode::encode_text(
        qrcode_string_data,
        &mut temp_buffer,
        &mut out_buffer,
        qrcodegen_no_heap::QrCodeEcc::Medium,
        qrcodegen_no_heap::Version::MIN,
        MAX_VERSION,
        Some(qrcodegen_no_heap::Mask::new(0)),
        true,
    ) {
        Ok(qr_code) => qr_code,
        Err(e) => {
            crate::println!("Error generating qr code: {e:?}");
            return;
        }
    };

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let colour = if qr_code.get_module(x / 2 - 4, y / 2 - 4) {
                0x0000
            } else {
                0xFFFF
            };
            gfx.draw_point(x, y, colour);
        }
    }
}
