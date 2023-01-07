use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn read8() -> u8_0;
    fn read16() -> u16_0;
    fn read32() -> u32_0;
    fn skip8(count: u32_0);
    fn file_tell_read() -> libc::c_int;
    fn file_tell_size() -> libc::c_int;
    fn FixSample(samp: *mut Sample);
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
pub unsafe extern "C" fn Load_WAV(
    mut samp: *mut Sample,
    mut verbose: bool_0,
    mut fix: bool_0,
) -> libc::c_int {
    let mut file_size: libc::c_uint = 0;
    let mut bit_depth = 8 as libc::c_int as libc::c_uint;
    let mut hasformat = 0 as libc::c_int as libc::c_uint;
    let mut hasdata = 0 as libc::c_int as libc::c_uint;
    let mut chunk_code: libc::c_uint = 0;
    let mut chunk_size: libc::c_uint = 0;
    let mut num_channels = 0 as libc::c_int as libc::c_uint;
    if verbose != 0 {
        printf(b"Loading WAV file...\n\0" as *const u8 as *const libc::c_char);
    }
    memset(
        samp as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Sample>() as libc::c_ulong,
    );
    file_size = file_tell_size() as libc::c_uint;
    read32();
    read32();
    read32();
    while !(file_tell_read() as libc::c_uint >= file_size) {
        chunk_code = read32();
        chunk_size = read32();
        match chunk_code {
            544501094 => {
                if read16() as libc::c_int != 1 as libc::c_int {
                    if verbose != 0 {
                        printf(b"Unsupported WAV format.\n\0" as *const u8 as *const libc::c_char);
                    }
                    return 0x11 as libc::c_int;
                }
                num_channels = read16() as libc::c_uint;
                (*samp).frequency = read32();
                read32();
                read16();
                bit_depth = read16() as libc::c_uint;
                if bit_depth != 8 as libc::c_int as libc::c_uint
                    && bit_depth != 16 as libc::c_int as libc::c_uint
                {
                    if verbose != 0 {
                        printf(b"Unsupported bit-depth.\n\0" as *const u8 as *const libc::c_char);
                    }
                    return 0x13 as libc::c_int;
                }
                if bit_depth == 16 as libc::c_int as libc::c_uint {
                    let ref mut fresh0 = (*samp).format;
                    *fresh0 = (*fresh0 as libc::c_int | 0x1 as libc::c_int) as u8_0;
                }
                if verbose != 0 {
                    printf(
                        b"Sample Rate...%i\n\0" as *const u8 as *const libc::c_char,
                        (*samp).frequency,
                    );
                    printf(
                        b"Bit Depth.....%i-bit\n\0" as *const u8 as *const libc::c_char,
                        bit_depth,
                    );
                }
                if chunk_size.wrapping_sub(0x10 as libc::c_int as libc::c_uint)
                    > 0 as libc::c_int as libc::c_uint
                {
                    skip8(chunk_size.wrapping_sub(0x10 as libc::c_int as libc::c_uint));
                }
                hasformat = 1 as libc::c_int as libc::c_uint;
            }
            1635017060 => {
                let mut t: libc::c_int = 0;
                let mut c: libc::c_int = 0;
                let mut dat: libc::c_int = 0;
                if hasformat == 0 {
                    return 0x1 as libc::c_int;
                }
                if verbose != 0 {
                    printf(b"Loading Sample Data...\n\0" as *const u8 as *const libc::c_char);
                }
                let mut br =
                    file_size.wrapping_sub(file_tell_read() as libc::c_uint) as libc::c_int;
                chunk_size = if chunk_size > br as libc::c_uint {
                    br as libc::c_uint
                } else {
                    chunk_size
                };
                (*samp).sample_length = chunk_size
                    .wrapping_div(bit_depth.wrapping_div(8 as libc::c_int as libc::c_uint))
                    .wrapping_div(num_channels);
                let ref mut fresh1 = (*samp).data;
                *fresh1 = malloc(chunk_size as libc::c_ulong);
                t = 0 as libc::c_int;
                while (t as libc::c_uint) < (*samp).sample_length {
                    dat = 0 as libc::c_int;
                    c = 0 as libc::c_int;
                    while (c as libc::c_uint) < num_channels {
                        dat += if bit_depth == 8 as libc::c_int as libc::c_uint {
                            read8() as libc::c_int - 128 as libc::c_int
                        } else {
                            read16() as libc::c_short as libc::c_int
                        };
                        c += 1;
                    }
                    dat = (dat as libc::c_uint).wrapping_div(num_channels) as libc::c_int
                        as libc::c_int;
                    if bit_depth == 8 as libc::c_int as libc::c_uint {
                        *((*samp).data as *mut u8_0).offset(t as isize) =
                            (dat + 128 as libc::c_int) as u8_0;
                    } else {
                        *((*samp).data as *mut u16_0).offset(t as isize) =
                            (dat.wrapping_add(32768) as libc::c_int) as u16_0;
                    }
                    t += 1;
                }
                hasdata = 1 as libc::c_int as libc::c_uint;
            }
            1819307379 => {
                let mut pos: libc::c_int = 0;
                skip8(
                    (4 as libc::c_int
                        + 4 as libc::c_int
                        + 4 as libc::c_int
                        + 4 as libc::c_int
                        + 4 as libc::c_int
                        + 4 as libc::c_int
                        + 4 as libc::c_int) as u32_0,
                );
                let mut num_sample_loops = read32() as libc::c_int;
                read32();
                pos = 36 as libc::c_int;
                if num_sample_loops != 0 {
                    read32();
                    let mut loop_type = read32() as libc::c_int;
                    pos += 8 as libc::c_int;
                    if loop_type < 2 as libc::c_int {
                        (*samp).loop_type = (loop_type + 1 as libc::c_int) as u8_0;
                        (*samp).loop_start = read32();
                        (*samp).loop_end = read32();
                        if (*samp).loop_end > (*samp).sample_length {
                            (*samp).loop_end = (*samp).sample_length;
                        }
                        if (*samp).loop_start > (*samp).sample_length
                            || ((*samp).loop_end).wrapping_sub((*samp).loop_start)
                                < 16 as libc::c_int as libc::c_uint
                        {
                            (*samp).loop_type = 0 as libc::c_int as u8_0;
                            (*samp).loop_start = 0 as libc::c_int as u32_0;
                            (*samp).loop_end = 0 as libc::c_int as u32_0;
                        }
                        pos += 8 as libc::c_int;
                    }
                }
                skip8(chunk_size.wrapping_sub(pos as libc::c_uint));
            }
            _ => {
                skip8(chunk_size);
            }
        }
    }
    if hasformat != 0 && hasdata != 0 {
        if fix != 0 {
            FixSample(samp);
        }
        return 0 as libc::c_int;
    } else {
        return 0x1 as libc::c_int;
    };
}
