use core::arch::asm;

// use crate::display::object::AffineMatrixAttributes;
use crate::fixnum::Num;

#[allow(non_snake_case)]

pub fn halt() {
    unsafe {
        asm!(
            "swi 0x02",
            lateout("r0") _,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
}

pub fn stop() {
    unsafe {
        asm!(
            "swi 0x03",
            lateout("r0") _,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
}

pub fn wait_for_interrupt() {
    unsafe {
        asm!(
            "swi 0x04",
            lateout("r0") _,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
}

/// The vblank interrupt handler [VBlank][crate::interrupt::VBlank] should be
/// used instead of calling this function directly.
pub fn wait_for_vblank() {
    unsafe {
        asm!(
            "swi 0x05",
            lateout("r0") _,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
}

pub fn div(numerator: i32, denominator: i32) -> (i32, i32, i32) {
    let divide: i32;
    let modulo: i32;
    let abs_divide: i32;
    unsafe {
        asm!(
            "swi 0x06",
            in("r0") numerator,
            in("r1") denominator,
            lateout("r0") divide,
            lateout("r1") modulo,
            lateout("r3") abs_divide,
        );
    }
    (divide, modulo, abs_divide)
}

pub fn sqrt(n: i32) -> i32 {
    let result: i32;
    unsafe {
        asm!(
            "swi 0x08",
            in("r0") n,
            lateout("r0") result,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
    result
}

pub fn arc_tan(n: i16) -> i16 {
    let result: i16;
    unsafe {
        asm!(
            "swi 0x09",
            in("r0") n,
            lateout("r0") result,
        );
    }
    result
}

pub fn arc_tan2(x: i16, y: i32) -> i16 {
    let result: i16;
    unsafe {
        asm!(
            "swi 0x09",
            in("r0") x,
            in("r1") y,
            lateout("r0") result,

        );
    }
    result
}

// pub fn affine_matrix(
//     x_scale: Num<i16, 8>,
//     y_scale: Num<i16, 8>,
//     rotation: u8,
// ) -> AffineMatrixAttributes {
//     let mut result = AffineMatrixAttributes {
//         p_a: 0,
//         p_b: 0,
//         p_c: 0,
//         p_d: 0,
//     };

//     #[allow(dead_code)]
//     #[repr(C, packed)]
//     struct Input {
//         x_scale: i16,
//         y_scale: i16,
//         rotation: u16,
//     }

//     let input = Input {
//         y_scale: x_scale.to_raw(),
//         x_scale: y_scale.to_raw(),
//         rotation: rotation as u16,
//     };

//     unsafe {
//         asm!("swi 0x0F",
//             in("r0") &input as *const Input as usize,
//             in("r1") &mut result as *mut AffineMatrixAttributes as usize,
//             in("r2") 1,
//             in("r3") 2,
//         )
//     }

//     result
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test_case]
//     fn affine(_gba: &mut crate::Gba) {
//         // expect identity matrix
//         let one: Num<i16, 8> = 1.into();

//         let aff = affine_matrix(one, one, 0);
//         assert_eq!(aff.p_a, one.to_raw());
//         assert_eq!(aff.p_d, one.to_raw());
//     }
// }
