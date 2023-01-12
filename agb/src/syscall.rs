use agb_fixnum::Vector2D;
use core::arch::asm;
use core::mem::MaybeUninit;

use crate::display::affine::AffineMatrixBackground;
use crate::fixnum::Num;

#[allow(non_snake_case)]

const fn swi_map(thumb_id: u32) -> u32 {
    if cfg!(target_feature = "thumb-mode") {
        thumb_id
    } else {
        thumb_id << 16
    }
}

pub fn halt() {
    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x02) },
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
            "swi {SWI}",
            SWI = const { swi_map(0x03) },
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
            "swi {SWI}",
            SWI = const { swi_map(0x04) },
            lateout("r0") _,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
}

/// The vblank interrupt handler [VBlank][crate::interrupt::VBlank] should be
/// used instead of calling this function directly.
pub(crate) fn wait_for_vblank() {
    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x05) },
            lateout("r0") _,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
}

#[must_use]
pub fn div(numerator: i32, denominator: i32) -> (i32, i32, i32) {
    let divide: i32;
    let modulo: i32;
    let abs_divide: i32;
    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x06) },
            in("r0") numerator,
            in("r1") denominator,
            lateout("r0") divide,
            lateout("r1") modulo,
            lateout("r3") abs_divide,
        );
    }
    (divide, modulo, abs_divide)
}

#[must_use]
pub fn sqrt(n: i32) -> i32 {
    let result: i32;
    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x08) },
            in("r0") n,
            lateout("r0") result,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
    result
}

#[must_use]
pub fn arc_tan(n: i16) -> i16 {
    let result: i16;
    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x09) },
            in("r0") n,
            lateout("r0") result,
            lateout("r1") _,
            lateout("r2") _,
            lateout("r3") _
        );
    }
    result
}

#[must_use]
pub fn arc_tan2(x: i16, y: i32) -> i16 {
    let result: i16;
    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x09) },
            in("r0") x,
            in("r1") y,
            lateout("r0") result,
            lateout("r2") _,
            lateout("r3") _
        );
    }
    result
}

/// `rotation` is in revolutions. It is hard to create the rotation, usually
/// you'll go in from a larger sized type.
#[must_use]
pub(crate) fn bg_affine_matrix(
    bg_center: Vector2D<Num<i32, 8>>,
    display_center: Vector2D<i16>,
    scale: Vector2D<Num<i16, 8>>,
    rotation: Num<u16, 16>,
) -> AffineMatrixBackground {
    #[repr(C, packed(4))]
    struct Input {
        bg_center_x: Num<i32, 8>,
        bg_center_y: Num<i32, 8>,
        display_center_x: i16,
        display_center_y: i16,
        scale_x: Num<i16, 8>,
        scale_y: Num<i16, 8>,
        rotation: Num<u16, 16>,
    }

    let input = Input {
        bg_center_x: bg_center.x,
        bg_center_y: bg_center.y,
        display_center_x: display_center.x,
        display_center_y: display_center.y,
        scale_x: scale.x,
        scale_y: scale.y,
        rotation,
    };

    let mut output = MaybeUninit::uninit();

    unsafe {
        asm!(
            "swi {SWI}",
            SWI = const { swi_map(0x0E) },
            in("r0") &input as *const Input,
            in("r1") output.as_mut_ptr(),
            in("r2") 1,

            clobber_abi("C")
        );
    }

    unsafe { output.assume_init() }
}

#[cfg(test)]
mod tests {
    use crate::display::affine::AffineMatrix;

    use super::*;

    #[test_case]
    fn affine_bg(_gba: &mut crate::Gba) {
        // expect the identity matrix
        let aff = bg_affine_matrix(
            (0, 0).into(),
            (0i16, 0i16).into(),
            (1i16, 1i16).into(),
            Default::default(),
        );

        let matrix = aff.to_affine_matrix();
        assert_eq!(matrix, AffineMatrix::identity());
    }
}
