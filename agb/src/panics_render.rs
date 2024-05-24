use core::{fmt::Write, panic::PanicInfo};

use alloc::{format, vec::Vec};

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
        None => "https://agbrs.dev/crash#",
    }
};

pub fn render_backtrace(trace: &backtrace::Frames, info: &PanicInfo) -> ! {
    critical_section::with(|_cs| {
        dma3_exclusive(|| {
            // SAFETY: This is not fine, but we're crashing anyway. The loop at the end should stop anything bad happening
            let mut gba = unsafe { crate::Gba::new_in_entry() };

            gba.dma.dma().dma3.disable();
            let mut gfx = gba.display.video.bitmap3();
            gfx.clear(0xFFFF);

            let qrcode_string_data = if WEBSITE.is_empty() {
                format!("{trace}")
            } else {
                format!("{WEBSITE}{trace}")
            };
            crate::println!("Stack trace: {qrcode_string_data}");

            let location = draw_qr_code(&mut gfx, &qrcode_string_data);

            let mut trace_text_render =
                text::BitmapTextRender::new(&mut gfx, (location, 8).into(), 0x0000);
            let _ = writeln!(
                &mut trace_text_render,
                "The game crashed :({}{WEBSITE}\n{trace}",
                if WEBSITE.is_empty() { "" } else { "\n" }
            );

            let trace_location = trace_text_render.head_y_position();

            let mut panic_text_render = text::BitmapTextRender::new(
                &mut gfx,
                (8, location.max(trace_location + PADDING)).into(),
                0x0000,
            );
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
const PADDING: i32 = 8;

/// Returns the width / height of the QR code + padding in pixels
fn draw_qr_code(gfx: &mut Bitmap3<'_>, qrcode_string_data: &str) -> i32 {
    const MAX_VERSION: qrcodegen_no_heap::Version = qrcodegen_no_heap::Version::new(6);

    let (Ok(mut temp_buffer), Ok(mut out_buffer)) = (
        Vec::try_with_capacity_in(MAX_VERSION.buffer_len(), crate::ExternalAllocator),
        Vec::try_with_capacity_in(MAX_VERSION.buffer_len(), crate::ExternalAllocator),
    ) else {
        crate::println!("Failed to allocate memory to generate QR code");
        return PADDING;
    };

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
            return PADDING;
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

    qr_code.size() * 2 + PADDING * 2
}
