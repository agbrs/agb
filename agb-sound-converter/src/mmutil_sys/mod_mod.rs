use ::libc;
extern "C" {
    fn strtol(_: *const libc::c_char, _: *mut *mut libc::c_char, _: libc::c_int) -> libc::c_long;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn memset(_: *mut libc::c_void, _: libc::c_int, _: libc::c_ulong) -> *mut libc::c_void;
    fn log(_: libc::c_double) -> libc::c_double;
    fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;
    fn round(_: libc::c_double) -> libc::c_double;
    static mut PANNING_SEP: libc::c_int;
    fn CONV_XM_EFFECT(fx: *mut u8_0, param: *mut u8_0);
    fn read8() -> u8_0;
    fn read32() -> u32_0;
    fn file_seek_read(offset: libc::c_int, mode: libc::c_int) -> libc::c_int;
    fn file_tell_read() -> libc::c_int;
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
#[inline]
unsafe extern "C" fn atoi(mut __nptr: *const libc::c_char) -> libc::c_int {
    return strtol(
        __nptr,
        0 as *mut libc::c_void as *mut *mut libc::c_char,
        10 as libc::c_int,
    ) as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Create_MOD_Instrument(
    mut inst: *mut Instrument,
    mut sample: u8_0,
) -> libc::c_int {
    let mut x: libc::c_int = 0;
    memset(
        inst as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Instrument>() as libc::c_ulong,
    );
    (*inst).global_volume = 128 as libc::c_int as u8_0;
    x = 0 as libc::c_int;
    while x < 120 as libc::c_int {
        (*inst).notemap[x as usize] =
            (x | (sample as libc::c_int + 1 as libc::c_int) << 8 as libc::c_int) as u16_0;
        x += 1;
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_MOD_SampleData(mut samp: *mut Sample) -> libc::c_int {
    let mut t: u32_0 = 0;
    if (*samp).sample_length > 0 as libc::c_int as libc::c_uint {
        let ref mut fresh0 = (*samp).data;
        *fresh0 = malloc((*samp).sample_length as libc::c_ulong) as *mut u8_0 as *mut libc::c_void;
        t = 0 as libc::c_int as u32_0;
        while t < (*samp).sample_length {
            *((*samp).data as *mut u8_0).offset(t as isize) =
                (read8() as libc::c_int + 128 as libc::c_int) as u8_0;
            t = t.wrapping_add(1);
        }
    }
    FixSample(samp);
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_MOD_Pattern(
    mut patt: *mut Pattern,
    mut nchannels: u8_0,
    mut inst_count: *mut u8_0,
) -> libc::c_int {
    let mut data1: u8_0 = 0;
    let mut data2: u8_0 = 0;
    let mut data3: u8_0 = 0;
    let mut data4: u8_0 = 0;
    let mut period: u16_0 = 0;
    let mut inst: u8_0 = 0;
    let mut effect: u8_0 = 0;
    let mut param: u8_0 = 0;
    let mut row: u32_0 = 0;
    let mut col: u32_0 = 0;
    let mut p = 0 as *mut PatternEntry;
    memset(
        patt as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Pattern>() as libc::c_ulong,
    );
    (*patt).nrows = 64 as libc::c_int as u16_0;
    row = 0 as libc::c_int as u32_0;
    while row < (64 as libc::c_int * 32 as libc::c_int) as libc::c_uint {
        (*patt).data[row as usize].note = 250 as libc::c_int as u8_0;
        row = row.wrapping_add(1);
    }
    row = 0 as libc::c_int as u32_0;
    while row < 64 as libc::c_int as libc::c_uint {
        col = 0 as libc::c_int as u32_0;
        while col < nchannels as libc::c_uint {
            data1 = read8();
            data2 = read8();
            data3 = read8();
            data4 = read8();
            period = ((data1 as libc::c_int & 0xf as libc::c_int) * 256 as libc::c_int
                + data2 as libc::c_int) as u16_0;
            inst = ((data1 as libc::c_int & 0xf0 as libc::c_int)
                + (data3 as libc::c_int >> 4 as libc::c_int)) as u8_0;
            effect = (data3 as libc::c_int & 0xf as libc::c_int) as u8_0;
            param = data4;
            match effect as libc::c_int {
                5 | 6 => {
                    if param as libc::c_int & 0xf0 as libc::c_int != 0 {
                        param = (param as libc::c_int & 0xf0 as libc::c_int) as u8_0;
                    }
                }
                _ => {}
            }
            p = &mut *((*patt).data).as_mut_ptr().offset(
                row.wrapping_mul(32 as libc::c_int as libc::c_uint)
                    .wrapping_add(col) as isize,
            ) as *mut PatternEntry;
            (*p).inst = inst;
            CONV_XM_EFFECT(&mut effect, &mut param);
            (*p).fx = effect;
            (*p).param = param;
            if period as libc::c_int != 0 as libc::c_int {
                (*p).note = (round(
                    12.0f64 * log(856.0f64 / period as libc::c_double)
                        / log(2 as libc::c_int as libc::c_double),
                ) as libc::c_int
                    + 37 as libc::c_int
                    + 11 as libc::c_int) as u8_0;
            }
            if (*inst_count as libc::c_int) < inst as libc::c_int + 1 as libc::c_int {
                *inst_count = (inst as libc::c_int + 1 as libc::c_int) as u8_0;
                if *inst_count as libc::c_int > 31 as libc::c_int {
                    *inst_count = 31 as libc::c_int as u8_0;
                }
            }
            col = col.wrapping_add(1);
        }
        row = row.wrapping_add(1);
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_MOD_Sample(
    mut samp: *mut Sample,
    mut verbose: bool_0,
    mut index: libc::c_int,
) -> libc::c_int {
    let mut finetune: libc::c_int = 0;
    let mut x: libc::c_int = 0;
    memset(
        samp as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<Sample>() as libc::c_ulong,
    );
    (*samp).msl_index = 0xffff as libc::c_int as u16_0;
    x = 0 as libc::c_int;
    while x < 22 as libc::c_int {
        (*samp).name[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < 12 as libc::c_int {
        (*samp).filename[x as usize] = (*samp).name[x as usize];
        x += 1;
    }
    (*samp).sample_length = ((read8() as libc::c_int * 256 as libc::c_int + read8() as libc::c_int)
        * 2 as libc::c_int) as u32_0;
    finetune = read8() as libc::c_int;
    if finetune >= 8 as libc::c_int {
        finetune -= 16 as libc::c_int;
    }
    (*samp).default_volume = read8();
    (*samp).loop_start = ((read8() as libc::c_int * 256 as libc::c_int + read8() as libc::c_int)
        * 2 as libc::c_int) as u32_0;
    (*samp).loop_end = ((*samp).loop_start).wrapping_add(
        ((read8() as libc::c_int * 256 as libc::c_int + read8() as libc::c_int) * 2 as libc::c_int)
            as libc::c_uint,
    );
    (*samp).frequency = (8363.0f64 * pow(2.0f64, finetune as libc::c_double * (1.0f64 / 192.0f64)))
        as libc::c_int as u32_0;
    (*samp).global_volume = 64 as libc::c_int as u8_0;
    if ((*samp).loop_end).wrapping_sub((*samp).loop_start) <= 2 as libc::c_int as libc::c_uint {
        let ref mut fresh1 = (*samp).loop_end;
        *fresh1 = 0 as libc::c_int as u32_0;
        let ref mut fresh2 = (*samp).loop_start;
        *fresh2 = *fresh1;
        (*samp).loop_type = *fresh2 as u8_0;
    } else {
        (*samp).loop_type = 1 as libc::c_int as u8_0;
    }
    if verbose != 0 {
        if (*samp).sample_length != 0 as libc::c_int as libc::c_uint {
            printf(
                b" %-2i    %-5i  %-3s   %3i%%    %ihz  %-22s \n\0" as *const u8
                    as *const libc::c_char,
                index,
                (*samp).sample_length,
                if (*samp).loop_type as libc::c_int != 0 as libc::c_int {
                    b"Yes\0" as *const u8 as *const libc::c_char
                } else {
                    b"No\0" as *const u8 as *const libc::c_char
                },
                (*samp).default_volume as libc::c_int * 100 as libc::c_int / 64 as libc::c_int,
                (*samp).frequency,
                ((*samp).name).as_mut_ptr(),
            );
        }
    }
    return 0 as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Load_MOD(mut mod_0: *mut MAS_Module, mut verbose: bool_0) -> libc::c_int {
    let mut file_start: u32_0 = 0;
    let mut mod_channels: u32_0 = 0;
    let mut x: libc::c_int = 0;
    let mut npatterns: libc::c_int = 0;
    let mut sig: u32_0 = 0;
    let mut sigs: [libc::c_char; 5] = [0; 5];
    if verbose != 0 {
        printf(b"Loading MOD, \0" as *const u8 as *const libc::c_char);
    }
    memset(
        mod_0 as *mut libc::c_void,
        0 as libc::c_int,
        ::std::mem::size_of::<MAS_Module>() as libc::c_ulong,
    );
    file_start = file_tell_read() as u32_0;
    file_seek_read(0x438 as libc::c_int, 0 as libc::c_int);
    sig = read32();
    sigs[0 as libc::c_int as usize] = (sig & 0xff as libc::c_int as libc::c_uint) as libc::c_char;
    sigs[1 as libc::c_int as usize] =
        (sig >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as libc::c_char;
    sigs[2 as libc::c_int as usize] =
        (sig >> 16 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as libc::c_char;
    sigs[3 as libc::c_int as usize] = (sig >> 24 as libc::c_int) as libc::c_char;
    sigs[4 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
    match sig {
        1313358641 => {
            mod_channels = 1 as libc::c_int as u32_0;
        }
        1313358642 => {
            mod_channels = 2 as libc::c_int as u32_0;
        }
        1313358643 => {
            mod_channels = 3 as libc::c_int as u32_0;
        }
        776678989 | 1313358644 => {
            mod_channels = 4 as libc::c_int as u32_0;
        }
        1313358645 => {
            mod_channels = 5 as libc::c_int as u32_0;
        }
        1313358646 => {
            mod_channels = 6 as libc::c_int as u32_0;
        }
        1313358647 => {
            mod_channels = 7 as libc::c_int as u32_0;
        }
        1313358648 => {
            mod_channels = 8 as libc::c_int as u32_0;
        }
        1313358649 => {
            mod_channels = 9 as libc::c_int as u32_0;
        }
        _ => {
            if sig >> 16 as libc::c_int == '䡃' as i32 as libc::c_uint {
                let mut chn_number: [libc::c_char; 3] = [0; 3];
                chn_number[0 as libc::c_int as usize] =
                    (sig & 0xff as libc::c_int as libc::c_uint) as libc::c_char;
                chn_number[1 as libc::c_int as usize] =
                    (sig >> 8 as libc::c_int & 0xff as libc::c_int as libc::c_uint) as libc::c_char;
                chn_number[2 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
                mod_channels = atoi(chn_number.as_mut_ptr()) as u32_0;
                if mod_channels > 32 as libc::c_int as libc::c_uint {
                    return 0x5 as libc::c_int;
                }
            } else {
                return 0x1 as libc::c_int;
            }
        }
    }
    file_seek_read(file_start as libc::c_int, 0 as libc::c_int);
    x = 0 as libc::c_int;
    while x < 20 as libc::c_int {
        (*mod_0).title[x as usize] = read8() as libc::c_char;
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"\"%s\"\n\0" as *const u8 as *const libc::c_char,
            ((*mod_0).title).as_mut_ptr(),
        );
        printf(
            b"%i channels (%s)\n\0" as *const u8 as *const libc::c_char,
            mod_channels,
            sigs.as_mut_ptr(),
        );
    }
    x = 0 as libc::c_int;
    while x < 32 as libc::c_int {
        if x & 3 as libc::c_int != 1 as libc::c_int && x & 3 as libc::c_int != 2 as libc::c_int {
            (*mod_0).channel_panning[x as usize] =
                clamp_u8(128 as libc::c_int - PANNING_SEP / 2 as libc::c_int) as u8_0;
        } else {
            (*mod_0).channel_panning[x as usize] =
                clamp_u8(128 as libc::c_int + PANNING_SEP / 2 as libc::c_int) as u8_0;
        }
        (*mod_0).channel_volume[x as usize] = 64 as libc::c_int as u8_0;
        x += 1;
    }
    (*mod_0).freq_mode = 0 as libc::c_int as u8_0;
    (*mod_0).global_volume = 64 as libc::c_int as u8_0;
    (*mod_0).initial_speed = 6 as libc::c_int as u8_0;
    (*mod_0).initial_tempo = 125 as libc::c_int as u8_0;
    (*mod_0).inst_count = 0 as libc::c_int as u8_0;
    (*mod_0).inst_mode = 0 as libc::c_int as bool_0;
    let ref mut fresh3 = (*mod_0).instruments;
    *fresh3 = malloc(
        (31 as libc::c_int as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Instrument>() as libc::c_ulong),
    ) as *mut Instrument;
    (*mod_0).link_gxx = 0 as libc::c_int as bool_0;
    (*mod_0).old_effects = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    (*mod_0).restart_pos = 0 as libc::c_int as u8_0;
    (*mod_0).samp_count = 0 as libc::c_int as u8_0;
    let ref mut fresh4 = (*mod_0).samples;
    *fresh4 = malloc(
        (31 as libc::c_int as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Sample>() as libc::c_ulong),
    ) as *mut Sample;
    (*mod_0).stereo = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    (*mod_0).xm_mode = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    (*mod_0).old_mode = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(b"Loading Samples...\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b" INDEX LENGTH LOOP  VOLUME  MID-C   NAME                   \n\0" as *const u8
                as *const libc::c_char,
        );
    }
    x = 0 as libc::c_int;
    while x < 31 as libc::c_int {
        Create_MOD_Instrument(&mut *((*mod_0).instruments).offset(x as isize), x as u8_0);
        Load_MOD_Sample(&mut *((*mod_0).samples).offset(x as isize), verbose, x);
        x += 1;
    }
    (*mod_0).order_count = read8() as u16_0;
    (*mod_0).restart_pos = read8();
    if (*mod_0).restart_pos as libc::c_int >= 127 as libc::c_int {
        (*mod_0).restart_pos = 0 as libc::c_int as u8_0;
    }
    npatterns = 0 as libc::c_int;
    x = 0 as libc::c_int;
    while x < 128 as libc::c_int {
        (*mod_0).orders[x as usize] = read8();
        if (*mod_0).orders[x as usize] as libc::c_int >= npatterns {
            npatterns = (*mod_0).orders[x as usize] as libc::c_int + 1 as libc::c_int;
        }
        x += 1;
    }
    read32();
    (*mod_0).patt_count = npatterns as u8_0;
    let ref mut fresh5 = (*mod_0).patterns;
    *fresh5 = malloc(
        ((*mod_0).patt_count as libc::c_ulong)
            .wrapping_mul(::std::mem::size_of::<Pattern>() as libc::c_ulong),
    ) as *mut Pattern;
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
        printf(
            b"Sequence has %i entries.\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).order_count as libc::c_int,
        );
        printf(
            b"Module has %i pattern%s.\n\0" as *const u8 as *const libc::c_char,
            (*mod_0).patt_count as libc::c_int,
            if (*mod_0).patt_count as libc::c_int == 1 as libc::c_int {
                b"\0" as *const u8 as *const libc::c_char
            } else {
                b"s\0" as *const u8 as *const libc::c_char
            },
        );
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
        Load_MOD_Pattern(
            &mut *((*mod_0).patterns).offset(x as isize),
            mod_channels as u8_0,
            &mut (*mod_0).inst_count,
        );
        x += 1;
    }
    if verbose != 0 {
        printf(b"\n\0" as *const u8 as *const libc::c_char);
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    if verbose != 0 {
        printf(b"Loading Sample Data...\n\0" as *const u8 as *const libc::c_char);
    }
    (*mod_0).samp_count = (*mod_0).inst_count;
    x = 0 as libc::c_int;
    while x < 31 as libc::c_int {
        Load_MOD_SampleData(&mut *((*mod_0).samples).offset(x as isize));
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"--------------------------------------------\n\0" as *const u8 as *const libc::c_char,
        );
    }
    return 0 as libc::c_int;
}
