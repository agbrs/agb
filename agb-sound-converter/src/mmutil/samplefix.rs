use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn floor(_: libc::c_double) -> libc::c_double;
    static mut target_system: libc::c_int;
    fn adpcm_compress_sample(sample: *mut Sample);
    static mut ignore_sflags: libc::c_int;
}
pub type u16_0 = libc::c_ushort;
pub type u32_0 = libc::c_uint;
pub type u8_0 = libc::c_uchar;
pub type bool_0 = libc::c_uchar;
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
#[no_mangle]
pub unsafe extern "C" fn Sample_PadStart(mut samp: *mut Sample, mut count: u32_0) {
    let mut newdata8 = 0 as *mut u8_0;
    let mut newdata16 = 0 as *mut u16_0;
    let mut x: u32_0 = 0;
    if count == 0 as libc::c_int as libc::c_uint {
        return;
    }
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        newdata16 = malloc(
            ((*samp).sample_length)
                .wrapping_add(count)
                .wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong,
        ) as *mut u16_0;
        x = 0 as libc::c_int as u32_0;
        while x < count {
            *newdata16.offset(x as isize) = 32768 as libc::c_int as u16_0;
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata16.offset(count.wrapping_add(x) as isize) =
                *((*samp).data as *mut u16_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh0 = (*samp).data;
        *fresh0 = newdata16 as *mut libc::c_void;
    } else {
        newdata8 =
            malloc(((*samp).sample_length).wrapping_add(count) as libc::c_ulong) as *mut u8_0;
        x = 0 as libc::c_int as u32_0;
        while x < count {
            *newdata8.offset(x as isize) = 128 as libc::c_int as u8_0;
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata8.offset(count.wrapping_add(x) as isize) =
                *((*samp).data as *mut u8_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh1 = (*samp).data;
        *fresh1 = newdata8 as *mut libc::c_void;
    }
    let ref mut fresh2 = (*samp).loop_start;
    *fresh2 = (*fresh2 as libc::c_uint).wrapping_add(count) as u32_0 as u32_0;
    let ref mut fresh3 = (*samp).loop_end;
    *fresh3 = (*fresh3 as libc::c_uint).wrapping_add(count) as u32_0 as u32_0;
    let ref mut fresh4 = (*samp).sample_length;
    *fresh4 = (*fresh4 as libc::c_uint).wrapping_add(count) as u32_0 as u32_0;
}
#[no_mangle]
pub unsafe extern "C" fn Sample_PadEnd(mut samp: *mut Sample, mut count: u32_0) {
    let mut newdata8 = 0 as *mut u8_0;
    let mut newdata16 = 0 as *mut u16_0;
    let mut x: u32_0 = 0;
    if count == 0 as libc::c_int as libc::c_uint {
        return;
    }
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        newdata16 = malloc(
            ((*samp).sample_length)
                .wrapping_add(count)
                .wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong,
        ) as *mut u16_0;
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata16.offset(x as isize) = *((*samp).data as *mut u16_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < count {
            *newdata16.offset(((*samp).sample_length).wrapping_add(x) as isize) =
                32768 as libc::c_int as u16_0;
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh5 = (*samp).data;
        *fresh5 = newdata16 as *mut libc::c_void;
    } else {
        newdata8 =
            malloc(((*samp).sample_length).wrapping_add(count) as libc::c_ulong) as *mut u8_0;
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata8.offset(x as isize) = *((*samp).data as *mut u8_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < count {
            *newdata8.offset(((*samp).sample_length).wrapping_add(x) as isize) =
                128 as libc::c_int as u8_0;
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh6 = (*samp).data;
        *fresh6 = newdata8 as *mut libc::c_void;
    }
    let ref mut fresh7 = (*samp).loop_end;
    *fresh7 = (*fresh7 as libc::c_uint).wrapping_add(count) as u32_0 as u32_0;
    let ref mut fresh8 = (*samp).sample_length;
    *fresh8 = (*fresh8 as libc::c_uint).wrapping_add(count) as u32_0 as u32_0;
}
#[no_mangle]
pub unsafe extern "C" fn Unroll_Sample_Loop(mut samp: *mut Sample, mut count: u32_0) {
    let mut newdata8 = 0 as *mut u8_0;
    let mut newdata16 = 0 as *mut u16_0;
    let mut newlen: u32_0 = 0;
    let mut looplen: u32_0 = 0;
    let mut x: u32_0 = 0;
    looplen = ((*samp).loop_end).wrapping_sub((*samp).loop_start);
    newlen = ((*samp).sample_length).wrapping_add(looplen.wrapping_mul(count));
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        newdata16 = malloc(newlen.wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong)
            as *mut u16_0;
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata16.offset(x as isize) = *((*samp).data as *mut u16_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < looplen.wrapping_mul(count) {
            *newdata16.offset(((*samp).sample_length).wrapping_add(x) as isize) = *((*samp).data
                as *mut u16_0)
                .offset(((*samp).loop_start).wrapping_add(x.wrapping_rem(looplen)) as isize);
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh9 = (*samp).data;
        *fresh9 = newdata16 as *mut libc::c_void;
    } else {
        newdata8 = malloc(newlen as libc::c_ulong) as *mut u8_0;
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata8.offset(x as isize) = *((*samp).data as *mut u8_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < looplen.wrapping_mul(count) {
            *newdata8.offset(((*samp).sample_length).wrapping_add(x) as isize) = *((*samp).data
                as *mut u8_0)
                .offset(((*samp).loop_start).wrapping_add(x.wrapping_rem(looplen)) as isize);
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh10 = (*samp).data;
        *fresh10 = newdata8 as *mut libc::c_void;
    }
    let ref mut fresh11 = (*samp).loop_end;
    *fresh11 =
        (*fresh11 as libc::c_uint).wrapping_add(looplen.wrapping_mul(count)) as u32_0 as u32_0;
    let ref mut fresh12 = (*samp).sample_length;
    *fresh12 =
        (*fresh12 as libc::c_uint).wrapping_add(looplen.wrapping_mul(count)) as u32_0 as u32_0;
}
#[no_mangle]
pub unsafe extern "C" fn Unroll_BIDI_Sample(mut samp: *mut Sample) {
    let mut newdata8 = 0 as *mut u8_0;
    let mut newdata16 = 0 as *mut u16_0;
    let mut newlen: u32_0 = 0;
    let mut looplen: u32_0 = 0;
    let mut x: u32_0 = 0;
    looplen = ((*samp).loop_end).wrapping_sub((*samp).loop_start);
    newlen = ((*samp).sample_length).wrapping_add(looplen);
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        newdata16 = malloc(newlen.wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong)
            as *mut u16_0;
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata16.offset(x as isize) = *((*samp).data as *mut u16_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < looplen {
            *newdata16.offset(x.wrapping_add((*samp).sample_length) as isize) =
                *((*samp).data as *mut u16_0).offset(
                    ((*samp).loop_end)
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        .wrapping_sub(x) as isize,
                );
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh13 = (*samp).data;
        *fresh13 = newdata16 as *mut libc::c_void;
    } else {
        newdata8 = malloc(newlen as libc::c_ulong) as *mut u8_0;
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            *newdata8.offset(x as isize) = *((*samp).data as *mut u8_0).offset(x as isize);
            x = x.wrapping_add(1);
        }
        x = 0 as libc::c_int as u32_0;
        while x < looplen {
            *newdata8.offset(x.wrapping_add((*samp).sample_length) as isize) =
                *((*samp).data as *mut u8_0).offset(
                    ((*samp).loop_end)
                        .wrapping_sub(1 as libc::c_int as libc::c_uint)
                        .wrapping_sub(x) as isize,
                );
            x = x.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh14 = (*samp).data;
        *fresh14 = newdata8 as *mut libc::c_void;
    }
    (*samp).loop_type = 1 as libc::c_int as u8_0;
    let ref mut fresh15 = (*samp).sample_length;
    *fresh15 = (*fresh15 as libc::c_uint).wrapping_add(looplen) as u32_0 as u32_0;
    let ref mut fresh16 = (*samp).loop_end;
    *fresh16 = (*fresh16 as libc::c_uint).wrapping_add(looplen) as u32_0 as u32_0;
}
#[no_mangle]
pub unsafe extern "C" fn Resample(mut samp: *mut Sample, mut newsize: u32_0) {
    let mut dst8 = 0 as *mut u8_0;
    let mut dst16 = 0 as *mut u16_0;
    let mut src8 = (*samp).data as *mut u8_0;
    let mut src16 = (*samp).data as *mut u16_0;
    let mut oldlength = (*samp).sample_length as libc::c_int;
    let mut lpoint = (*samp).loop_start as libc::c_int;
    let mut i: libc::c_int = 0;
    let mut bit16 = ((*samp).format as libc::c_int & 0x1 as libc::c_int) as bool_0;
    let mut sign_diff: libc::c_double = 0.;
    if bit16 != 0 {
        dst16 = malloc(newsize.wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong)
            as *mut u16_0;
        sign_diff = 32768.0f64;
    } else {
        dst8 = malloc(newsize as libc::c_ulong) as *mut u8_0;
        sign_diff = 128.0f64;
    }
    let mut tscale = oldlength as libc::c_double / newsize as libc::c_double;
    let mut posf: libc::c_double = 0.;
    i = 0 as libc::c_int;
    while (i as libc::c_uint) < newsize {
        posf = i as libc::c_double * tscale;
        let mut posi = floor(posf) as libc::c_int;
        let mut mu = posf - posi as libc::c_double;
        let mut s0: libc::c_double = 0.;
        let mut s1: libc::c_double = 0.;
        let mut s2: libc::c_double = 0.;
        let mut s3: libc::c_double = 0.;
        let mut mu2: libc::c_double = 0.;
        let mut a0: libc::c_double = 0.;
        let mut a1: libc::c_double = 0.;
        let mut a2: libc::c_double = 0.;
        let mut a3: libc::c_double = 0.;
        let mut res: libc::c_double = 0.;
        if bit16 != 0 {
            s0 = if (posi - 1 as libc::c_int) < 0 as libc::c_int {
                0 as libc::c_int as libc::c_double
            } else {
                *src16.offset((posi - 1 as libc::c_int) as isize) as libc::c_double
            };
            s1 = *src16.offset(posi as isize) as libc::c_double;
            s2 = if posi + 1 as libc::c_int >= oldlength {
                if (*samp).loop_type as libc::c_int != 0 {
                    *src16.offset((lpoint + (posi + 1 as libc::c_int - oldlength)) as isize)
                        as libc::c_double
                } else {
                    0 as libc::c_int as libc::c_double
                }
            } else {
                *src16.offset((posi + 1 as libc::c_int) as isize) as libc::c_double
            };
            s3 = if posi + 1 as libc::c_int >= oldlength {
                if (*samp).loop_type as libc::c_int != 0 {
                    *src16.offset((lpoint + (posi + 2 as libc::c_int - oldlength)) as isize)
                        as libc::c_double
                } else {
                    0 as libc::c_int as libc::c_double
                }
            } else {
                *src16.offset((posi + 2 as libc::c_int) as isize) as libc::c_double
            };
        } else {
            s0 = if (posi - 1 as libc::c_int) < 0 as libc::c_int {
                0 as libc::c_int as libc::c_double
            } else {
                *src8.offset((posi - 1 as libc::c_int) as isize) as libc::c_double
            };
            s1 = *src8.offset(posi as isize) as libc::c_double;
            s2 = if posi + 1 as libc::c_int >= oldlength {
                if (*samp).loop_type as libc::c_int != 0 {
                    *src8.offset((lpoint + (posi + 1 as libc::c_int - oldlength)) as isize)
                        as libc::c_double
                } else {
                    0 as libc::c_int as libc::c_double
                }
            } else {
                *src8.offset((posi + 1 as libc::c_int) as isize) as libc::c_double
            };
            s3 = if posi + 1 as libc::c_int >= oldlength {
                if (*samp).loop_type as libc::c_int != 0 {
                    *src8.offset((lpoint + (posi + 2 as libc::c_int - oldlength)) as isize)
                        as libc::c_double
                } else {
                    0 as libc::c_int as libc::c_double
                }
            } else {
                *src8.offset((posi + 2 as libc::c_int) as isize) as libc::c_double
            };
        }
        s0 -= sign_diff;
        s1 -= sign_diff;
        s2 -= sign_diff;
        s3 -= sign_diff;
        mu2 = mu * mu;
        a0 = s3 - s2 - s0 + s1;
        a1 = s0 - s1 - a0;
        a2 = s2 - s0;
        a3 = s1;
        res = a0 * mu * mu2 + a1 * mu2 + a2 * mu + a3;
        let mut resi = floor(res + 0.5f64) as libc::c_int;
        if bit16 != 0 {
            if resi < -(32768 as libc::c_int) {
                resi = -(32768 as libc::c_int);
            }
            if resi > 32767 as libc::c_int {
                resi = 32767 as libc::c_int;
            }
            *dst16.offset(i as isize) = (resi + 32768 as libc::c_int) as u16_0;
        } else {
            if resi < -(128 as libc::c_int) {
                resi = -(128 as libc::c_int);
            }
            if resi > 127 as libc::c_int {
                resi = 127 as libc::c_int;
            }
            *dst8.offset(i as isize) = (resi + 128 as libc::c_int) as u8_0;
        }
        i += 1;
    }
    free((*samp).data);
    if bit16 != 0 {
        let ref mut fresh17 = (*samp).data;
        *fresh17 = dst16 as *mut libc::c_void;
    } else {
        let ref mut fresh18 = (*samp).data;
        *fresh18 = dst8 as *mut libc::c_void;
    }
    (*samp).sample_length = newsize;
    (*samp).loop_end = newsize;
    (*samp).loop_start = (((*samp).loop_start as libc::c_double * newsize as libc::c_double
        + oldlength as libc::c_double / 2 as libc::c_int as libc::c_double)
        / oldlength as libc::c_double) as libc::c_int as u32_0;
    (*samp).frequency = (((*samp).frequency as libc::c_double * newsize as libc::c_double
        + oldlength as libc::c_double / 2 as libc::c_int as libc::c_double)
        / oldlength as libc::c_double) as libc::c_int as u32_0;
}
#[no_mangle]
pub unsafe extern "C" fn Sample_8bit(mut samp: *mut Sample) {
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        let mut newdata = 0 as *mut u8_0;
        let mut t: u32_0 = 0;
        newdata = malloc((*samp).sample_length as libc::c_ulong) as *mut u8_0;
        t = 0 as libc::c_int as u32_0;
        while t < (*samp).sample_length {
            *newdata.offset(t as isize) = (*((*samp).data as *mut u16_0).offset(t as isize)
                as libc::c_int
                / 256 as libc::c_int) as u8_0;
            t = t.wrapping_add(1);
        }
        free((*samp).data);
        let ref mut fresh19 = (*samp).data;
        *fresh19 = newdata as *mut libc::c_void;
        let ref mut fresh20 = (*samp).format;
        *fresh20 = (*fresh20 as libc::c_int & !(0x1 as libc::c_int)) as u8_0;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Sample_Sign(mut samp: *mut Sample) {
    let mut x: u32_0 = 0;
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            let mut a = *((*samp).data as *mut u16_0).offset(x as isize) as libc::c_int
                - 32768 as libc::c_int;
            if a < -(32767 as libc::c_int) {
                a = -(32767 as libc::c_int);
            }
            *((*samp).data as *mut u16_0).offset(x as isize) = a as u16_0;
            x = x.wrapping_add(1);
        }
    } else {
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            let mut a_0 =
                *((*samp).data as *mut u8_0).offset(x as isize) as libc::c_int - 128 as libc::c_int;
            if a_0 == -(128 as libc::c_int) {
                a_0 = -(127 as libc::c_int);
            }
            *((*samp).data as *mut u8_0).offset(x as isize) = a_0 as u8_0;
            x = x.wrapping_add(1);
        }
    }
    let ref mut fresh21 = (*samp).format;
    *fresh21 = (*fresh21 as libc::c_int | 0x2 as libc::c_int) as u8_0;
}
#[no_mangle]
pub unsafe extern "C" fn FixSample_GBA(mut samp: *mut Sample) {
    Sample_8bit(samp);
    if (*samp).loop_type as libc::c_int != 0 as libc::c_int {
        (*samp).sample_length = (*samp).loop_end;
    }
    if (*samp).loop_type as libc::c_int == 2 as libc::c_int {
        Unroll_BIDI_Sample(samp);
    }
    if (*samp).loop_type != 0 {
        if ((*samp).loop_end).wrapping_sub((*samp).loop_start) < 512 as libc::c_int as libc::c_uint
        {
            Unroll_Sample_Loop(
                samp,
                (512 as libc::c_int as libc::c_uint)
                    .wrapping_div(((*samp).loop_end).wrapping_sub((*samp).loop_start))
                    .wrapping_add(1 as libc::c_int as libc::c_uint),
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn strcmpshit(
    mut str1: *mut libc::c_char,
    mut str2: *mut libc::c_char,
) -> libc::c_int {
    let mut x = 0 as libc::c_int;
    let mut f = 0 as libc::c_int;
    while *str1.offset(x as isize) as libc::c_int != 0 as libc::c_int {
        if *str1.offset(x as isize) as libc::c_int == *str2.offset(f as isize) as libc::c_int {
            f += 1;
        } else {
            f = 0 as libc::c_int;
        }
        if *str2.offset(f as isize) as libc::c_int == 0 as libc::c_int {
            return 1 as libc::c_int;
        }
        x += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn FixSample_NDS(mut samp: *mut Sample) {
    if (*samp).sample_length == 0 as libc::c_int as libc::c_uint {
        let ref mut fresh22 = (*samp).loop_start;
        *fresh22 = 0 as libc::c_int as u32_0;
        (*samp).loop_end = *fresh22;
        return;
    }
    if (*samp).loop_type as libc::c_int != 0 as libc::c_int {
        (*samp).sample_length = (*samp).loop_end;
    }
    if (*samp).loop_type as libc::c_int == 2 as libc::c_int {
        Unroll_BIDI_Sample(samp);
    }
    if (*samp).loop_type != 0 {
        if ignore_sflags == 0 {
            if strcmpshit(
                ((*samp).name).as_mut_ptr(),
                b"%o\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            ) > 0 as libc::c_int
            {
                Unroll_Sample_Loop(samp, 1 as libc::c_int as u32_0);
                let ref mut fresh23 = (*samp).loop_start;
                *fresh23 = (*fresh23 as libc::c_uint).wrapping_add(
                    ((*samp).loop_end)
                        .wrapping_sub((*samp).loop_start)
                        .wrapping_div(2 as libc::c_int as libc::c_uint),
                ) as u32_0 as u32_0;
            }
        }
    }
    if ignore_sflags == 0 {
        if strcmpshit(
            ((*samp).name).as_mut_ptr(),
            b"%c\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
        ) > 0 as libc::c_int
        {
            let ref mut fresh24 = (*samp).format;
            *fresh24 = (*fresh24 as libc::c_int | 0x4 as libc::c_int) as u8_0;
        }
    }
    if (*samp).loop_type != 0 {
        let mut looplen = ((*samp).loop_end).wrapping_sub((*samp).loop_start) as libc::c_int;
        if (*samp).format as libc::c_int & 0x4 as libc::c_int == 0 {
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                if looplen & 1 as libc::c_int != 0 {
                    let mut addition =
                        ((*samp).loop_end).wrapping_sub((*samp).loop_start) as libc::c_int;
                    if addition > 1024 as libc::c_int {
                        Resample(
                            samp,
                            ((*samp).sample_length).wrapping_add(1 as libc::c_int as libc::c_uint),
                        );
                    } else {
                        Unroll_Sample_Loop(samp, 1 as libc::c_int as u32_0);
                    }
                }
            } else if looplen & 3 as libc::c_int != 0 {
                let mut count: libc::c_int = 0;
                let mut addition_0: libc::c_int = 0;
                count = looplen & 3 as libc::c_int;
                match count {
                    0 => {
                        count = 0 as libc::c_int;
                    }
                    1 => {
                        count = 3 as libc::c_int;
                    }
                    2 => {
                        count = 1 as libc::c_int;
                    }
                    3 => {
                        count = 3 as libc::c_int;
                    }
                    _ => {}
                }
                addition_0 = looplen * count;
                if addition_0 > 1024 as libc::c_int {
                    Resample(
                        samp,
                        ((*samp).sample_length).wrapping_add(
                            (4 as libc::c_int - (looplen & 3 as libc::c_int)) as libc::c_uint,
                        ),
                    );
                } else {
                    Unroll_Sample_Loop(samp, count as u32_0);
                }
            }
        } else {
            let mut a = looplen;
            let mut count_0 = 0 as libc::c_int;
            let mut addition_1: libc::c_int = 0;
            while looplen & 7 as libc::c_int != 0 {
                count_0 += 1;
                looplen += a;
            }
            addition_1 = looplen * count_0;
            if addition_1 > 1024 as libc::c_int {
                Resample(
                    samp,
                    ((*samp).sample_length).wrapping_add(
                        (4 as libc::c_int - (looplen & 7 as libc::c_int)) as libc::c_uint,
                    ),
                );
            } else {
                Unroll_Sample_Loop(samp, count_0 as u32_0);
            }
        }
    }
    if (*samp).loop_type != 0 {
        let mut padsize: libc::c_int = 0;
        if (*samp).format as libc::c_int & 0x4 as libc::c_int == 0 {
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                padsize = ((2 as libc::c_int as libc::c_uint)
                    .wrapping_sub((*samp).loop_start & 1 as libc::c_int as libc::c_uint)
                    & 1 as libc::c_int as libc::c_uint) as libc::c_int;
            } else {
                padsize = ((4 as libc::c_int as libc::c_uint)
                    .wrapping_sub((*samp).loop_start & 3 as libc::c_int as libc::c_uint)
                    & 3 as libc::c_int as libc::c_uint) as libc::c_int;
            }
        } else {
            padsize = ((8 as libc::c_int as libc::c_uint)
                .wrapping_sub((*samp).loop_start & 7 as libc::c_int as libc::c_uint)
                & 7 as libc::c_int as libc::c_uint) as libc::c_int;
        }
        Sample_PadStart(samp, padsize as u32_0);
    }
    if (*samp).format as libc::c_int & 0x4 as libc::c_int == 0 {
        if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
            if (*samp).sample_length & 1 as libc::c_int as libc::c_uint != 0 {
                Sample_PadEnd(
                    samp,
                    (2 as libc::c_int as libc::c_uint)
                        .wrapping_sub((*samp).sample_length & 1 as libc::c_int as libc::c_uint),
                );
            }
        } else if (*samp).sample_length & 3 as libc::c_int as libc::c_uint != 0 {
            Sample_PadEnd(
                samp,
                (4 as libc::c_int as libc::c_uint)
                    .wrapping_sub((*samp).sample_length & 3 as libc::c_int as libc::c_uint),
            );
        }
    } else if (*samp).sample_length & 7 as libc::c_int as libc::c_uint != 0 {
        Sample_PadEnd(
            samp,
            (8 as libc::c_int as libc::c_uint)
                .wrapping_sub((*samp).sample_length & 7 as libc::c_int as libc::c_uint),
        );
    }
    Sample_Sign(samp);
    if (*samp).format as libc::c_int & 0x4 as libc::c_int != 0 {
        adpcm_compress_sample(samp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn FixSample(mut samp: *mut Sample) {
    (*samp).loop_start = if (*samp).loop_start < 0 as libc::c_int as libc::c_uint {
        0 as libc::c_int as libc::c_uint
    } else if (*samp).loop_start > (*samp).sample_length {
        (*samp).sample_length
    } else {
        (*samp).loop_start
    };
    (*samp).loop_end = if (*samp).loop_end < 0 as libc::c_int as libc::c_uint {
        0 as libc::c_int as libc::c_uint
    } else if (*samp).loop_end > (*samp).sample_length {
        (*samp).sample_length
    } else {
        (*samp).loop_end
    };
    if target_system == 0 as libc::c_int {
        FixSample_GBA(samp);
    } else if target_system == 1 as libc::c_int {
        FixSample_NDS(samp);
    }
}
