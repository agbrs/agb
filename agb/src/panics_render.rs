use core::{fmt::Write, panic::PanicInfo};

use alloc::{collections::TryReserveError, format, vec::Vec};
use qrcodegen_no_heap::DataTooLong;

use crate::{
    ExternalAllocator, backtrace,
    display::{HEIGHT, WIDTH, bitmap3::Bitmap3, busy_wait_for_vblank},
    dma::dma3_exclusive,
    mgba,
};

mod text;

static WEBSITE: &str = {
    match core::option_env!("AGBRS_BACKTRACE_WEBSITE") {
        Some(x) => x,
        None => "https://agbrs.dev/crash#",
    }
};

pub fn render_backtrace(trace: &backtrace::Frames, info: &PanicInfo) -> ! {
    dma3_exclusive(|| {
        // SAFETY: This is not fine, but we're crashing anyway. The loop at the end should stop anything bad happening
        unsafe { crate::dma::Dma::new(3) }.disable();

        // SAFETY: Again, not fine, but we're crashing anyway so we can clobber VRam if we need to
        let mut gfx = unsafe { Bitmap3::new() };
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
            crate::halt();
        }
    })
}
const PADDING: i32 = 8;

struct QrCodeBuffers {
    temp_buffer: Vec<u8, ExternalAllocator>,
    out_buffer: Vec<u8, ExternalAllocator>,
    version: qrcodegen_no_heap::Version,
}

impl QrCodeBuffers {
    fn new(version: qrcodegen_no_heap::Version) -> Result<Self, TryReserveError> {
        let buffer_length = version.buffer_len();
        let mut temp = Vec::try_with_capacity_in(buffer_length, ExternalAllocator)?;
        let mut out = Vec::try_with_capacity_in(buffer_length, ExternalAllocator)?;
        temp.resize(buffer_length, 0);
        out.resize(buffer_length, 0);

        Ok(Self {
            temp_buffer: temp,
            out_buffer: out,
            version,
        })
    }

    fn generate_qr_code(&mut self, data: &str) -> Result<qrcodegen_no_heap::QrCode, DataTooLong> {
        qrcodegen_no_heap::QrCode::encode_text(
            data,
            &mut self.temp_buffer,
            &mut self.out_buffer,
            qrcodegen_no_heap::QrCodeEcc::Medium,
            qrcodegen_no_heap::Version::MIN,
            self.version,
            None,
            true,
        )
    }
}

/// Returns the width / height of the QR code + padding in pixels
fn draw_qr_code(gfx: &mut Bitmap3<'_>, qrcode_string_data: &str) -> i32 {
    const MAX_VERSION: qrcodegen_no_heap::Version = qrcodegen_no_heap::Version::new(6);

    let Ok(mut buffers) = QrCodeBuffers::new(MAX_VERSION) else {
        crate::println!("Failed to allocate memory to generate QR code");
        return PADDING;
    };

    let qr_code = buffers.generate_qr_code(qrcode_string_data);

    let qr_code = match qr_code {
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

#[cfg(test)]
mod tests {
    use super::QrCodeBuffers;

    const MAX_VERSION: qrcodegen_no_heap::Version = qrcodegen_no_heap::Version::new(6);

    #[test_case]
    fn check_qr_code_generation(_: &mut crate::Gba) {
        let mut buffers =
            QrCodeBuffers::new(MAX_VERSION).expect("should be able to allocate buffers");
        buffers
            .generate_qr_code("https://agbrs.dev/crash#09rESxF0r0Cz06hv1")
            .expect("should be able to generate qr code");
    }
}
