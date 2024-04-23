use core::{fmt::Write, panic::PanicInfo};

use alloc::{format, vec};

use crate::{
    backtrace,
    display::{bitmap3::Bitmap3, busy_wait_for_vblank, HEIGHT, WIDTH},
    dma::dma3_exclusive,
    mgba, syscall,
};

mod text;

static WEBSITE: &str = {
    match core::option_env!("AGBRS_BACKTRACE_WEBSITE") {
        Some(x) => x,
        None => "",
    }
};

pub fn render_backtrace(trace: &backtrace::Frames, info: &PanicInfo) -> ! {
    critical_section::with(|_cs| {
        dma3_exclusive(|| {
            // SAFETY: This is not fine, but we're crashing anyway. The loop at the end should stop anything bad happening
            let mut gba = unsafe { crate::Gba::new_in_entry() };

            gba.dma.dma().dma3.disable();
            let mut gfx = gba.display.video.bitmap3();

            let qrcode_string_data = if WEBSITE.is_empty() {
                format!("{trace}")
            } else {
                format!("{WEBSITE}#{trace}")
            };
            crate::println!("Stack trace: {qrcode_string_data}");

            let location = draw_qr_code(&mut gfx, &qrcode_string_data);

            let mut trace_text_render =
                text::BitmapTextRender::new(&mut gfx, (location, 8).into(), 0x0000);
            let _ = write!(
                &mut trace_text_render,
                "The game crashed :({}{WEBSITE}\n{trace}",
                if WEBSITE.is_empty() { "" } else { "\n" }
            );

            let mut panic_text_render =
                text::BitmapTextRender::new(&mut gfx, (8, location).into(), 0x0000);
            let _ = write!(&mut panic_text_render, "{info}");

            // need to wait 2 frames to ensure that mgba finishes rendering before the fatal call below
            busy_wait_for_vblank();
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

/// Returns the width / height of the QR code + padding in pixels
fn draw_qr_code(gfx: &mut Bitmap3<'_>, qrcode_string_data: &str) -> i32 {
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
        None,
        true,
    ) {
        Ok(qr_code) => qr_code,
        Err(e) => {
            crate::println!("Error generating qr code: {e:?}");
            return 8;
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

    qr_code.size() * 2 + 8 * 2
}
