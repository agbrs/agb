use core::{arch::asm, ops::Index};

use alloc::vec::Vec;

// only works for code compiled as THUMB
#[repr(C)]
#[derive(Clone, Default, Debug)]
struct Context {
    registers: [u32; 11],
}

pub struct Frames {
    frames: Vec<u32>,
}

#[allow(unused)]
enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    FP,
    SP,
    LR,
    PC,
}

impl Index<Register> for Context {
    type Output = u32;

    fn index(&self, index: Register) -> &Self::Output {
        &self.registers[index as usize]
    }
}

#[inline(never)]
pub(crate) fn unwind_exception() -> Frames {
    let mut context = Context::default();

    unsafe {
        let context_ptr = (&mut context) as *mut _;

        asm!(
            "
            str r0, [r0, #0x00]
            str r1, [r0, #0x04]
            str r2, [r0, #0x08]
            str r3, [r0, #0x0C]
            str r4, [r0, #0x10]
            str r5, [r0, #0x14]
            str r6, [r0, #0x18]
            str r7, [r0, #0x1C]
            mov r7, sp
            str r7, [r0, #0x20]
            mov r7, lr
            str r7, [r0, #0x24]
            mov r7, pc
            str r7, [r0, #0x28]
            ldr r7, [r0, #0x1C]
            ",
            in("r0") context_ptr
        );
    }

    let mut frame_pointer = context[Register::FP];

    let mut frames = Vec::new();

    loop {
        let sp = unsafe { *(frame_pointer as *const u32) };
        let lr = unsafe { *((frame_pointer as *const u32).add(1)) };

        if sp == 0 {
            break;
        }

        // need to subtract 2 here since the link register points to the _next_ instruction
        // to execute, not the one that is being branched from which is the one we care about
        // in the stack trace.
        frames.push(lr - 2);

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
