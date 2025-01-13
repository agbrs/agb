use core::arch::asm;

use alloc::vec::Vec;

pub struct Frames {
    frames: Vec<u32>,
}

#[inline(never)]
pub(crate) fn unwind_exception() -> Frames {
    let mut frame_pointer = unsafe {
        let mut frame_pointer: u32 = 0;
        let ptr = &mut frame_pointer as *mut _;
        #[cfg(target_feature = "thumb-mode")]
        asm!(
            "
            str r7, [r0, 0]
            ",
            in("r0") ptr
        );
        #[cfg(not(target_feature = "thumb-mode"))]
        asm!(
            "
            str r11, [r0, 0]
            ",
            in("r0") ptr
        );
        frame_pointer
    };

    let mut frames = Vec::new();

    loop {
        let sp = unsafe { *(frame_pointer as *const u32) };
        let lr = unsafe { *((frame_pointer as *const u32).add(1)) };

        if sp == 0 {
            break;
        }

        let is_thumb = lr & 1 == 1;
        let instruction_size = if is_thumb { 2 } else { 4 };

        // need to subtract instruction_size here since the link register points
        // to the _next_ instruction to execute, not the one that is being
        // branched from which is the one we care about in the stack trace.
        frames.push(lr - instruction_size);

        frame_pointer = sp;
    }

    Frames { frames }
}

impl core::fmt::Display for Frames {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for frame in &self.frames {
            if frame & 0xFFFF_0000 == 0x0800_0000 {
                let frame = *frame as u16; // intentionally truncate
                let frame_encoded = gwilym_encoding::encode_16(frame);
                let frame_str = unsafe { core::str::from_utf8_unchecked(&frame_encoded) };

                write!(f, "{frame_str}")?;
            } else {
                let frame_encoded = gwilym_encoding::encode_32(*frame);
                let frame_str = unsafe { core::str::from_utf8_unchecked(&frame_encoded) };

                write!(f, "{frame_str}")?;
            }
        }

        write!(f, "v1")
    }
}

mod gwilym_encoding {
    static ALPHABET: &[u8] = b"0123456789=ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";

    pub fn encode_16(input: u16) -> [u8; 3] {
        let input = input as usize;
        [
            ALPHABET[input >> (16 - 5)],
            ALPHABET[(input >> (16 - 10)) & 0b11111],
            ALPHABET[input & 0b111111],
        ]
    }

    pub fn encode_32(input: u32) -> [u8; 6] {
        let input = input as usize;
        let output_lower_16 = encode_16(input as u16);
        let input_upper_16 = input >> 16;
        [
            ALPHABET[(input_upper_16 >> (16 - 5)) | (1 << 5)],
            ALPHABET[(input_upper_16 >> (16 - 10)) & 0b11111],
            ALPHABET[input_upper_16 & 0b111111],
            output_lower_16[0],
            output_lower_16[1],
            output_lower_16[2],
        ]
    }
}
