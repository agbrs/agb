use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn skip8(count: u32_0);
    fn file_seek_read(offset: libc::c_int, mode: libc::c_int) -> libc::c_int;
    fn read16() -> u16_0;
    fn read8() -> u8_0;
    fn read32() -> u32_0;
    fn readbits(buffer: *mut u8_0, pos: libc::c_uint, size: libc::c_uint) -> u32_0;
    fn FixSample(samp: *mut Sample);
}
pub type u16_0 = libc::c_ushort;
pub type u32_0 = libc::c_uint;
pub type s16 = libc::c_short;
pub type u8_0 = libc::c_uchar;
pub type s8 = libc::c_schar;
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
pub unsafe extern "C" fn Load_IT_Envelope(
    mut env: *mut Instrument_Envelope,
    mut unsign: bool_0,
) -> bool_0 {
    let mut a: u8_0 = 0;
    let mut node_count: u8_0 = 0;
    let mut x: libc::c_int = 0;
    let mut env_loop = 0 as libc::c_int as bool_0;
    let mut env_sus = 0 as libc::c_int as bool_0;
    let mut env_enabled = 0 as libc::c_int as bool_0;
    let mut env_filter = 0 as libc::c_int as bool_0;
    memset(
        env as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Instrument_Envelope>() as libc::c_ulong,
    );
    a = read8();
    if a as libc::c_int & 1 as libc::c_int != 0 {
        env_enabled = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    }
    if a as libc::c_int & 2 as libc::c_int == 0 {
        (*env).loop_start = 255 as libc::c_int as u8_0;
        (*env).loop_end = 255 as libc::c_int as u8_0;
    } else {
        env_loop = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    }
    if a as libc::c_int & 4 as libc::c_int == 0 {
        (*env).sus_start = 255 as libc::c_int as u8_0;
        (*env).sus_end = 255 as libc::c_int as u8_0;
    } else {
        env_sus = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    }
    if a as libc::c_int & 128 as libc::c_int != 0 {
        unsign = 0 as libc::c_int as bool_0;
        env_filter = (0 as libc::c_int == 0) as libc::c_int as bool_0;
        (*env).env_filter = env_filter;
    }
    node_count = read8();
    if node_count as libc::c_int != 0 as libc::c_int {
        (*env).env_valid = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    }
    (*env).node_count = node_count;
    if env_loop != 0 {
        (*env).loop_start = read8();
        (*env).loop_end = read8();
    } else {
        skip8(2 as libc::c_int as u32_0);
    }
    if env_sus != 0 {
        (*env).sus_start = read8();
        (*env).sus_end = read8();
    } else {
        skip8(2 as libc::c_int as u32_0);
    }
    x = 0 as libc::c_int;
    while x < 25 as libc::c_int {
        (*env).node_y[x as usize] = read8();
        if unsign != 0 {
            let ref mut fresh0 = (*env).node_y[x as usize];
            *fresh0 = (*fresh0 as libc::c_int + 32 as libc::c_int) as u8_0;
        }
        (*env).node_x[x as usize] = read16();
        x += 1;
    }
    read8();
    (*env).env_enabled = env_enabled;
    return env_enabled;
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT_Instrument(
    mut inst: *mut Instrument,
    mut verbose: bool_0,
    mut index: libc::c_int,
) -> libc::c_int {
    let mut a: u16_0 = 0;
    let mut x: libc::c_int = 0;
    memset(
        inst as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Instrument>() as libc::c_ulong,
    );
    skip8(17 as libc::c_int as u32_0);
    (*inst).nna = read8();
    (*inst).dct = read8();
    (*inst).dca = read8();
    a = read16();
    if a as libc::c_int > 255 as libc::c_int {
        a = 255 as libc::c_int as u16_0;
    }
    (*inst).fadeout = a as u8_0 as u16_0;
    skip8(2 as libc::c_int as u32_0);
    (*inst).global_volume = read8();
    a = read8() as u16_0;
    a = (a as libc::c_int & 128 as libc::c_int
        | (if (a as libc::c_int & 127 as libc::c_int) * 2 as libc::c_int > 127 as libc::c_int {
            127 as libc::c_int
        } else {
            (a as libc::c_int & 127 as libc::c_int) * 2 as libc::c_int
        })) as u16_0;
    (*inst).setpan = (a as libc::c_int ^ 128 as libc::c_int) as u8_0;
    (*inst).random_volume = read8();
    skip8(5 as libc::c_int as u32_0);
    x = 0 as libc::c_int;
    while x < 26 as libc::c_int {
        (*inst).name[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    skip8(6 as libc::c_int as u32_0);
    x = 0 as libc::c_int;
    while x < 120 as libc::c_int {
        (*inst).notemap[x as usize] = read16();
        x += 1;
    }
    (*inst).env_flags = 0 as libc::c_int as u8_0;
    Load_IT_Envelope(&mut (*inst).envelope_volume, 0 as libc::c_int as bool_0);
    let ref mut fresh1 = (*inst).env_flags;
    *fresh1 = (*fresh1 as libc::c_int
        | if (*inst).envelope_volume.env_valid as libc::c_int != 0 {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0;
    let ref mut fresh2 = (*inst).env_flags;
    *fresh2 = (*fresh2 as libc::c_int
        | if (*inst).envelope_volume.env_enabled as libc::c_int != 0 {
            8 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0;
    Load_IT_Envelope(
        &mut (*inst).envelope_pan,
        (0 as libc::c_int == 0) as libc::c_int as bool_0,
    );
    let ref mut fresh3 = (*inst).env_flags;
    *fresh3 = (*fresh3 as libc::c_int
        | if (*inst).envelope_pan.env_enabled as libc::c_int != 0 {
            2 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0;
    Load_IT_Envelope(
        &mut (*inst).envelope_pitch,
        (0 as libc::c_int == 0) as libc::c_int as bool_0,
    );
    let ref mut fresh4 = (*inst).env_flags;
    *fresh4 = (*fresh4 as libc::c_int
        | if (*inst).envelope_pitch.env_enabled as libc::c_int != 0 {
            4 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0;
    if verbose != 0 {
        printf(
            b" %-3i   %3i%%    %3s   %s%s%s   %-26s \n\0" as *const u8 as *const libc::c_char,
            index + 1 as libc::c_int,
            (*inst).global_volume as libc::c_int * 100 as libc::c_int / 128 as libc::c_int,
            if (*inst).nna as libc::c_int == 0 as libc::c_int {
                b"CUT\0" as *const u8 as *const libc::c_char
            } else if (*inst).nna as libc::c_int == 1 as libc::c_int {
                b"CON\0" as *const u8 as *const libc::c_char
            } else if (*inst).nna as libc::c_int == 2 as libc::c_int {
                b"OFF\0" as *const u8 as *const libc::c_char
            } else if (*inst).nna as libc::c_int == 3 as libc::c_int {
                b"FAD\0" as *const u8 as *const libc::c_char
            } else {
                b"???\0" as *const u8 as *const libc::c_char
            },
            if (*inst).env_flags as libc::c_int & 8 as libc::c_int != 0 {
                b"V\0" as *const u8 as *const libc::c_char
            } else {
                b"-\0" as *const u8 as *const libc::c_char
            },
            if (*inst).env_flags as libc::c_int & 2 as libc::c_int != 0 {
                b"P\0" as *const u8 as *const libc::c_char
            } else {
                b"-\0" as *const u8 as *const libc::c_char
            },
            if (*inst).env_flags as libc::c_int & 4 as libc::c_int != 0 {
                b"T\0" as *const u8 as *const libc::c_char
            } else {
                b"-\0" as *const u8 as *const libc::c_char
            },
            ((*inst).name).as_mut_ptr(),
        );
    }
    skip8(7 as libc::c_int as u32_0);
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Create_IT_Instrument(mut inst: *mut Instrument, mut sample: libc::c_int) {
    let mut x: libc::c_int = 0;
    memset(
        inst as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Instrument>() as libc::c_ulong,
    );
    (*inst).global_volume = 128 as libc::c_int as u8_0;
    x = 0 as libc::c_int;
    while x < 120 as libc::c_int {
        (*inst).notemap[x as usize] = (x + sample * 256 as libc::c_int) as u16_0;
        x += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT_Sample(mut samp: *mut Sample) -> libc::c_int {
    let mut bit16: bool_0 = 0;
    let mut hasloop: bool_0 = 0;
    let mut pingpong: bool_0 = 0;
    let mut samp_unsigned = 0 as libc::c_int as bool_0;
    let mut a: u8_0 = 0;
    let mut samp_length: u32_0 = 0;
    let mut loop_start: u32_0 = 0;
    let mut loop_end: u32_0 = 0;
    let mut c5spd: u32_0 = 0;
    let mut data_address: u32_0 = 0;
    let mut x: libc::c_int = 0;
    memset(
        samp as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Sample>() as libc::c_ulong,
    );
    (*samp).msl_index = 0xffff as libc::c_int as u16_0;
    if read32() != 1397771593i32 as libc::c_uint {
        return 0x6 as libc::c_int;
    }
    x = 0 as libc::c_int;
    while x < 12 as libc::c_int {
        (*samp).filename[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    if read8() as libc::c_int != 0 as libc::c_int {
        return 0x6 as libc::c_int;
    }
    (*samp).global_volume = read8();
    a = read8();
    (*samp).it_compression = (if a as libc::c_int & 8 as libc::c_int != 0 {
        1 as libc::c_int
    } else {
        0 as libc::c_int
    }) as u8_0;
    bit16 = (a as libc::c_int & 2 as libc::c_int) as bool_0;
    hasloop = (a as libc::c_int & 16 as libc::c_int) as bool_0;
    pingpong = (a as libc::c_int & 64 as libc::c_int) as bool_0;
    (*samp).default_volume = read8();
    x = 0 as libc::c_int;
    while x < 26 as libc::c_int {
        (*samp).name[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    a = read8();
    (*samp).default_panning = read8();
    (*samp).default_panning =
        ((if (*samp).default_panning as libc::c_int & 127 as libc::c_int == 64 as libc::c_int {
            127 as libc::c_int
        } else {
            ((*samp).default_panning as libc::c_int) << 1 as libc::c_int
        }) | (*samp).default_panning as libc::c_int & 128 as libc::c_int) as u8_0;
    if a as libc::c_int & 1 as libc::c_int == 0 {
        samp_unsigned = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    }
    samp_length = read32();
    loop_start = read32();
    loop_end = read32();
    c5spd = read32();
    (*samp).frequency = c5spd;
    (*samp).sample_length = samp_length;
    (*samp).loop_start = loop_start;
    (*samp).loop_end = loop_end;
    skip8(8 as libc::c_int as u32_0);
    data_address = read32();
    (*samp).vibspeed = read8();
    (*samp).vibdepth = read8();
    (*samp).vibrate = read8();
    (*samp).vibtype = read8();
    (*samp).datapointer = data_address;
    if hasloop != 0 {
        if pingpong != 0 {
            (*samp).loop_type = 2 as libc::c_int as u8_0;
        } else {
            (*samp).loop_type = 1 as libc::c_int as u8_0;
        }
        (*samp).loop_start = loop_start;
        (*samp).loop_end = loop_end;
    } else {
        (*samp).loop_type = 0 as libc::c_int as u8_0;
    }
    (*samp).format = ((if bit16 as libc::c_int != 0 {
        0x1 as libc::c_int
    } else {
        0 as libc::c_int
    }) | (if samp_unsigned as libc::c_int != 0 {
        0 as libc::c_int
    } else {
        0x2 as libc::c_int
    })) as u8_0;
    if (*samp).sample_length == 0 as libc::c_int as libc::c_uint {
        (*samp).loop_type = 0 as libc::c_int as u8_0;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT_SampleData(mut samp: *mut Sample, mut cwmt: u16_0) -> libc::c_int {
    let mut x: u32_0 = 0;
    let mut a: libc::c_int = 0;
    if (*samp).sample_length == 0 as libc::c_int as libc::c_uint {
        return 0 as libc::c_int;
    }
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        let ref mut fresh5 = (*samp).data;
        *fresh5 = malloc(
            ((*samp).sample_length).wrapping_mul(2 as libc::c_int as libc::c_uint) as libc::c_ulong,
        ) as *mut u16_0 as *mut libc::c_void;
    } else {
        let ref mut fresh6 = (*samp).data;
        *fresh6 = malloc((*samp).sample_length as libc::c_ulong) as *mut u8_0 as *mut libc::c_void;
    }
    if (*samp).it_compression == 0 {
        x = 0 as libc::c_int as u32_0;
        while x < (*samp).sample_length {
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                if (*samp).format as libc::c_int & 0x2 as libc::c_int == 0 {
                    a = read16() as libc::c_int;
                } else {
                    a = read16() as libc::c_short as libc::c_int;
                    a += 32768 as libc::c_int;
                }
                *((*samp).data as *mut u16_0).offset(x as isize) = a as u16_0;
            } else {
                if (*samp).format as libc::c_int & 0x2 as libc::c_int == 0 {
                    a = read8() as libc::c_int;
                } else {
                    a = read8() as libc::c_schar as libc::c_int;
                    a += 128 as libc::c_int;
                }
                *((*samp).data as *mut u8_0).offset(x as isize) = a as u8_0;
            }
            x = x.wrapping_add(1);
        }
    } else {
        Load_IT_Sample_CMP(
            (*samp).data as *mut u8_0,
            (*samp).sample_length as libc::c_int,
            cwmt,
            ((*samp).format as libc::c_int & 0x1 as libc::c_int) as bool_0,
        );
    }
    FixSample(samp);
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Empty_IT_Pattern(mut patt: *mut Pattern) -> libc::c_int {
    let mut x: libc::c_int = 0;
    memset(
        patt as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Pattern>() as libc::c_ulong,
    );
    (*patt).nrows = 64 as libc::c_int as u16_0;
    x = 0 as libc::c_int;
    while x < (*patt).nrows as libc::c_int * 32 as libc::c_int {
        (*patt).data[x as usize].note = 250 as libc::c_int as u8_0;
        (*patt).data[x as usize].vol = 255 as libc::c_int as u8_0;
        x += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT_Pattern(mut patt: *mut Pattern) -> libc::c_int {
    let mut x: libc::c_int = 0;
    let mut clength: libc::c_int = 0;
    let mut chanvar: u8_0 = 0;
    let mut chan: u8_0 = 0;
    let mut maskvar: u8_0 = 0;
    let mut old_maskvar: [u8_0; 32] = [0; 32];
    let mut old_note: [u8_0; 32] = [0; 32];
    let mut old_inst: [u8_0; 32] = [0; 32];
    let mut old_vol: [u8_0; 32] = [0; 32];
    let mut old_fx: [u8_0; 32] = [0; 32];
    let mut old_param: [u8_0; 32] = [0; 32];
    memset(
        patt as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Pattern>() as libc::c_ulong,
    );
    clength = read16() as libc::c_int;
    (*patt).nrows = read16();
    skip8(4 as libc::c_int as u32_0);
    (*patt).clength = clength;
    x = 0 as libc::c_int;
    while x < (*patt).nrows as libc::c_int * 32 as libc::c_int {
        (*patt).data[x as usize].note = 250 as libc::c_int as u8_0;
        (*patt).data[x as usize].vol = 255 as libc::c_int as u8_0;
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*patt).nrows as libc::c_int {
        loop {
            chanvar = read8();
            if chanvar as libc::c_int == 0 as libc::c_int {
                break;
            }
            chan = (chanvar as libc::c_int - 1 as libc::c_int & 63 as libc::c_int) as u8_0;
            if chan as libc::c_int >= 32 as libc::c_int {
                return 0x5 as libc::c_int;
            }
            if chanvar as libc::c_int & 128 as libc::c_int != 0 {
                old_maskvar[chan as usize] = read8();
            }
            maskvar = old_maskvar[chan as usize];
            if maskvar as libc::c_int & 1 as libc::c_int != 0 {
                old_note[chan as usize] = read8();
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].note =
                    old_note[chan as usize];
            }
            if maskvar as libc::c_int & 2 as libc::c_int != 0 {
                old_inst[chan as usize] = read8();
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].inst =
                    old_inst[chan as usize];
            }
            if maskvar as libc::c_int & 4 as libc::c_int != 0 {
                old_vol[chan as usize] = read8();
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].vol =
                    old_vol[chan as usize];
            }
            if maskvar as libc::c_int & 8 as libc::c_int != 0 {
                old_fx[chan as usize] = read8();
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].fx =
                    old_fx[chan as usize];
                old_param[chan as usize] = read8();
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].param =
                    old_param[chan as usize];
            }
            if maskvar as libc::c_int & 16 as libc::c_int != 0 {
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].note =
                    old_note[chan as usize];
            }
            if maskvar as libc::c_int & 32 as libc::c_int != 0 {
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].inst =
                    old_inst[chan as usize];
            }
            if maskvar as libc::c_int & 64 as libc::c_int != 0 {
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].vol =
                    old_vol[chan as usize];
            }
            if maskvar as libc::c_int & 128 as libc::c_int != 0 {
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].fx =
                    old_fx[chan as usize];
                (*patt).data[(x * 32 as libc::c_int + chan as libc::c_int) as usize].param =
                    old_param[chan as usize];
            }
        }
        x += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT(mut itm: *mut MAS_Module, mut verbose: bool_0) -> libc::c_int {
    let mut b: u8_0 = 0;
    let mut w: u16_0 = 0;
    let mut x: libc::c_int = 0;
    let mut cc: libc::c_int = 0;
    let mut cwt: u16_0 = 0;
    let mut cmwt: u16_0 = 0;
    let mut parap_inst = 0 as *mut u32_0;
    let mut parap_samp = 0 as *mut u32_0;
    let mut parap_patt = 0 as *mut u32_0;
    let mut instr_mode: bool_0 = 0;
    memset(
        itm as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<MAS_Module>() as libc::c_ulong,
    );
    if read32() != 1297108297i32 as libc::c_uint {
        return 0x1 as libc::c_int;
    }
    x = 0 as libc::c_int;
    while x < 28 as libc::c_int {
        (*itm).title[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    (*itm).order_count = read16();
    (*itm).inst_count = read16() as u8_0;
    (*itm).samp_count = read16() as u8_0;
    (*itm).patt_count = read16() as u8_0;
    cwt = read16();
    cmwt = read16();
    w = read16();
    (*itm).stereo = (w as libc::c_int & 1 as libc::c_int) as bool_0;
    instr_mode = (w as libc::c_int & 4 as libc::c_int) as bool_0;
    (*itm).inst_mode = instr_mode;
    (*itm).freq_mode = (w as libc::c_int & 8 as libc::c_int) as u8_0;
    (*itm).old_effects = (w as libc::c_int & 16 as libc::c_int) as bool_0;
    (*itm).link_gxx = (w as libc::c_int & 32 as libc::c_int) as bool_0;
    skip8(2 as libc::c_int as u32_0);
    (*itm).global_volume = read8();
    skip8(1 as libc::c_int as u32_0);
    (*itm).initial_speed = read8();
    (*itm).initial_tempo = read8();
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b"Loading IT, \"%s\"\n\0" as *const u8 as *const libc::c_char,
            ((*itm).title).as_mut_ptr(),
        );
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b"#Orders......%i\n\0" as *const u8 as *const libc::c_char,
            (*itm).order_count as libc::c_int,
        );
        printf(
            b"#Instr.......%i\n\0" as *const u8 as *const libc::c_char,
            (*itm).inst_count as libc::c_int,
        );
        printf(
            b"#Samples.....%i\n\0" as *const u8 as *const libc::c_char,
            (*itm).samp_count as libc::c_int,
        );
        printf(
            b"#Patterns....%i\n\0" as *const u8 as *const libc::c_char,
            (*itm).patt_count as libc::c_int,
        );
        printf(
            b"Stereo.......%s\n\0" as *const u8 as *const libc::c_char,
            if (*itm).stereo as libc::c_int != 0 {
                b"Yes\0" as *const u8 as *const libc::c_char
            } else {
                b"No\0" as *const u8 as *const libc::c_char
            },
        );
        printf(
            b"Slides.......%s\n\0" as *const u8 as *const libc::c_char,
            if (*itm).freq_mode as libc::c_int != 0 {
                b"Linear\0" as *const u8 as *const libc::c_char
            } else {
                b"Amiga\0" as *const u8 as *const libc::c_char
            },
        );
        printf(
            b"Old Effects..%s\n\0" as *const u8 as *const libc::c_char,
            if (*itm).old_effects as libc::c_int != 0 {
                b"Yes\0" as *const u8 as *const libc::c_char
            } else {
                b"No\0" as *const u8 as *const libc::c_char
            },
        );
        printf(
            b"Global Vol...%i%%\n\0" as *const u8 as *const libc::c_char,
            (*itm).global_volume as libc::c_int * 100 as libc::c_int / 128 as libc::c_int,
        );
        printf(
            b"Speed........%i\n\0" as *const u8 as *const libc::c_char,
            (*itm).initial_speed as libc::c_int,
        );
        printf(
            b"Tempo........%i\n\0" as *const u8 as *const libc::c_char,
            (*itm).initial_tempo as libc::c_int,
        );
        printf(
            b"Instruments..%s\n\0" as *const u8 as *const libc::c_char,
            if instr_mode as libc::c_int != 0 {
                b"Yes\0" as *const u8 as *const libc::c_char
            } else {
                b"Will be supplied\0" as *const u8 as *const libc::c_char
            },
        );
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    skip8(12 as libc::c_int as u32_0);
    x = 0 as libc::c_int;
    while x < 64 as libc::c_int {
        b = read8();
        if x < 32 as libc::c_int {
            (*itm).channel_panning[x as usize] =
                (if b as libc::c_int * 4 as libc::c_int > 255 as libc::c_int {
                    255 as libc::c_int
                } else {
                    b as libc::c_int * 4 as libc::c_int
                }) as u8_0;
        }
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < 64 as libc::c_int {
        b = read8();
        if x < 32 as libc::c_int {
            (*itm).channel_volume[x as usize] = b;
        }
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*itm).order_count as libc::c_int {
        (*itm).orders[x as usize] = read8();
        x += 1;
    }
    parap_inst = malloc(
        ((*itm).inst_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<u32_0>() as libc::c_ulong),
    ) as *mut u32_0;
    parap_samp = malloc(
        ((*itm).samp_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<u32_0>() as libc::c_ulong),
    ) as *mut u32_0;
    parap_patt = malloc(
        ((*itm).patt_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<u32_0>() as libc::c_ulong),
    ) as *mut u32_0;
    x = 0 as libc::c_int;
    while x < (*itm).inst_count as libc::c_int {
        *parap_inst.offset(x as isize) = read32();
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*itm).samp_count as libc::c_int {
        *parap_samp.offset(x as isize) = read32();
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*itm).patt_count as libc::c_int {
        *parap_patt.offset(x as isize) = read32();
        x += 1;
    }
    let ref mut fresh7 = (*itm).samples;
    *fresh7 = malloc(
        ((*itm).samp_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Sample>() as libc::c_ulong),
    ) as *mut Sample;
    let ref mut fresh8 = (*itm).patterns;
    *fresh8 = malloc(
        ((*itm).patt_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Pattern>() as libc::c_ulong),
    ) as *mut Pattern;
    if instr_mode != 0 {
        let ref mut fresh9 = (*itm).instruments;
        *fresh9 = malloc(
            ((*itm).inst_count as libc::c_ulong)
                .wrapping_mul(::std::mem::size_of::<Instrument>() as libc::c_ulong),
        ) as *mut Instrument;
        if verbose != 0 {
            printf(b"Loading Instruments...\n\0" as *const u8 as *const libc::c_char);
            printf(
                b"--------------------------------------------\n\0" as *const u8
                    as *const libc::c_char,
            );
            printf(b" INDEX VOLUME  NNA   ENV   NAME\n\0" as *const u8 as *const libc::c_char);
        }
        x = 0 as libc::c_int;
        while x < (*itm).inst_count as libc::c_int {
            file_seek_read(
                *parap_inst.offset(x as isize) as libc::c_int,
                0 as libc::c_int,
            );
            Load_IT_Instrument(&mut *((*itm).instruments).offset(x as isize), verbose, x);
            x += 1;
        }
        if verbose != 0 {
            printf(
                b"--------------------------------------------\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
    }
    if verbose != 0 {
        printf(b"Loading Samples...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b" INDEX VOLUME DVOLUME LOOP   MID-C     NAME            \n\0" as *const u8
                as *const libc::c_char,
        );
    }
    x = 0 as libc::c_int;
    while x < (*itm).samp_count as libc::c_int {
        file_seek_read(
            *parap_samp.offset(x as isize) as libc::c_int,
            0 as libc::c_int,
        );
        Load_IT_Sample(&mut *((*itm).samples).offset(x as isize));
        if verbose != 0 {
            printf(
                b" %-3i   %3i%%   %3i%%    %4s  %6ihz   %-26s \n\0" as *const u8
                    as *const libc::c_char,
                x + 1 as libc::c_int,
                (*((*itm).samples).offset(x as isize)).global_volume as libc::c_int
                    * 100 as libc::c_int
                    / 64 as libc::c_int,
                (*((*itm).samples).offset(x as isize)).default_volume as libc::c_int
                    * 100 as libc::c_int
                    / 64 as libc::c_int,
                if (*((*itm).samples).offset(x as isize)).loop_type as libc::c_int
                    == 0 as libc::c_int
                {
                    b"None\0" as *const u8 as *const libc::c_char
                } else if (*((*itm).samples).offset(x as isize)).loop_type as libc::c_int
                    == 1 as libc::c_int
                {
                    b"Forw\0" as *const u8 as *const libc::c_char
                } else {
                    b"BIDI\0" as *const u8 as *const libc::c_char
                },
                (*((*itm).samples).offset(x as isize)).frequency,
                ((*((*itm).samples).offset(x as isize)).name).as_mut_ptr(),
            );
        }
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    if instr_mode == 0 {
        if verbose != 0 {
            printf(b"Adding Instrument Templates...\n\0" as *const u8 as *const libc::c_char);
            printf(
                b"--------------------------------------------\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
        (*itm).inst_count = (*itm).samp_count;
        let ref mut fresh10 = (*itm).instruments;
        *fresh10 = malloc(
            ((*itm).inst_count as libc::c_ulong)
                .wrapping_mul(::std::mem::size_of::<Instrument>() as libc::c_ulong),
        ) as *mut Instrument;
        cc = 0 as libc::c_int;
        x = 0 as libc::c_int;
        while x < (*itm).samp_count as libc::c_int {
            if verbose != 0 {
                printf(
                    b" * %2i\0" as *const u8 as *const libc::c_char,
                    x + 1 as libc::c_int,
                );
                cc += 1;
                if cc == 15 as libc::c_int {
                    cc = 0 as libc::c_int;
                    printf(b"\n\0" as *const u8 as *const libc::c_char);
                }
            }
            Create_IT_Instrument(
                &mut *((*itm).instruments).offset(x as isize),
                x + 1 as libc::c_int,
            );
            x += 1;
        }
        if verbose != 0 {
            if cc != 0 as libc::c_int {
                printf(
                    if (x + 1 as libc::c_int) % 15 as libc::c_int == 0 as libc::c_int {
                        b"\0" as *const u8 as *const libc::c_char
                    } else {
                        b"\n\0" as *const u8 as *const libc::c_char
                    },
                );
            }
            printf(
                b"--------------------------------------------\n\0" as *const u8
                    as *const libc::c_char,
            );
        }
    }
    if verbose != 0 {
        printf(b"Reading Patterns...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    cc = 0 as libc::c_int;
    x = 0 as libc::c_int;
    while x < (*itm).patt_count as libc::c_int {
        file_seek_read(
            *parap_patt.offset(x as isize) as libc::c_int,
            0 as libc::c_int,
        );
        if *parap_patt.offset(x as isize) != 0 as libc::c_int as libc::c_uint {
            if verbose != 0 {
                printf(
                    b" * %2i\0" as *const u8 as *const libc::c_char,
                    x + 1 as libc::c_int,
                );
                cc += 1;
                if cc == 15 as libc::c_int {
                    cc = 0 as libc::c_int;
                    printf(b"\n\0" as *const u8 as *const libc::c_char);
                }
            }
            Load_IT_Pattern(&mut *((*itm).patterns).offset(x as isize));
        } else {
            Empty_IT_Pattern(&mut *((*itm).patterns).offset(x as isize));
        }
        x += 1;
    }
    if verbose != 0 {
        if cc != 0 as libc::c_int {
            printf(b"\n\0" as *const u8 as *const libc::c_char);
        }
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading Sample Data...\n\0" as *const u8 as *const libc::c_char);
    }
    x = 0 as libc::c_int;
    while x < (*itm).samp_count as libc::c_int {
        file_seek_read(
            (*((*itm).samples).offset(x as isize)).datapointer as libc::c_int,
            0 as libc::c_int,
        );
        Load_IT_SampleData(&mut *((*itm).samples).offset(x as isize), cmwt);
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    free(parap_inst as *mut libc::c_void);
    free(parap_samp as *mut libc::c_void);
    free(parap_patt as *mut libc::c_void);
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT_CompressedSampleBlock(mut buffer: *mut *mut u8_0) -> libc::c_int {
    let mut size: u32_0 = 0;
    let mut x: u32_0 = 0;
    size = read16() as u32_0;
    *buffer =
        malloc(size.wrapping_add(4 as libc::c_int as libc::c_uint) as libc::c_ulong) as *mut u8_0;
    *(*buffer).offset(size.wrapping_add(0 as libc::c_int as libc::c_uint) as isize) =
        0 as libc::c_int as u8_0;
    *(*buffer).offset(size.wrapping_add(1 as libc::c_int as libc::c_uint) as isize) =
        0 as libc::c_int as u8_0;
    *(*buffer).offset(size.wrapping_add(2 as libc::c_int as libc::c_uint) as isize) =
        0 as libc::c_int as u8_0;
    *(*buffer).offset(size.wrapping_add(3 as libc::c_int as libc::c_uint) as isize) =
        0 as libc::c_int as u8_0;
    x = 0 as libc::c_int as u32_0;
    while x < size {
        *(*buffer).offset(x as isize) = read8();
        x = x.wrapping_add(1);
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_IT_Sample_CMP(
    mut p_dest_buffer: *mut u8_0,
    mut samp_len: libc::c_int,
    mut cmwt: u16_0,
    mut bit16: bool_0,
) -> libc::c_int {
    let mut c_buffer = 0 as *mut u8_0;
    let mut block_length: u16_0 = 0;
    let mut block_position: u16_0 = 0;
    let mut bit_width: u8_0 = 0;
    let mut aux_value: u32_0 = 0;
    let mut d1: s16 = 0;
    let mut d2: s16 = 0;
    let mut d18: s8 = 0;
    let mut d28: s8 = 0;
    let mut v8: s8 = 0;
    let mut v16: s16 = 0;
    let mut it215: bool_0 = 0;
    let mut border: u16_0 = 0;
    let mut tmp_shift: u8_0 = 0;
    let mut bit_readpos = 0 as libc::c_int as u32_0;
    let mut i: libc::c_int = 0;
    let mut nbits: u32_0 = 0;
    let mut dsize: u32_0 = 0;
    let mut dest8_write = p_dest_buffer;
    let mut dest16_write = p_dest_buffer as *mut u16_0;
    nbits = (if bit16 as libc::c_int != 0 {
        16 as libc::c_int
    } else {
        8 as libc::c_int
    }) as u32_0;
    dsize = (if bit16 as libc::c_int != 0 {
        4 as libc::c_int
    } else {
        3 as libc::c_int
    }) as u32_0;
    i = 0 as libc::c_int;
    while i < samp_len {
        *p_dest_buffer.offset(i as isize) = 128 as libc::c_int as u8_0;
        i += 1;
    }
    it215 = (cmwt as libc::c_int == 0x215 as libc::c_int) as libc::c_int as bool_0;
    while samp_len != 0 {
        Load_IT_CompressedSampleBlock(&mut c_buffer);
        bit_readpos = 0 as libc::c_int as u32_0;
        if bit16 != 0 {
            block_length = (if samp_len < 0x4000 as libc::c_int {
                samp_len
            } else {
                0x4000 as libc::c_int
            }) as u16_0;
        } else {
            block_length = (if samp_len < 0x8000 as libc::c_int {
                samp_len
            } else {
                0x8000 as libc::c_int
            }) as u16_0;
        }
        block_position = 0 as libc::c_int as u16_0;
        bit_width = nbits.wrapping_add(1 as libc::c_int as libc::c_uint) as u8_0;
        d28 = 0 as libc::c_int as s8;
        d18 = d28;
        d2 = d18 as s16;
        d1 = d2;
        while (block_position as libc::c_int) < block_length as libc::c_int {
            aux_value = readbits(c_buffer, bit_readpos, bit_width as libc::c_uint);
            bit_readpos = (bit_readpos as libc::c_uint).wrapping_add(bit_width as libc::c_uint)
                as u32_0 as u32_0;
            if (bit_width as libc::c_int) < 7 as libc::c_int {
                if bit16 != 0 {
                    if aux_value as libc::c_int
                        == (1 as libc::c_int) << bit_width as libc::c_int - 1 as libc::c_int
                    {
                        aux_value = (readbits(c_buffer, bit_readpos, dsize))
                            .wrapping_add(1 as libc::c_int as libc::c_uint);
                        bit_readpos =
                            (bit_readpos as libc::c_uint).wrapping_add(dsize) as u32_0 as u32_0;
                        bit_width = (if aux_value < bit_width as libc::c_uint {
                            aux_value
                        } else {
                            aux_value.wrapping_add(1 as libc::c_int as libc::c_uint)
                        }) as u8_0;
                        continue;
                    }
                } else if aux_value
                    == (1 as libc::c_int as u32_0)
                        << (bit_width as u32_0).wrapping_sub(1 as libc::c_int as libc::c_uint)
                {
                    aux_value = (readbits(c_buffer, bit_readpos, dsize))
                        .wrapping_add(1 as libc::c_int as libc::c_uint);
                    bit_readpos =
                        (bit_readpos as libc::c_uint).wrapping_add(dsize) as u32_0 as u32_0;
                    bit_width = (if aux_value < bit_width as libc::c_uint {
                        aux_value
                    } else {
                        aux_value.wrapping_add(1 as libc::c_int as libc::c_uint)
                    }) as u8_0;
                    continue;
                }
            } else if (bit_width as libc::c_uint)
                < nbits.wrapping_add(1 as libc::c_int as libc::c_uint)
            {
                if bit16 != 0 {
                    border = ((0xffff as libc::c_int
                        >> nbits
                            .wrapping_add(1 as libc::c_int as libc::c_uint)
                            .wrapping_sub(bit_width as libc::c_uint))
                        as libc::c_uint)
                        .wrapping_sub(nbits.wrapping_div(2 as libc::c_int as libc::c_uint))
                        as u16_0;
                    if aux_value as libc::c_int > border as libc::c_int
                        && aux_value as libc::c_int as libc::c_uint
                            <= (border as libc::c_int as libc::c_uint).wrapping_add(nbits)
                    {
                        aux_value = (aux_value as libc::c_uint).wrapping_sub(border as libc::c_uint)
                            as u32_0 as u32_0;
                        bit_width = (if aux_value < bit_width as libc::c_uint {
                            aux_value
                        } else {
                            aux_value.wrapping_add(1 as libc::c_int as libc::c_uint)
                        }) as u8_0;
                        continue;
                    }
                } else {
                    border = ((0xff as libc::c_int
                        >> nbits
                            .wrapping_add(1 as libc::c_int as libc::c_uint)
                            .wrapping_sub(bit_width as libc::c_uint))
                        as libc::c_uint)
                        .wrapping_sub(nbits.wrapping_div(2 as libc::c_int as libc::c_uint))
                        as u16_0;
                    if aux_value > border as libc::c_uint
                        && aux_value <= (border as libc::c_uint).wrapping_add(nbits)
                    {
                        aux_value = (aux_value as libc::c_uint).wrapping_sub(border as libc::c_uint)
                            as u32_0 as u32_0;
                        bit_width = (if aux_value < bit_width as libc::c_uint {
                            aux_value
                        } else {
                            aux_value.wrapping_add(1 as libc::c_int as libc::c_uint)
                        }) as u8_0;
                        continue;
                    }
                }
            } else if bit_width as libc::c_uint
                == nbits.wrapping_add(1 as libc::c_int as libc::c_uint)
            {
                if aux_value & ((1 as libc::c_int) << nbits) as libc::c_uint != 0 {
                    bit_width = (aux_value.wrapping_add(1 as libc::c_int as libc::c_uint)
                        & 0xff as libc::c_int as libc::c_uint)
                        as u8_0;
                    continue;
                }
            } else {
                if !c_buffer.is_null() {
                    free(c_buffer as *mut libc::c_void);
                    c_buffer = 0 as *mut u8_0;
                }
                return 0x6 as libc::c_int;
            }
            if (bit_width as libc::c_uint) < nbits {
                tmp_shift = nbits.wrapping_sub(bit_width as libc::c_uint) as u8_0;
                if bit16 != 0 {
                    v16 = (aux_value << tmp_shift as libc::c_int) as s16;
                    v16 = (v16 as libc::c_int >> tmp_shift as libc::c_int) as s16;
                } else {
                    v8 = (aux_value << tmp_shift as libc::c_int) as s8;
                    v8 = (v8 as libc::c_int >> tmp_shift as libc::c_int) as s8;
                }
            } else if bit16 != 0 {
                v16 = aux_value as s16;
            } else {
                v8 = aux_value as s8;
            }
            if bit16 != 0 {
                d1 = (d1 as libc::c_int + v16 as libc::c_int) as s16;
                d2 = (d2 as libc::c_int + d1 as libc::c_int) as s16;
                let fresh11 = dest16_write;
                dest16_write = dest16_write.offset(1);
                *fresh11 = (if it215 as libc::c_int != 0 {
                    d2 as libc::c_int + 32768 as libc::c_int
                } else {
                    d1 as libc::c_int + 32768 as libc::c_int
                }) as u16_0;
            } else {
                d18 = (d18 as libc::c_int + v8 as libc::c_int) as s8;
                d28 = (d28 as libc::c_int + d18 as libc::c_int) as s8;
                let fresh12 = dest8_write;
                dest8_write = dest8_write.offset(1);
                *fresh12 = (if it215 as libc::c_int != 0 {
                    d28 as libc::c_int + 128 as libc::c_int
                } else {
                    d18 as libc::c_int + 128 as libc::c_int
                }) as u8_0;
            }
            block_position = block_position.wrapping_add(1);
        }
        if !c_buffer.is_null() {
            free(c_buffer as *mut libc::c_void);
            c_buffer = 0 as *mut u8_0;
        }
        samp_len -= block_length as libc::c_int;
    }
    return 0 as libc::c_int;
}
