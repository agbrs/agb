use agb_fixnum::Vector2D;
use core::arch::asm;
use core::mem::MaybeUninit;

use crate::display::object::AffineMatrixAttributes;
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
pub fn wait_for_vblank() {
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

#[repr(C)]
pub struct BgAffineSetData {
    pub matrix: AffineMatrixAttributes,
    pub position: Vector2D<Num<i32, 8>>,
}
impl Default for BgAffineSetData {
    fn default() -> Self {
        Self {
            matrix: AffineMatrixAttributes::default(),
            position: (0, 0).into(),
        }
    }
}

/// `rotation` is in revolutions.
#[must_use]
pub fn bg_affine_matrix(
    bg_center: Vector2D<Num<i32, 8>>,
    display_center: Vector2D<i16>,
    scale: Vector2D<Num<i16, 8>>,
    rotation: Num<u16, 8>,
) -> BgAffineSetData {
    #[repr(C, packed)]
    struct Input {
        bg_center: Vector2D<Num<i32, 8>>,
        display_center: Vector2D<i16>,
        scale: Vector2D<Num<i16, 8>>,
        rotation: Num<u16, 8>,
    }

    let input = Input {
        bg_center,
        display_center,
        scale,
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
        );
    }

    unsafe { output.assume_init() }
}

/// `rotation` is in revolutions.
#[must_use]
pub fn obj_affine_matrix(
    scale: Vector2D<Num<i16, 8>>,
    rotation: Num<u8, 8>,
) -> AffineMatrixAttributes {
    #[allow(dead_code)]
    #[repr(C, packed)]
    struct Input {
        scale: Vector2D<Num<i16, 8>>,
        rotation: u16,
    }

    let input = Input {
        scale,
        rotation: u16::from(rotation.to_raw()) << 8,
    };

    let mut output = MaybeUninit::uninit();

    unsafe {
        asm!(
        "swi {SWI}",
        SWI = const { swi_map(0x0F) },
        in("r0") &input as *const Input,
        in("r1") output.as_mut_ptr(),
        in("r2") 1,
        in("r3") 2,
        );
    }

    unsafe { output.assume_init() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn affine(_gba: &mut crate::Gba) {
        // expect identity matrix
        let one: Num<i16, 8> = 1.into();

        let aff = obj_affine_matrix((one, one).into(), Num::default());
        assert_eq!(aff.p_a, one);
        assert_eq!(aff.p_d, one);
    }
}
