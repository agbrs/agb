use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn read8() -> u8_0;
    fn read16() -> u16_0;
    fn read32() -> u32_0;
    fn skip8(count: u32_0);
    fn file_seek_read(offset: libc::c_int, mode: libc::c_int) -> libc::c_int;
    fn file_tell_read() -> libc::c_int;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;
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
pub unsafe extern "C" fn Get_XM_Frequency(mut relnote: s8, mut finetune: s8) -> libc::c_int {
    let mut rn = relnote as libc::c_double;
    let mut ft = finetune as libc::c_double;
    let mut middle_c: libc::c_double = 0.;
    let mut freq: libc::c_double = 0.;
    middle_c = 8363.0f32 as libc::c_double;
    freq = middle_c
        * pow(
            2.0f64,
            1.0f64 / 12.0f64 * rn + 1.0f64 / (12.0f64 * 128.0f64) * ft,
        );
    return freq as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_XM_Instrument(
    mut inst: *mut Instrument,
    mut mas: *mut MAS_Module,
    mut p_nextsample: *mut u8_0,
    mut verbose: bool_0,
) -> libc::c_int {
    let mut inst_size: libc::c_int = 0;
    let mut nsamples: libc::c_int = 0;
    let mut ns: libc::c_int = 0;
    let mut samp_headstart: libc::c_int = 0;
    let mut samp_headsize: libc::c_int = 0;
    let mut inst_headstart: libc::c_int = 0;
    let mut sample_old: libc::c_int = 0;
    let mut x: libc::c_int = 0;
    let mut y: libc::c_int = 0;
    let mut t: u32_0 = 0;
    let mut vibtype: u8_0 = 0;
    let mut vibsweep: u8_0 = 0;
    let mut vibdepth: u8_0 = 0;
    let mut vibrate: u8_0 = 0;
    let mut finetune: s8 = 0;
    let mut relnote: s8 = 0;
    let mut loopbits: u8_0 = 0;
    let mut volbits: u8_0 = 0;
    let mut panbits: u8_0 = 0;
    let mut samp = 0 as *mut Sample;
    ns = *p_nextsample as libc::c_int;
    memset(
        inst as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Instrument>() as libc::c_ulong,
    );
    inst_headstart = file_tell_read();
    inst_size = read32() as libc::c_int;
    x = 0 as libc::c_int;
    while x < 22 as libc::c_int {
        (*inst).name[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    read8();
    nsamples = read16() as libc::c_int;
    if nsamples > 0 as libc::c_int {
        samp_headsize = read32() as libc::c_int;
        x = 0 as libc::c_int;
        while x < 96 as libc::c_int {
            (*inst).notemap[(x + 12 as libc::c_int) as usize] =
                ((read8() as libc::c_int + ns + 1 as libc::c_int) * 256 as libc::c_int
                    | x + 12 as libc::c_int) as u16_0;
            x += 1;
        }
        x = 0 as libc::c_int;
        while x < 12 as libc::c_int {
            (*inst).notemap[x as usize] =
                ((*inst).notemap[12 as libc::c_int as usize] as libc::c_int & 0xff00 as libc::c_int
                    | x) as u16_0;
            x += 1;
        }
        x = 96 as libc::c_int;
        while x < 120 as libc::c_int {
            (*inst).notemap[x as usize] =
                ((*inst).notemap[12 as libc::c_int as usize] as libc::c_int & 0xff00 as libc::c_int
                    | x) as u16_0;
            x += 1;
        }
        x = 0 as libc::c_int;
        while x < 12 as libc::c_int {
            (*inst).envelope_volume.node_x[x as usize] = read16();
            (*inst).envelope_volume.node_y[x as usize] = read16() as u8_0;
            x += 1;
        }
        x = 0 as libc::c_int;
        while x < 12 as libc::c_int {
            (*inst).envelope_pan.node_x[x as usize] = read16();
            (*inst).envelope_pan.node_y[x as usize] = read16() as u8_0;
            x += 1;
        }
        (*inst).global_volume = 128 as libc::c_int as u8_0;
        (*inst).envelope_volume.node_count = read8();
        (*inst).envelope_pan.node_count = read8();
        let ref mut fresh0 = (*inst).envelope_volume.sus_end;
        *fresh0 = read8();
        (*inst).envelope_volume.sus_start = *fresh0;
        (*inst).envelope_volume.loop_start = read8();
        (*inst).envelope_volume.loop_end = read8();
        let ref mut fresh1 = (*inst).envelope_pan.sus_end;
        *fresh1 = read8();
        (*inst).envelope_pan.sus_start = *fresh1;
        (*inst).envelope_pan.loop_start = read8();
        (*inst).envelope_pan.loop_end = read8();
        volbits = read8();
        panbits = read8();
        (*inst).env_flags = 0 as libc::c_int as u8_0;
        if volbits as libc::c_int & 1 as libc::c_int != 0 {
            let ref mut fresh2 = (*inst).env_flags;
            *fresh2 = (*fresh2 as libc::c_int | (1 as libc::c_int | 8 as libc::c_int)) as u8_0;
        }
        if panbits as libc::c_int & 1 as libc::c_int != 0 {
            let ref mut fresh3 = (*inst).env_flags;
            *fresh3 = (*fresh3 as libc::c_int | 2 as libc::c_int) as u8_0;
        }
        if volbits as libc::c_int & 2 as libc::c_int == 0 {
            let ref mut fresh4 = (*inst).envelope_volume.sus_end;
            *fresh4 = 255 as libc::c_int as u8_0;
            (*inst).envelope_volume.sus_start = *fresh4;
        }
        if panbits as libc::c_int & 2 as libc::c_int == 0 {
            let ref mut fresh5 = (*inst).envelope_pan.sus_end;
            *fresh5 = 255 as libc::c_int as u8_0;
            (*inst).envelope_pan.sus_start = *fresh5;
        }
        if volbits as libc::c_int & 4 as libc::c_int == 0 {
            let ref mut fresh6 = (*inst).envelope_volume.loop_end;
            *fresh6 = 255 as libc::c_int as u8_0;
            (*inst).envelope_volume.loop_start = *fresh6;
        }
        if panbits as libc::c_int & 4 as libc::c_int == 0 {
            let ref mut fresh7 = (*inst).envelope_pan.loop_end;
            *fresh7 = 255 as libc::c_int as u8_0;
            (*inst).envelope_pan.loop_start = *fresh7;
        }
        vibtype = read8();
        vibsweep = (32768 as libc::c_int / (read8() as libc::c_int + 1 as libc::c_int)) as u8_0;
        vibdepth = read8();
        vibrate = read8();
        (*inst).fadeout = (read16() as libc::c_int / 32 as libc::c_int) as u16_0;
        file_seek_read(inst_headstart + inst_size, 0 as libc::c_int);
        x = 0 as libc::c_int;
        while x < nsamples {
            if ns + x >= 256 as libc::c_int {
                return 0x9 as libc::c_int;
            }
            samp_headstart = file_tell_read();
            samp = &mut *((*mas).samples).offset((ns + x) as isize) as *mut Sample;
            memset(
                samp as *mut libc::c_void,
                0 as libc::c_int,
                ::std::mem::size_of::<Sample>() as libc::c_ulong,
            );
            (*samp).msl_index = 0xffff as libc::c_int as u16_0;
            (*samp).sample_length = read32();
            (*samp).loop_start = read32();
            (*samp).loop_end = (read32()).wrapping_add((*samp).loop_start);
            (*samp).default_volume = read8();
            (*samp).global_volume = 64 as libc::c_int as u8_0;
            (*samp).vibtype = vibtype;
            (*samp).vibdepth = vibdepth;
            (*samp).vibspeed = vibrate;
            (*samp).vibrate = vibsweep;
            finetune = read8() as s8;
            loopbits = read8();
            (*samp).default_panning =
                (read8() as libc::c_int >> 1 as libc::c_int | 128 as libc::c_int) as u8_0;
            relnote = read8() as s8;
            read8();
            y = 0 as libc::c_int;
            while y < 22 as libc::c_int {
                (*samp).name[y as usize] = read8() as libc::c_char;
                if y < 12 as libc::c_int {
                    (*samp).filename[y as usize] = (*samp).name[y as usize];
                }
                y += 1;
            }
            (*samp).frequency = Get_XM_Frequency(relnote, finetune) as u32_0;
            (*samp).format = (if loopbits as libc::c_int & 16 as libc::c_int != 0 {
                0x1 as libc::c_int
            } else {
                0 as libc::c_int
            }) as u8_0;
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                let ref mut fresh8 = (*samp).sample_length;
                *fresh8 = (*fresh8 as libc::c_uint).wrapping_div(2 as libc::c_int as libc::c_uint)
                    as u32_0 as u32_0;
                let ref mut fresh9 = (*samp).loop_start;
                *fresh9 = (*fresh9 as libc::c_uint).wrapping_div(2 as libc::c_int as libc::c_uint)
                    as u32_0 as u32_0;
                let ref mut fresh10 = (*samp).loop_end;
                *fresh10 = (*fresh10 as libc::c_uint).wrapping_div(2 as libc::c_int as libc::c_uint)
                    as u32_0 as u32_0;
            }
            (*samp).loop_type = (loopbits as libc::c_int & 3 as libc::c_int) as u8_0;
            file_seek_read(samp_headstart + samp_headsize, 0 as libc::c_int);
            x += 1;
        }
        x = 0 as libc::c_int;
        while x < nsamples {
            samp = &mut *((*mas).samples).offset((ns + x) as isize) as *mut Sample;
            if !((*samp).sample_length == 0 as libc::c_int as libc::c_uint) {
                sample_old = 0 as libc::c_int;
                if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                    let ref mut fresh11 = (*samp).data;
                    *fresh11 = malloc(
                        ((*samp).sample_length).wrapping_mul(2 as libc::c_int as libc::c_uint)
                            as libc::c_ulong,
                    ) as *mut u16_0 as *mut libc::c_void;
                    t = 0 as libc::c_int as u32_0;
                    while t < (*samp).sample_length {
                        sample_old =
                            (read16() as s16 as libc::c_int + sample_old) as s16 as libc::c_int;
                        *((*samp).data as *mut u16_0).offset(t as isize) =
                            (sample_old + 32768 as libc::c_int) as u16_0;
                        t = t.wrapping_add(1);
                    }
                } else {
                    let ref mut fresh12 = (*samp).data;
                    *fresh12 = malloc((*samp).sample_length as libc::c_ulong) as *mut u8_0
                        as *mut libc::c_void;
                    t = 0 as libc::c_int as u32_0;
                    while t < (*samp).sample_length {
                        sample_old =
                            (read8() as s8 as libc::c_int + sample_old) as s8 as libc::c_int;
                        *((*samp).data as *mut u8_0).offset(t as isize) =
                            (sample_old + 128 as libc::c_int) as u8_0;
                        t = t.wrapping_add(1);
                    }
                }
                FixSample(samp);
            }
            x += 1;
        }
        *p_nextsample = (ns + nsamples) as u8_0;
        if verbose != 0 {
            printf(
                b"  %2i   |   %s%s   | %-22s |\n\0" as *const u8 as *const libc::c_char,
                nsamples,
                if volbits as libc::c_int & 1 as libc::c_int != 0 {
                    b"V\0" as *const u8 as *const libc::c_char
                } else {
                    b"-\0" as *const u8 as *const libc::c_char
                },
                if panbits as libc::c_int & 1 as libc::c_int != 0 {
                    b"P\0" as *const u8 as *const libc::c_char
                } else {
                    b"-\0" as *const u8 as *const libc::c_char
                },
                ((*inst).name).as_mut_ptr(),
            );
        }
    } else {
        file_seek_read(inst_headstart + inst_size, 0 as libc::c_int);
        if verbose != 0 {
            printf(
                b"  --   |   --   | %-22s |\n\0" as *const u8 as *const libc::c_char,
                ((*inst).name).as_mut_ptr(),
            );
        }
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn CONV_XM_EFFECT(mut fx: *mut u8_0, mut param: *mut u8_0) {
    let mut wfx: libc::c_int = 0;
    let mut wpm: libc::c_int = 0;
    wfx = *fx as libc::c_int;
    wpm = *param as libc::c_int;
    let mut current_block_77: u64;
    match wfx {
        0 => {
            if wpm != 0 as libc::c_int {
                wfx = 'J' as i32 - 64 as libc::c_int;
            } else {
                wpm = 0 as libc::c_int;
                wfx = wpm;
            }
            current_block_77 = 15447629348493591490;
        }
        1 => {
            wfx = 'F' as i32 - 64 as libc::c_int;
            if wpm >= 0xe0 as libc::c_int {
                wpm = 0xdf as libc::c_int;
            }
            current_block_77 = 15447629348493591490;
        }
        2 => {
            wfx = 'E' as i32 - 64 as libc::c_int;
            if wpm >= 0xe0 as libc::c_int {
                wpm = 0xdf as libc::c_int;
            }
            current_block_77 = 15447629348493591490;
        }
        3 => {
            wfx = 'G' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        4 => {
            wfx = 'H' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        5 => {
            wfx = 'L' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        6 => {
            wfx = 'K' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        7 => {
            wfx = 'R' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        8 => {
            wfx = 'X' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        9 => {
            wfx = 'O' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        10 => {
            wfx = 'D' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        11 => {
            wfx = 'B' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        12 => {
            wfx = 27 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        13 => {
            wfx = 'C' as i32 - 64 as libc::c_int;
            wpm = (wpm & 0xf as libc::c_int) + (wpm >> 4 as libc::c_int) * 10 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        14 => {
            match wpm >> 4 as libc::c_int {
                1 => {
                    wfx = 'F' as i32 - 64 as libc::c_int;
                    wpm = 0xf0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                2 => {
                    wfx = 'E' as i32 - 64 as libc::c_int;
                    wpm = 0xf0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                3 | 5 => {
                    wfx = 0 as libc::c_int;
                    wpm = 0 as libc::c_int;
                }
                4 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0x30 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                6 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0xb0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                7 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0x40 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                8 => {
                    wfx = 'X' as i32 - 64 as libc::c_int;
                    wpm = (wpm & 0xf as libc::c_int) * 16 as libc::c_int;
                }
                9 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0x20 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                10 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                11 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0x10 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                12 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0xc0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                13 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0xd0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                14 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = 0xe0 as libc::c_int | wpm & 0xf as libc::c_int;
                }
                15 => {
                    wfx = 'S' as i32 - 64 as libc::c_int;
                    wpm = wpm;
                }
                0 => {
                    wfx = 0 as libc::c_int;
                    wpm = 0 as libc::c_int;
                }
                _ => {}
            }
            current_block_77 = 15447629348493591490;
        }
        15 => {
            if wpm >= 32 as libc::c_int {
                wfx = 'T' as i32 - 64 as libc::c_int;
            } else {
                wfx = 'A' as i32 - 64 as libc::c_int;
            }
            current_block_77 = 15447629348493591490;
        }
        16 => {
            wfx = 'V' as i32 - 64 as libc::c_int;
            wpm = wpm;
            current_block_77 = 15447629348493591490;
        }
        17 => {
            wfx = 'W' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        18 => {
            current_block_77 = 6294134568384158399;
        }
        19 => {
            current_block_77 = 6294134568384158399;
        }
        22 => {
            current_block_77 = 14287426202897139007;
        }
        23 => {
            current_block_77 = 7034867951062344145;
        }
        24 => {
            current_block_77 = 3380490629895039722;
        }
        26 => {
            current_block_77 = 772399669957145217;
        }
        28 => {
            current_block_77 = 12253739152035783869;
        }
        30 => {
            current_block_77 = 17665181095911590905;
        }
        31 => {
            current_block_77 = 3862714763397931078;
        }
        32 => {
            current_block_77 = 17103469524162562953;
        }
        34 | 35 => {
            current_block_77 = 7469032322570361295;
        }
        20 => {
            wfx = 28 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        21 => {
            wfx = 29 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        25 => {
            wfx = 'P' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        27 => {
            wfx = 'Q' as i32 - 64 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        29 => {
            wfx = 30 as libc::c_int;
            current_block_77 = 15447629348493591490;
        }
        33 => {
            if wpm >> 4 as libc::c_int == 1 as libc::c_int {
                wfx = 'F' as i32 - 64 as libc::c_int;
                wpm = 0xe0 as libc::c_int | wpm & 0xf as libc::c_int;
            } else if wpm >> 4 as libc::c_int == 2 as libc::c_int {
                wfx = 'E' as i32 - 64 as libc::c_int;
                wpm = 0xe0 as libc::c_int | wpm & 0xf as libc::c_int;
            } else {
                wfx = 0 as libc::c_int;
                wpm = 0 as libc::c_int;
            }
            current_block_77 = 15447629348493591490;
        }
        _ => {
            current_block_77 = 15447629348493591490;
        }
    }
    match current_block_77 {
        6294134568384158399 => {
            current_block_77 = 14287426202897139007;
        }
        _ => {}
    }
    match current_block_77 {
        14287426202897139007 => {
            current_block_77 = 7034867951062344145;
        }
        _ => {}
    }
    match current_block_77 {
        7034867951062344145 => {
            current_block_77 = 3380490629895039722;
        }
        _ => {}
    }
    match current_block_77 {
        3380490629895039722 => {
            current_block_77 = 772399669957145217;
        }
        _ => {}
    }
    match current_block_77 {
        772399669957145217 => {
            current_block_77 = 12253739152035783869;
        }
        _ => {}
    }
    match current_block_77 {
        12253739152035783869 => {
            current_block_77 = 17665181095911590905;
        }
        _ => {}
    }
    match current_block_77 {
        17665181095911590905 => {
            current_block_77 = 3862714763397931078;
        }
        _ => {}
    }
    match current_block_77 {
        3862714763397931078 => {
            current_block_77 = 17103469524162562953;
        }
        _ => {}
    }
    match current_block_77 {
        17103469524162562953 => {
            current_block_77 = 7469032322570361295;
        }
        _ => {}
    }
    match current_block_77 {
        7469032322570361295 => {
            wfx = 0 as libc::c_int;
            wpm = 0 as libc::c_int;
        }
        _ => {}
    }
    *fx = wfx as u8_0;
    *param = wpm as u8_0;
}
#[no_mangle]
pub unsafe extern "C" fn Load_XM_Pattern(
    mut patt: *mut Pattern,
    mut nchannels: u32_0,
    mut verbose: bool_0,
) -> libc::c_int {
    let mut headsize: u32_0 = 0;
    let mut headstart: u32_0 = 0;
    let mut clength: u16_0 = 0;
    let mut row: u16_0 = 0;
    let mut col: u16_0 = 0;
    let mut b: u8_0 = 0;
    let mut e: u32_0 = 0;
    let mut fx: u8_0 = 0;
    let mut param: u8_0 = 0;
    headstart = file_tell_read() as u32_0;
    headsize = read32();
    if read8() as libc::c_int != 0 as libc::c_int {
        return 0x7 as libc::c_int;
    }
    memset(
        patt as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Pattern>() as libc::c_ulong,
    );
    (*patt).nrows = read16();
    clength = read16();
    if verbose != 0 {
        printf(
            b"- %i rows, %.2f KB\n\0" as *const u8 as *const libc::c_char,
            (*patt).nrows as libc::c_int,
            (clength as libc::c_float / 1000 as libc::c_int as libc::c_float) as libc::c_double,
        );
    }
    row = 0 as libc::c_int as u16_0;
    while (row as libc::c_int) < (*patt).nrows as libc::c_int * 32 as libc::c_int {
        (*patt).data[row as usize].note = 250 as libc::c_int as u8_0;
        (*patt).data[row as usize].vol = 0 as libc::c_int as u8_0;
        row = row.wrapping_add(1);
    }
    file_seek_read(
        headstart.wrapping_add(headsize) as libc::c_int,
        0 as libc::c_int,
    );
    if clength as libc::c_int == 0 as libc::c_int {
        return 0 as libc::c_int;
    }
    row = 0 as libc::c_int as u16_0;
    while (row as libc::c_int) < (*patt).nrows as libc::c_int {
        col = 0 as libc::c_int as u16_0;
        while (col as libc::c_uint) < nchannels {
            e = (row as libc::c_int * 32 as libc::c_int + col as libc::c_int) as u32_0;
            b = read8();
            if b as libc::c_int & 128 as libc::c_int != 0 {
                if b as libc::c_int & 1 as libc::c_int != 0 {
                    (*patt).data[e as usize].note = read8();
                    if (*patt).data[e as usize].note as libc::c_int == 97 as libc::c_int {
                        (*patt).data[e as usize].note = 255 as libc::c_int as u8_0;
                    } else {
                        let ref mut fresh13 = (*patt).data[e as usize].note;
                        *fresh13 = (*fresh13 as libc::c_int
                            + (12 as libc::c_int - 1 as libc::c_int))
                            as u8_0;
                    }
                }
                if b as libc::c_int & 2 as libc::c_int != 0 {
                    (*patt).data[e as usize].inst = read8();
                }
                if b as libc::c_int & 4 as libc::c_int != 0 {
                    (*patt).data[e as usize].vol = read8();
                }
                if b as libc::c_int & 8 as libc::c_int != 0 {
                    fx = read8();
                } else {
                    fx = 0 as libc::c_int as u8_0;
                }
                if b as libc::c_int & 16 as libc::c_int != 0 {
                    param = read8();
                } else {
                    param = 0 as libc::c_int as u8_0;
                }
                if fx as libc::c_int != 0 as libc::c_int || param as libc::c_int != 0 as libc::c_int
                {
                    CONV_XM_EFFECT(&mut fx, &mut param);
                    (*patt).data[e as usize].fx = fx;
                    (*patt).data[e as usize].param = param;
                }
            } else {
                (*patt).data[e as usize].note = b;
                if (*patt).data[e as usize].note as libc::c_int == 97 as libc::c_int {
                    (*patt).data[e as usize].note = 255 as libc::c_int as u8_0;
                } else {
                    let ref mut fresh14 = (*patt).data[e as usize].note;
                    *fresh14 =
                        (*fresh14 as libc::c_int + (12 as libc::c_int - 1 as libc::c_int)) as u8_0;
                }
                (*patt).data[e as usize].inst = read8();
                (*patt).data[e as usize].vol = read8();
                fx = read8();
                param = read8();
                CONV_XM_EFFECT(&mut fx, &mut param);
                (*patt).data[e as usize].fx = fx;
                (*patt).data[e as usize].param = param;
            }
            col = col.wrapping_add(1);
        }
        row = row.wrapping_add(1);
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_XM(mut mod_0: *mut MAS_Module, mut verbose: bool_0) -> libc::c_int {
    let mut x: libc::c_int = 0;
    let mut xm_version: u16_0 = 0;
    let mut xm_headsize: u32_0 = 0;
    let mut xm_nchannels: u16_0 = 0;
    let mut next_sample: u8_0 = 0;
    memset(
        mod_0 as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<MAS_Module>() as libc::c_ulong,
    );
    (*mod_0).old_effects = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    (*mod_0).xm_mode = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    (*mod_0).global_volume = 64 as libc::c_int as u8_0;
    (*mod_0).old_mode = 0 as libc::c_int as bool_0;
    if read32() != 1702131781i32 as libc::c_uint
        || read32() != 1684366446i32 as libc::c_uint
        || read32() != 1685015840i32 as libc::c_uint
        || read32() != 979725429i32 as libc::c_uint
        || read8() as libc::c_int != ' ' as i32
    {
        return 0x1 as libc::c_int;
    }
    x = 0 as libc::c_int;
    while x < 20 as libc::c_int {
        (*mod_0).title[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b"Loading XM, \"%s\"\n\0" as *const u8 as *const libc::c_char,
            ((*mod_0).title).as_mut_ptr(),
        );
    }
    if read8() as libc::c_int != 0x1a as libc::c_int {
        return 0x1 as libc::c_int;
    }
    skip8(20 as libc::c_int as u32_0);
    xm_version = read16();
    xm_headsize = read32();
    (*mod_0).order_count = read16() as u8_0 as u16_0;
    (*mod_0).restart_pos = read16() as u8_0;
    xm_nchannels = read16();
    (*mod_0).patt_count = read16() as u8_0;
    (*mod_0).inst_count = read16() as u8_0;
    (*mod_0).freq_mode = (if read16() as libc::c_int & 1 as libc::c_int != 0 {
        (0 as libc::c_int == 0) as libc::c_int
    } else {
        0 as libc::c_int
    }) as u8_0;
    (*mod_0).initial_speed = read16() as u8_0;
    (*mod_0).initial_tempo = read16() as u8_0;
    if verbose != 0 {
        printf(
            b"Version....%i.%i\n\0" as *const u8 as *const libc::c_char,
            xm_version as libc::c_int >> 8 as libc::c_int & 0xff as libc::c_int,
            xm_version as libc::c_int & 0xff as libc::c_int,
        );
        printf(
            b"Length.....%i\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).order_count as libc::c_int,
        );
        printf(
            b"Restart....%i\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).restart_pos as libc::c_int,
        );
        printf(
            b"Channels...%i\n\0" as *const u8 as *const libc::c_char,
            xm_nchannels as libc::c_int,
        );
        printf(
            b"#Patterns..%i\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).patt_count as libc::c_int,
        );
        printf(
            b"#Instr.....%i\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).inst_count as libc::c_int,
        );
        printf(
            b"Freq Mode..%s\n\0" as *const u8 as *const libc::c_char,
            if (*mod_0).freq_mode as libc::c_int != 0 {
                b"Linear\0" as *const u8 as *const libc::c_char
            } else {
                b"Amiga\0" as *const u8 as *const libc::c_char
            },
        );
        printf(
            b"Speed......%i\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).initial_speed as libc::c_int,
        );
        printf(
            b"Tempo......%i\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).initial_tempo as libc::c_int,
        );
    }
    x = 0 as libc::c_int;
    while x < 32 as libc::c_int {
        (*mod_0).channel_volume[x as usize] = 64 as libc::c_int as u8_0;
        (*mod_0).channel_panning[x as usize] = 128 as libc::c_int as u8_0;
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Reading sequence...\n\0" as *const u8 as *const libc::c_char);
    }
    x = 0 as libc::c_int;
    while x < 200 as libc::c_int {
        if x < (*mod_0).order_count as libc::c_int {
            (*mod_0).orders[x as usize] = read8();
        } else {
            read8();
            (*mod_0).orders[x as usize] = 255 as libc::c_int as u8_0;
        }
        x += 1;
    }
    while x < 256 as libc::c_int {
        read8();
        x += 1;
    }
    file_seek_read(
        (60 as libc::c_int as libc::c_uint).wrapping_add(xm_headsize) as libc::c_int,
        0 as libc::c_int,
    );
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading patterns...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    let ref mut fresh15 = (*mod_0).patterns;
    *fresh15 = malloc(
        ((*mod_0).patt_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Pattern>() as libc::c_ulong),
    ) as *mut Pattern;
    x = 0 as libc::c_int;
    while x < (*mod_0).patt_count as libc::c_int {
        if verbose != 0 {
            printf(
                b" Pattern %2i \0" as *const u8 as *const libc::c_char,
                x + 1 as libc::c_int,
            );
        }
        Load_XM_Pattern(
            &mut *((*mod_0).patterns).offset(x as isize),
            xm_nchannels as u32_0,
            verbose,
        );
        x += 1;
    }
    let ref mut fresh16 = (*mod_0).instruments;
    *fresh16 = malloc(
        ((*mod_0).inst_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Instrument>() as libc::c_ulong),
    ) as *mut Instrument;
    let ref mut fresh17 = (*mod_0).samples;
    *fresh17 = malloc(
        (256 as libc::c_int as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Sample>() as libc::c_ulong),
    ) as *mut Sample;
    next_sample = 0 as libc::c_int as u8_0;
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading instruments...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b".-----------------------------------------------.\n\0" as *const u8
                as *const libc::c_char,
        );
        printf(
            b"|INDEX|SAMPLES|ENVELOPE|          NAME          |\n\0" as *const u8
                as *const libc::c_char,
        );
        printf(
            b"|-----+-------+--------+------------------------|\n\0" as *const u8
                as *const libc::c_char,
        );
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).inst_count as libc::c_int {
        if verbose != 0 {
            printf(
                b"|%3i  |\0" as *const u8 as *const libc::c_char,
                x + 1 as libc::c_int,
            );
        }
        Load_XM_Instrument(
            &mut *((*mod_0).instruments).offset(x as isize),
            mod_0,
            &mut next_sample,
            verbose,
        );
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"`-----------------------------------------------'\n\0" as *const u8
                as *const libc::c_char,
        );
    }
    (*mod_0).samp_count = next_sample;
    return 0 as libc::c_int;
}
