use core::{arch::asm, ops::Index};

use alloc::vec::Vec;

// only works for code compiled as THUMB
#[repr(C)]
#[derive(Clone, Default, Debug)]
struct Context {
    registers: [u32; 11],
}

pub struct Frame {
    pub address: u32,
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
pub(crate) fn unwind_exception() -> Vec<Frame> {
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

        frames.push(Frame { address: lr });

        frame_pointer = sp;
    }

    frames
}
