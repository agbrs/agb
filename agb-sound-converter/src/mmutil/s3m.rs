use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    static mut PANNING_SEP: libc::c_int;
    fn read8() -> u8_0;
    fn read16() -> u16_0;
    fn file_seek_read(offset: libc::c_int, mode: libc::c_int) -> libc::c_int;
    fn read32() -> u32_0;
    fn skip8(count: u32_0);
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn clamp_u8(value: libc::c_int) -> libc::c_int;
    fn FixSample(samp: *mut Sample);
}
pub type u16_0 = libc::c_ushort;
pub type u32_0 = libc::c_uint;
pub type u8_0 = libc::c_uchar;
pub type bool_0 = libc::c_uchar;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tInstrument_Envelope {
    pub loop_start: u8_0,
    pub loop_end: u8_0,
    pub sus_start: u8_0,
    pub sus_end: u8_0,
    pub node_count: u8_0,
    pub node_x: [u16_0; 25],
    pub node_y: [u8_0; 25],
    pub env_filter: bool_0,
    pub env_valid: bool_0,
    pub env_enabled: bool_0,
}
pub type Instrument_Envelope = tInstrument_Envelope;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tInstrument {
    pub parapointer: u32_0,
    pub global_volume: u8_0,
    pub setpan: u8_0,
    pub fadeout: u16_0,
    pub random_volume: u8_0,
    pub nna: u8_0,
    pub dct: u8_0,
    pub dca: u8_0,
    pub env_flags: u8_0,
    pub notemap: [u16_0; 120],
    pub name: [libc::c_char; 32],
    pub envelope_volume: Instrument_Envelope,
    pub envelope_pan: Instrument_Envelope,
    pub envelope_pitch: Instrument_Envelope,
}
pub type Instrument = tInstrument;
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tPatternEntry {
    pub note: u8_0,
    pub inst: u8_0,
    pub vol: u8_0,
    pub fx: u8_0,
    pub param: u8_0,
}
pub type PatternEntry = tPatternEntry;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tPattern {
    pub parapointer: u32_0,
    pub nrows: u16_0,
    pub clength: libc::c_int,
    pub data: [PatternEntry; 8192],
    pub cmarks: [bool_0; 256],
}
pub type Pattern = tPattern;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct tMAS_Module {
    pub title: [libc::c_char; 32],
    pub order_count: u16_0,
    pub inst_count: u8_0,
    pub samp_count: u8_0,
    pub patt_count: u8_0,
    pub restart_pos: u8_0,
    pub stereo: bool_0,
    pub inst_mode: bool_0,
    pub freq_mode: u8_0,
    pub old_effects: bool_0,
    pub link_gxx: bool_0,
    pub xm_mode: bool_0,
    pub old_mode: bool_0,
    pub global_volume: u8_0,
    pub initial_speed: u8_0,
    pub initial_tempo: u8_0,
    pub channel_volume: [u8_0; 32],
    pub channel_panning: [u8_0; 32],
    pub orders: [u8_0; 256],
    pub instruments: *mut Instrument,
    pub samples: *mut Sample,
    pub patterns: *mut Pattern,
}
pub type MAS_Module = tMAS_Module;
#[no_mangle]
pub unsafe extern "C" fn Load_S3M_SampleData(mut samp: *mut Sample, mut ffi: u8_0) -> libc::c_int {
    let mut x: u32_0 = 0;
    let mut a: libc::c_int = 0;
    if (*samp).sample_length == 0 as libc::c_int as libc::c_uint {
        return 0 as libc::c_int;
    }
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        let ref mut fresh0 = (*samp).data;
        *fresh0 = malloc(
            ((*samp).sample_length).wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong,
        ) as *mut u16_0 as *mut libc::c_void;
    } else {
        let ref mut fresh1 = (*samp).data;
        *fresh1 = malloc((*samp).sample_length as libc::c_ulong) as *mut u8_0 as *mut libc::c_void;
    }
    if ffi as libc::c_int == 1 as libc::c_int {
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                a = read16() as libc::c_int;
                a += 32768 as libc::c_int;
                *((*samp).data as *mut u16_0).offset(x as isize) = a as u16_0;
            } else {
                a = read8() as libc::c_int;
                a += 128 as libc::c_int;
                *((*samp).data as *mut u8_0).offset(x as isize) = a as u8_0;
            }
            x = x.wrapping_add(1);
        }
    } else if ffi as libc::c_int == 2 as libc::c_int {
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                a = read16() as libc::c_int;
                *((*samp).data as *mut u16_0).offset(x as isize) = a as u16_0;
            } else {
                a = read8() as libc::c_int;
                *((*samp).data as *mut u8_0).offset(x as isize) = a as u8_0;
            }
            x = x.wrapping_add(1);
        }
    } else {
        return 0x6 as libc::c_int;
    }
    FixSample(samp);
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_S3M_Sample(
    mut samp: *mut Sample,
    mut verbose: bool_0,
) -> libc::c_int {
    let mut flags: u8_0 = 0;
    let mut x: u32_0 = 0;
    memset(
        samp as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Sample>() as libc::c_ulong,
    );
    (*samp).msl_index = 0xffff as libc::c_int as u16_0;
    if read8() as libc::c_int == 1 as libc::c_int {
        x = 0 as libc::c_int as u32_0;
        while x < 12 as libc::c_int as libc::c_uint {
            (*samp).filename[x as usize] = read8() as libc::c_char;
            x = x.wrapping_add(1);
        }
        (*samp).datapointer = ((read8() as libc::c_int * 65536 as libc::c_int
            + read16() as libc::c_int)
            * 16 as libc::c_int) as u32_0;
        (*samp).sample_length = read32();
        (*samp).loop_start = read32();
        (*samp).loop_end = read32();
        (*samp).default_volume = read8();
        (*samp).global_volume = 64 as libc::c_int as u8_0;
        read8();
        if read8() as libc::c_int != 0 as libc::c_int {
            return 0x6 as libc::c_int;
        }
        flags = read8();
        (*samp).loop_type = (if flags as libc::c_int & 1 as libc::c_int != 0 {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0;
        if flags as libc::c_int & 2 as libc::c_int != 0 {
            return 0x6 as libc::c_int;
        }
        (*samp).format = (if flags as libc::c_int & 4 as libc::c_int != 0 {
            0x1 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0;
        (*samp).frequency = read32();
        read32();
        skip8(8 as libc::c_int as u32_0);
        x = 0 as libc::c_int as u32_0;
        while x < 28 as libc::c_int as libc::c_uint {
            (*samp).name[x as usize] = read8() as libc::c_char;
            x = x.wrapping_add(1);
        }
        if read32() != 1397900115i32 as libc::c_uint {
            return 0x6 as libc::c_int;
        }
        if verbose != 0 {
            printf(
                b"%-5i   %-3s   %3i%%   %5ihz  %-28s \n\0" as *const u8 as *const libc::c_char,
                (*samp).sample_length,
                if (*samp).loop_type as libc::c_int != 0 {
                    b"Yes\0" as *const u8 as *const libc::c_char
                } else {
                    b"No\0" as *const u8 as *const libc::c_char
                },
                (*samp).default_volume as libc::c_int * 100 as libc::c_int / 64 as libc::c_int,
                (*samp).frequency,
                ((*samp).name).as_mut_ptr(),
            );
        }
    } else if verbose != 0 {
        printf(
            b"-----   ---   ----   -------  %-28s\n\0" as *const u8 as *const libc::c_char,
            ((*samp).name).as_mut_ptr(),
        );
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_S3M_Pattern(mut patt: *mut Pattern) -> libc::c_int {
    let mut clength: libc::c_int = 0;
    let mut row: libc::c_int = 0;
    let mut col: libc::c_int = 0;
    let mut what: u8_0 = 0;
    let mut z: libc::c_int = 0;
    clength = read16() as libc::c_int;
    memset(
        patt as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Pattern>() as libc::c_ulong,
    );
    (*patt).clength = clength;
    (*patt).nrows = 64 as libc::c_int as u16_0;
    row = 0 as libc::c_int;
    while row < 64 as libc::c_int * 32 as libc::c_int {
        (*patt).data[row as usize].note = 250 as libc::c_int as u8_0;
        (*patt).data[row as usize].vol = 255 as libc::c_int as u8_0;
        row += 1;
    }
    row = 0 as libc::c_int;
    while row < 64 as libc::c_int {
        loop {
            what = read8();
            if !(what as libc::c_int != 0 as libc::c_int) {
                break;
            }
            col = what as libc::c_int & 31 as libc::c_int;
            z = row * 32 as libc::c_int + col;
            if what as libc::c_int & 32 as libc::c_int != 0 {
                (*patt).data[z as usize].note = read8();
                if (*patt).data[z as usize].note as libc::c_int == 255 as libc::c_int {
                    (*patt).data[z as usize].note = 250 as libc::c_int as u8_0;
                } else if (*patt).data[z as usize].note as libc::c_int == 254 as libc::c_int {
                    (*patt).data[z as usize].note = 254 as libc::c_int as u8_0;
                } else {
                    (*patt).data[z as usize].note =
                        (((*patt).data[z as usize].note as libc::c_int & 15 as libc::c_int)
                            + ((*patt).data[z as usize].note as libc::c_int >> 4 as libc::c_int)
                                * 12 as libc::c_int
                            + 12 as libc::c_int) as u8_0;
                }
                (*patt).data[z as usize].inst = read8();
            }
            if what as libc::c_int & 64 as libc::c_int != 0 {
                (*patt).data[z as usize].vol = read8();
            }
            if what as libc::c_int & 128 as libc::c_int != 0 {
                (*patt).data[z as usize].fx = read8();
                (*patt).data[z as usize].param = read8();
                if (*patt).data[z as usize].fx as libc::c_int == 3 as libc::c_int {
                    (*patt).data[z as usize].param =
                        (((*patt).data[z as usize].param as libc::c_int & 0xf as libc::c_int)
                            + (*patt).data[z as usize].param as libc::c_int / 16 as libc::c_int
                                * 10 as libc::c_int) as u8_0;
                }
                if (*patt).data[z as usize].fx as libc::c_int == 'X' as i32 - 64 as libc::c_int {
                    let ref mut fresh2 = (*patt).data[z as usize].param;
                    *fresh2 = (*fresh2 as libc::c_int * 2 as libc::c_int) as u8_0;
                }
                if (*patt).data[z as usize].fx as libc::c_int == 'V' as i32 - 64 as libc::c_int {
                    let ref mut fresh3 = (*patt).data[z as usize].param;
                    *fresh3 = (*fresh3 as libc::c_int * 2 as libc::c_int) as u8_0;
                }
            }
            if (*patt).data[z as usize].fx as libc::c_int == 255 as libc::c_int {
                (*patt).data[z as usize].fx = 0 as libc::c_int as u8_0;
                (*patt).data[z as usize].param = 0 as libc::c_int as u8_0;
            }
        }
        row += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_S3M(mut mod_0: *mut MAS_Module, mut verbose: bool_0) -> libc::c_int {
    let mut s3m_flags: u16_0 = 0;
    let mut cwt: u16_0 = 0;
    let mut ffi: u16_0 = 0;
    let mut dp: u8_0 = 0;
    let mut stereo: bool_0 = 0;
    let mut a: u8_0 = 0;
    let mut chan_enabled: [bool_0; 32] = [0; 32];
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut parap_inst = 0 as *mut u16_0;
    let mut parap_patt = 0 as *mut u16_0;
    memset(
        mod_0 as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<MAS_Module>() as libc::c_ulong,
    );
    x = 0 as libc::c_int;
    while x < 28 as libc::c_int {
        (*mod_0).title[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    read8() as libc::c_int != 0x1a as libc::c_int;
    if read8() as libc::c_int != 16 as libc::c_int {
        return 0x1 as libc::c_int;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    if verbose != 0 {
        printf(
            b"Loading S3M, \"%s\"\n\0" as *const u8 as *const libc::c_char,
            ((*mod_0).title).as_mut_ptr(),
        );
    }
    skip8(2 as libc::c_int as u32_0);
    (*mod_0).order_count = read16() as u8_0 as u16_0;
    (*mod_0).inst_count = read16() as u8_0;
    (*mod_0).samp_count = (*mod_0).inst_count;
    (*mod_0).patt_count = read16() as u8_0;
    x = 0 as libc::c_int;
    while x < 32 as libc::c_int {
        (*mod_0).channel_volume[x as usize] = 64 as libc::c_int as u8_0;
        x += 1;
    }
    (*mod_0).freq_mode = 0 as libc::c_int as u8_0;
    (*mod_0).old_effects = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    (*mod_0).link_gxx = 0 as libc::c_int as bool_0;
    (*mod_0).restart_pos = 0 as libc::c_int as u8_0;
    (*mod_0).old_mode = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    s3m_flags = read16();
    cwt = read16();
    ffi = read16();
    if read32() != 1297236819i32 as libc::c_uint {
        return 0x1 as libc::c_int;
    }
    (*mod_0).global_volume = (read8() as libc::c_int * 2 as libc::c_int) as u8_0;
    (*mod_0).initial_speed = read8();
    (*mod_0).initial_tempo = read8();
    stereo = (read8() as libc::c_int >> 7 as libc::c_int) as bool_0;
    read8();
    dp = read8();
    skip8((8 as libc::c_int + 2 as libc::c_int) as u32_0);
    x = 0 as libc::c_int;
    while x < 32 as libc::c_int {
        let mut chn = read8();
        chan_enabled[x as usize] = (chn as libc::c_int >> 7 as libc::c_int) as bool_0;
        if stereo != 0 {
            if (chn as libc::c_int & 127 as libc::c_int) < 8 as libc::c_int {
                (*mod_0).channel_panning[x as usize] =
                    clamp_u8(128 as libc::c_int - PANNING_SEP / 2 as libc::c_int) as u8_0;
            } else {
                (*mod_0).channel_panning[x as usize] =
                    clamp_u8(128 as libc::c_int + PANNING_SEP / 2 as libc::c_int) as u8_0;
            }
        } else {
            (*mod_0).channel_panning[x as usize] = 128 as libc::c_int as u8_0;
        }
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).order_count as libc::c_int {
        (*mod_0).orders[x as usize] = read8();
        x += 1;
    }
    parap_inst = malloc(
        ((*mod_0).inst_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<u16_0>() as libc::c_ulong),
    ) as *mut u16_0;
    parap_patt = malloc(
        ((*mod_0).patt_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<u16_0>() as libc::c_ulong),
    ) as *mut u16_0;
    x = 0 as libc::c_int;
    while x < (*mod_0).inst_count as libc::c_int {
        *parap_inst.offset(x as isize) = read16();
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).patt_count as libc::c_int {
        *parap_patt.offset(x as isize) = read16();
        x += 1;
    }
    if dp as libc::c_int == 252 as libc::c_int {
        x = 0 as libc::c_int;
        while x < 32 as libc::c_int {
            a = read8();
            if a as libc::c_int & 32 as libc::c_int != 0 {
                (*mod_0).channel_panning[x as usize] = (if (a as libc::c_int & 15 as libc::c_int)
                    * 16 as libc::c_int
                    > 255 as libc::c_int
                {
                    255 as libc::c_int
                } else {
                    (a as libc::c_int & 15 as libc::c_int) * 16 as libc::c_int
                }) as u8_0;
            }
            x += 1;
        }
    } else {
        x = 0 as libc::c_int;
        while x < 32 as libc::c_int {
            if stereo != 0 {
                (*mod_0).channel_panning[x as usize] = (if x & 1 as libc::c_int != 0 {
                    clamp_u8(128 as libc::c_int - PANNING_SEP / 2 as libc::c_int)
                } else {
                    clamp_u8(128 as libc::c_int + PANNING_SEP / 2 as libc::c_int)
                }) as u8_0;
            } else {
                (*mod_0).channel_panning[x as usize] = 128 as libc::c_int as u8_0;
            }
            x += 1;
        }
    }
    let ref mut fresh4 = (*mod_0).instruments;
    *fresh4 = malloc(
        ((*mod_0).inst_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Instrument>() as libc::c_ulong),
    ) as *mut Instrument;
    let ref mut fresh5 = (*mod_0).samples;
    *fresh5 = malloc(
        ((*mod_0).samp_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Sample>() as libc::c_ulong),
    ) as *mut Sample;
    let ref mut fresh6 = (*mod_0).patterns;
    *fresh6 = malloc(
        ((*mod_0).patt_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Pattern>() as libc::c_ulong),
    ) as *mut Pattern;
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading Samples...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b" INDEX LENGTH  LOOP  VOLUME  MID-C   NAME\n\0" as *const u8 as *const libc::c_char,
        );
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).inst_count as libc::c_int {
        if verbose != 0 {
            printf(
                b" %-2i    \0" as *const u8 as *const libc::c_char,
                x + 1 as libc::c_int,
            );
        }
        memset(
            &mut *((*mod_0).instruments).offset(x as isize) as *mut Instrument as *mut libc::c_void,
            0 as libc::c_int,
            ::std::mem::size_of::<Instrument>() as libc::c_ulong,
        );
        (*((*mod_0).instruments).offset(x as isize)).global_volume = 128 as libc::c_int as u8_0;
        y = 0 as libc::c_int;
        while y < 120 as libc::c_int {
            (*((*mod_0).instruments).offset(x as isize)).notemap[y as usize] =
                (y | (x + 1 as libc::c_int) << 8 as libc::c_int) as u16_0;
            y += 1;
        }
        file_seek_read(
            *parap_inst.offset(x as isize) as libc::c_int * 16 as libc::c_int,
            0 as libc::c_int,
        );
        if Load_S3M_Sample(&mut *((*mod_0).samples).offset(x as isize), verbose) != 0 {
            printf(b"Error loading sample!\n\0" as *const u8 as *const libc::c_char);
            return 0x6 as libc::c_int;
        }
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading Patterns...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).patt_count as libc::c_int {
        if verbose != 0 {
            printf(
                b" * %2i%s\0" as *const u8 as *const libc::c_char,
                x + 1 as libc::c_int,
                if (x + 1 as libc::c_int) % 15 as libc::c_int != 0 {
                    b"\0" as *const u8 as *const libc::c_char
                } else {
                    b"\n\0" as *const u8 as *const libc::c_char
                },
            );
        }
        file_seek_read(
            *parap_patt.offset(x as isize) as libc::c_int * 16 as libc::c_int,
            0 as libc::c_int,
        );
        Load_S3M_Pattern(&mut *((*mod_0).patterns).offset(x as isize));
        x += 1;
    }
    if verbose != 0 {
        printf(b"\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading Sample Data...\n\0" as *const u8 as *const libc::c_char);
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).samp_count as libc::c_int {
        file_seek_read(
            (*((*mod_0).samples).offset(x as isize)).datapointer as libc::c_int,
            0 as libc::c_int,
        );
        Load_S3M_SampleData(&mut *((*mod_0).samples).offset(x as isize), ffi as u8_0);
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    return 0 as libc::c_int;
}
