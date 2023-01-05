use ::libc;
extern "C" {
    fn strlen(_: *const libc::c_char) -> libc::c_ulong;
    fn __ctype_tolower_loc() -> *mut *const __int32_t;
}
pub type __int32_t = libc::c_int;
pub type u16_0 = libc::c_ushort;
pub type u32_0 = libc::c_uint;
pub type u8_0 = libc::c_uchar;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tSample {
    pub parapointer: u32_0,
    pub global_volume: u8_0,
    pub default_volume: u8_0,
    pub default_panning: u8_0,
    pub sample_length: u32_0,
    pub loop_start: u32_0,
    pub loop_end: u32_0,
    pub loop_type: u8_0,
    pub frequency: u32_0,
    pub data: *mut libc::c_void,
    pub vibtype: u8_0,
    pub vibdepth: u8_0,
    pub vibspeed: u8_0,
    pub vibrate: u8_0,
    pub msl_index: u16_0,
    pub rsamp_index: u8_0,
    pub format: u8_0,
    pub datapointer: u32_0,
    pub it_compression: u8_0,
    pub name: [libc::c_char; 32],
    pub filename: [libc::c_char; 12],
}
pub type Sample = tSample;
#[inline]
unsafe extern "C" fn tolower(mut __c: libc::c_int) -> libc::c_int {
    return if __c >= -(128 as libc::c_int) && __c < 256 as libc::c_int {
        *(*__ctype_tolower_loc()).offset(__c as isize)
    } else {
        __c
    };
}
#[no_mangle]
pub unsafe extern "C" fn readbits(
    mut buffer: *mut u8_0,
    mut pos: libc::c_uint,
    mut size: libc::c_uint,
) -> u32_0 {
    let mut result = 0 as libc::c_int as u32_0;
    let mut i: u32_0 = 0;
    i = 0 as libc::c_int as u32_0;
    while i < size {
        let mut byte_pos: u32_0 = 0;
        let mut bit_pos: u32_0 = 0;
        byte_pos = pos.wrapping_add(i) >> 3 as libc::c_int;
        bit_pos = pos.wrapping_add(i) & 7 as libc::c_int as libc::c_uint;
        result |= ((*buffer.offset(byte_pos as isize) as libc::c_int >> bit_pos & 1 as libc::c_int)
            << i) as libc::c_uint;
        i = i.wrapping_add(1);
    }
    return result;
}
#[no_mangle]
pub unsafe extern "C" fn get_ext(mut filename: *mut libc::c_char) -> libc::c_int {
    let mut strl = strlen(filename) as libc::c_int;
    let mut x: libc::c_int = 0;
    let mut a = 0 as libc::c_int as u32_0;
    if strl < 4 as libc::c_int {
        return 6 as libc::c_int;
    }
    x = 0 as libc::c_int;
    while x < 4 as libc::c_int {
        if !(*filename.offset((strl - x - 1 as libc::c_int) as isize) as libc::c_int != '.' as i32)
        {
            break;
        }
        a |= (({
            let mut __res: libc::c_int = 0;
            if ::std::mem::size_of::<libc::c_char>() as libc::c_ulong
                > 1 as libc::c_int as libc::c_ulong
            {
                if 0 != 0 {
                    let mut __c =
                        *filename.offset((strl - x - 1 as libc::c_int) as isize) as libc::c_int;
                    __res = (if __c < -(128 as libc::c_int) || __c > 255 as libc::c_int {
                        __c
                    } else {
                        *(*__ctype_tolower_loc()).offset(__c as isize)
                    });
                } else {
                    __res = tolower(
                        *filename.offset((strl - x - 1 as libc::c_int) as isize) as libc::c_int
                    );
                }
            } else {
                __res =
                    *(*__ctype_tolower_loc())
                        .offset(*filename.offset((strl - x - 1 as libc::c_int) as isize)
                            as libc::c_int as isize);
            }
            __res
        }) << x * 8 as libc::c_int) as libc::c_uint;
        x += 1;
    }
    match a {
        7171940 => return 0 as libc::c_int,
        7549805 => return 1 as libc::c_int,
        7633012 => return 5 as libc::c_int,
        7823734 => return 4 as libc::c_int,
        7172972 => return 8 as libc::c_int,
        30829 => return 2 as libc::c_int,
        26996 => return 3 as libc::c_int,
        104 => return 7 as libc::c_int,
        _ => {}
    }
    return 6 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn calc_samplen_ex2(mut s: *mut Sample) -> u32_0 {
    if (*s).loop_type as libc::c_int == 0 as libc::c_int {
        return (*s).sample_length;
    } else {
        return (*s).loop_end;
    };
}
#[no_mangle]
pub unsafe extern "C" fn calc_samplooplen(mut s: *mut Sample) -> u32_0 {
    let mut a: u32_0 = 0;
    if (*s).loop_type as libc::c_int == 1 as libc::c_int {
        a = ((*s).loop_end).wrapping_sub((*s).loop_start);
        return a;
    } else if (*s).loop_type as libc::c_int == 2 as libc::c_int {
        a = ((*s).loop_end)
            .wrapping_sub((*s).loop_start)
            .wrapping_mul(2 as libc::c_int as libc::c_uint);
        return a;
    } else {
        return 0xffffffff as libc::c_uint;
    };
}
#[no_mangle]
pub unsafe extern "C" fn calc_samplen(mut s: *mut Sample) -> u32_0 {
    if (*s).loop_type as libc::c_int == 1 as libc::c_int {
        return (*s).loop_end;
    } else if (*s).loop_type as libc::c_int == 2 as libc::c_int {
        return ((*s).loop_end)
            .wrapping_sub((*s).loop_start)
            .wrapping_add((*s).loop_end);
    } else {
        return (*s).sample_length;
    };
}
#[no_mangle]
pub unsafe extern "C" fn sample_dsformat(mut samp: *mut Sample) -> u8_0 {
    if (*samp).format as libc::c_int & 0x4 as libc::c_int != 0 {
        return 2 as libc::c_int as u8_0;
    } else if (*samp).format as libc::c_int & 0x2 as libc::c_int != 0 {
        if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
            return 1 as libc::c_int as u8_0;
        } else {
            return 0 as libc::c_int as u8_0;
        }
    } else if (*samp).format as libc::c_int & 0x1 as libc::c_int == 0 {
        return 3 as libc::c_int as u8_0;
    } else {
        return 3 as libc::c_int as u8_0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn sample_dsreptype(mut samp: *mut Sample) -> u8_0 {
    if (*samp).loop_type != 0 {
        return 1 as libc::c_int as u8_0;
    } else {
        return 2 as libc::c_int as u8_0;
    };
}
#[no_mangle]
pub unsafe extern "C" fn clamp_s8(mut value: libc::c_int) -> libc::c_int {
    if value < -(128 as libc::c_int) {
        value = -(128 as libc::c_int);
    }
    if value > 127 as libc::c_int {
        value = 127 as libc::c_int;
    }
    return value;
}
#[no_mangle]
pub unsafe extern "C" fn clamp_u8(mut value: libc::c_int) -> libc::c_int {
    if value < 0 as libc::c_int {
        value = 0 as libc::c_int;
    }
    if value > 255 as libc::c_int {
        value = 255 as libc::c_int;
    }
    return value;
}
