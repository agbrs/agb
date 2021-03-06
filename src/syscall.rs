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

#[allow(non_snake_case)]
pub fn wait_for_VBlank() {
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
