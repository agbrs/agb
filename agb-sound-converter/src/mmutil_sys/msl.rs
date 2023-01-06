use ::libc::{self, fclose, fopen, fprintf, free, malloc, printf, strlen, toupper, FILE};

extern "C" {
    fn write32(p_v: u32_0);
    fn fseek(__stream: *mut FILE, __off: libc::c_long, __whence: libc::c_int) -> libc::c_int;
    fn ftell(__stream: *mut FILE) -> libc::c_long;
    fn file_size(filename: *mut libc::c_char) -> libc::c_int;
    fn file_open_read(filename: *mut libc::c_char) -> libc::c_int;
    fn file_open_write(filename: *mut libc::c_char) -> libc::c_int;
    fn file_open_write_end(filename: *mut libc::c_char) -> libc::c_int;
    fn file_close_read();
    fn file_close_write();
    fn read8() -> u8_0;
    fn read32() -> u32_0;
    fn write8(p_v: u8_0);
    fn write16(p_v: u16_0);
    fn read32f(p_fin: *mut FILE) -> u32_0;
    fn skip8f(count: u32_0, p_file: *mut FILE);
    fn file_delete(filename: *mut libc::c_char);
    fn read16f(p_fin: *mut FILE) -> u16_0;
    fn align32();
    fn read8f(p_fin: *mut FILE) -> u8_0;
    fn file_tell_write() -> libc::c_int;
    fn file_seek_write(offset: libc::c_int, mode: libc::c_int) -> libc::c_int;
    fn Write_SampleData(samp: *mut Sample);
    fn Write_MAS(mod_0: *mut MAS_Module, verbose: bool_0, msl_dep: bool_0) -> libc::c_int;
    fn Delete_Module(mod_0: *mut MAS_Module);
    fn Load_MOD(mod_0: *mut MAS_Module, verbose: bool_0) -> libc::c_int;
    fn Load_S3M(mod_0: *mut MAS_Module, verbose: bool_0) -> libc::c_int;
    fn Load_XM(mod_0: *mut MAS_Module, verbose: bool_0) -> libc::c_int;
    fn Load_IT(itm: *mut MAS_Module, verbose: bool_0) -> libc::c_int;
    fn Load_WAV(samp: *mut Sample, verbose: bool_0, fix: bool_0) -> libc::c_int;
    fn get_ext(filename: *mut libc::c_char) -> libc::c_int;
    fn sample_dsformat(samp: *mut Sample) -> u8_0;
    static mut target_system: libc::c_int;
}
pub type size_t = libc::c_ulong;
pub type __int32_t = libc::c_int;
pub type __off_t = libc::c_long;
pub type __off64_t = libc::c_long;
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
pub static mut F_SCRIPT: *mut FILE = 0 as *const FILE as *mut FILE;
#[no_mangle]
pub static mut F_SAMP: *mut FILE = 0 as *const FILE as *mut FILE;
#[no_mangle]
pub static mut F_SONG: *mut FILE = 0 as *const FILE as *mut FILE;
#[no_mangle]
pub static mut F_HEADER: *mut FILE = 0 as *const FILE as *mut FILE;
#[no_mangle]
pub static mut MSL_NSAMPS: u16_0 = 0;
#[no_mangle]
pub static mut MSL_NSONGS: u16_0 = 0;
#[no_mangle]
pub static mut str_msl: [libc::c_char; 256] = [0; 256];
#[no_mangle]
pub unsafe extern "C" fn MSL_Erase() {
    MSL_NSAMPS = 0 as libc::c_int as u16_0;
    MSL_NSONGS = 0 as libc::c_int as u16_0;
    file_delete(b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
    file_delete(b"songDJ34957FAI.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
}
#[no_mangle]
pub unsafe extern "C" fn MSL_AddSample(mut samp: *mut Sample) -> u16_0 {
    let mut sample_length: u32_0 = 0;
    let mut x: u32_0 = 0;
    file_open_write_end(
        b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    );
    sample_length = (*samp).sample_length;
    write32(
        (if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
            sample_length.wrapping_mul(2 as libc::c_int as libc::c_uint)
        } else {
            sample_length
        })
        .wrapping_add(
            (12 as libc::c_int
                + (if target_system == 1 as libc::c_int {
                    4 as libc::c_int
                } else {
                    0 as libc::c_int
                })) as libc::c_uint,
        )
        .wrapping_add(4 as libc::c_int as libc::c_uint),
    );
    write8(
        (if target_system == 0 as libc::c_int {
            1 as libc::c_int
        } else {
            2 as libc::c_int
        }) as u8_0,
    );
    write8(0x18 as libc::c_int as u8_0);
    write8(
        (if (*samp).filename[0 as libc::c_int as usize] as libc::c_int == '#' as i32 {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        }) as u8_0,
    );
    write8(0xba as libc::c_int as u8_0);
    Write_SampleData(samp);
    file_close_write();
    MSL_NSAMPS = MSL_NSAMPS.wrapping_add(1);
    return (MSL_NSAMPS as libc::c_int - 1 as libc::c_int) as u16_0;
}
#[no_mangle]
pub unsafe extern "C" fn MSL_AddSampleC(mut samp: *mut Sample) -> u16_0 {
    let mut st: u32_0 = 0;
    let mut samp_len: u32_0 = 0;
    let mut samp_llen: u32_0 = 0;
    let mut sformat: u8_0 = 0;
    let mut target_sformat: u8_0 = 0;
    let mut h_filesize: u32_0 = 0;
    let mut samp_id: libc::c_int = 0;
    let mut samp_match: bool_0 = 0;
    let mut fsize =
        file_size(b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
    if fsize == 0 as libc::c_int {
        return MSL_AddSample(samp);
    }
    F_SAMP = fopen(
        b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char,
        b"rb\0" as *const u8 as *const libc::c_char,
    );
    fseek(F_SAMP, 0 as libc::c_int as libc::c_long, 0 as libc::c_int);
    samp_id = 0 as libc::c_int;
    while ftell(F_SAMP) < fsize as libc::c_long {
        h_filesize = read32f(F_SAMP);
        read32f(F_SAMP);
        samp_len = read32f(F_SAMP);
        samp_llen = read32f(F_SAMP);
        sformat = read8f(F_SAMP);
        skip8f(3 as libc::c_int as u32_0, F_SAMP);
        if target_system == 1 as libc::c_int {
            target_sformat = sample_dsformat(samp);
            skip8f(4 as libc::c_int as u32_0, F_SAMP);
        } else {
            target_sformat = 0 as libc::c_int as u8_0;
        }
        samp_match = (0 as libc::c_int == 0) as libc::c_int as bool_0;
        if (*samp).sample_length == samp_len
            && (if (*samp).loop_type as libc::c_int != 0 {
                ((*samp).loop_end).wrapping_sub((*samp).loop_start)
            } else {
                0xffffffff as libc::c_uint
            }) == samp_llen
            && sformat as libc::c_int == target_sformat as libc::c_int
        {
            if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
                st = 0 as libc::c_int as u32_0;
                while st < samp_len {
                    if read16f(F_SAMP) as libc::c_int
                        != *((*samp).data as *mut u16_0).offset(st as isize) as libc::c_int
                    {
                        samp_match = 0 as libc::c_int as bool_0;
                        break;
                    } else {
                        st = st.wrapping_add(1);
                    }
                }
            } else {
                st = 0 as libc::c_int as u32_0;
                while st < samp_len {
                    if read8f(F_SAMP) as libc::c_int
                        != *((*samp).data as *mut u8_0).offset(st as isize) as libc::c_int
                    {
                        samp_match = 0 as libc::c_int as bool_0;
                        break;
                    } else {
                        st = st.wrapping_add(1);
                    }
                }
            }
            if samp_match != 0 {
                fclose(F_SAMP);
                return samp_id as u16_0;
            } else {
                skip8f(
                    h_filesize
                        .wrapping_sub(
                            (12 as libc::c_int
                                + (if target_system == 1 as libc::c_int {
                                    4 as libc::c_int
                                } else {
                                    0 as libc::c_int
                                })) as libc::c_uint,
                        )
                        .wrapping_sub(st.wrapping_add(1 as libc::c_int as libc::c_uint)),
                    F_SAMP,
                );
            }
        } else {
            skip8f(
                h_filesize.wrapping_sub(
                    (12 as libc::c_int
                        + (if target_system == 1 as libc::c_int {
                            4 as libc::c_int
                        } else {
                            0 as libc::c_int
                        })) as libc::c_uint,
                ),
                F_SAMP,
            );
        }
        samp_id += 1;
    }
    fclose(F_SAMP);
    return MSL_AddSample(samp);
}
#[no_mangle]
pub unsafe extern "C" fn MSL_AddModule(mut mod_0: *mut MAS_Module) -> u16_0 {
    let mut x: libc::c_int = 0;
    let mut samp_id: libc::c_int = 0;
    x = 0 as libc::c_int;
    while x < (*mod_0).samp_count as libc::c_int {
        samp_id = MSL_AddSampleC(&mut *((*mod_0).samples).offset(x as isize)) as libc::c_int;
        if (*((*mod_0).samples).offset(x as isize)).filename[0 as libc::c_int as usize]
            as libc::c_int
            == '#' as i32
        {
            MSL_PrintDefinition(
                ((*((*mod_0).samples).offset(x as isize)).filename)
                    .as_mut_ptr()
                    .offset(1 as libc::c_int as isize),
                samp_id as u16_0,
                b"SFX_\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            );
        }
        (*((*mod_0).samples).offset(x as isize)).msl_index = samp_id as u16_0;
        x += 1;
    }
    file_open_write_end(
        b"songDJ34957FAI.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    );
    Write_MAS(
        mod_0,
        0 as libc::c_int as bool_0,
        (0 as libc::c_int == 0) as libc::c_int as bool_0,
    );
    file_close_write();
    MSL_NSONGS = MSL_NSONGS.wrapping_add(1);
    return (MSL_NSONGS as libc::c_int - 1 as libc::c_int) as u16_0;
}
#[no_mangle]
pub unsafe extern "C" fn MSL_Export(mut filename: *mut libc::c_char) {
    let mut x: u32_0 = 0;
    let mut y: u32_0 = 0;
    let mut file_size_0: u32_0 = 0;
    let mut parap_samp = 0 as *mut u32_0;
    let mut parap_song = 0 as *mut u32_0;
    file_open_write(filename);
    write16(MSL_NSAMPS);
    write16(MSL_NSONGS);
    write8('*' as i32 as u8_0);
    write8('m' as i32 as u8_0);
    write8('a' as i32 as u8_0);
    write8('x' as i32 as u8_0);
    write8('m' as i32 as u8_0);
    write8('o' as i32 as u8_0);
    write8('d' as i32 as u8_0);
    write8('*' as i32 as u8_0);
    parap_samp = malloc(
        (MSL_NSAMPS as libc::c_ulong).wrapping_mul(::std::mem::size_of::<u32_0>() as libc::c_ulong)
            as libc::size_t,
    ) as *mut u32_0;
    parap_song = malloc(
        (MSL_NSONGS as libc::c_ulong).wrapping_mul(::std::mem::size_of::<u32_0>() as libc::c_ulong)
            as libc::size_t,
    ) as *mut u32_0;
    x = 0 as libc::c_int as u32_0;
    while x < MSL_NSAMPS as libc::c_uint {
        write32(0xaaaaaaaa as libc::c_uint);
        x = x.wrapping_add(1);
    }
    x = 0 as libc::c_int as u32_0;
    while x < MSL_NSONGS as libc::c_uint {
        write32(0xaaaaaaaa as libc::c_uint);
        x = x.wrapping_add(1);
    }
    file_open_read(
        b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    );
    x = 0 as libc::c_int as u32_0;
    while x < MSL_NSAMPS as libc::c_uint {
        align32();
        *parap_samp.offset(x as isize) = file_tell_write() as u32_0;
        file_size_0 = read32();
        write32(file_size_0);
        y = 0 as libc::c_int as u32_0;
        while y < file_size_0.wrapping_add(4 as libc::c_int as libc::c_uint) {
            write8(read8());
            y = y.wrapping_add(1);
        }
        x = x.wrapping_add(1);
    }
    file_close_read();
    file_open_read(
        b"songDJ34957FAI.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    );
    x = 0 as libc::c_int as u32_0;
    while x < MSL_NSONGS as libc::c_uint {
        align32();
        *parap_song.offset(x as isize) = file_tell_write() as u32_0;
        file_size_0 = read32();
        write32(file_size_0);
        y = 0 as libc::c_int as u32_0;
        while y < file_size_0.wrapping_add(4 as libc::c_int as libc::c_uint) {
            write8(read8());
            y = y.wrapping_add(1);
        }
        x = x.wrapping_add(1);
    }
    file_close_read();
    file_seek_write(0xc as libc::c_int, 0 as libc::c_int);
    x = 0 as libc::c_int as u32_0;
    while x < MSL_NSAMPS as libc::c_uint {
        write32(*parap_samp.offset(x as isize));
        x = x.wrapping_add(1);
    }
    x = 0 as libc::c_int as u32_0;
    while x < MSL_NSONGS as libc::c_uint {
        write32(*parap_song.offset(x as isize));
        x = x.wrapping_add(1);
    }
    file_close_write();
    if !parap_samp.is_null() {
        free(parap_samp as *mut libc::c_void);
    }
    if !parap_song.is_null() {
        free(parap_song as *mut libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn MSL_PrintDefinition(
    mut filename: *mut libc::c_char,
    mut id: u16_0,
    mut prefix: *mut libc::c_char,
) {
    let mut newtitle: [libc::c_char; 64] = [0; 64];
    let mut x: libc::c_int = 0;
    let mut s = 0 as libc::c_int;
    if *filename.offset(0 as libc::c_int as isize) as libc::c_int == 0 as libc::c_int {
        return;
    }
    x = 0 as libc::c_int;
    while x < strlen(filename) as libc::c_int {
        if *filename.offset(x as isize) as libc::c_int == '\\' as i32
            || *filename.offset(x as isize) as libc::c_int == '/' as i32
        {
            s = x + 1 as libc::c_int;
        }
        x += 1;
    }
    x = s;
    while x < strlen(filename) as libc::c_int {
        if !(*filename.offset(x as isize) as libc::c_int != '.' as i32) {
            break;
        }
        newtitle[(x - s) as usize] =
            toupper(*filename.offset(x as isize) as libc::c_int) as libc::c_char;
        if newtitle[(x - s) as usize] as libc::c_int >= ' ' as i32
            && newtitle[(x - s) as usize] as libc::c_int <= '/' as i32
        {
            newtitle[(x - s) as usize] = '_' as i32 as libc::c_char;
        }
        if newtitle[(x - s) as usize] as libc::c_int >= ':' as i32
            && newtitle[(x - s) as usize] as libc::c_int <= '@' as i32
        {
            newtitle[(x - s) as usize] = '_' as i32 as libc::c_char;
        }
        if newtitle[(x - s) as usize] as libc::c_int >= '[' as i32
            && newtitle[(x - s) as usize] as libc::c_int <= '`' as i32
        {
            newtitle[(x - s) as usize] = '_' as i32 as libc::c_char;
        }
        if newtitle[(x - s) as usize] as libc::c_int >= '{' as i32 {
            newtitle[(x - s) as usize] = '_' as i32 as libc::c_char;
        }
        x += 1;
    }
    newtitle[(x - s) as usize] = 0 as libc::c_int as libc::c_char;
    if !F_HEADER.is_null() {
        fprintf(
            F_HEADER,
            b"#define %s%s\t%i\r\n\0" as *const u8 as *const libc::c_char,
            prefix,
            newtitle.as_mut_ptr(),
            id as libc::c_int,
        );
    }
}
#[no_mangle]
pub unsafe extern "C" fn MSL_LoadFile(mut filename: *mut libc::c_char, mut verbose: bool_0) {
    let mut wav = Sample {
        parapointer: 0,
        global_volume: 0,
        default_volume: 0,
        default_panning: 0,
        sample_length: 0,
        loop_start: 0,
        loop_end: 0,
        loop_type: 0,
        frequency: 0,
        data: 0 as *mut libc::c_void,
        vibtype: 0,
        vibdepth: 0,
        vibspeed: 0,
        vibrate: 0,
        msl_index: 0,
        rsamp_index: 0,
        format: 0,
        datapointer: 0,
        it_compression: 0,
        name: [0; 32],
        filename: [0; 12],
    };
    let mut mod_0 = MAS_Module {
        title: [0; 32],
        order_count: 0,
        inst_count: 0,
        samp_count: 0,
        patt_count: 0,
        restart_pos: 0,
        stereo: 0,
        inst_mode: 0,
        freq_mode: 0,
        old_effects: 0,
        link_gxx: 0,
        xm_mode: 0,
        old_mode: 0,
        global_volume: 0,
        initial_speed: 0,
        initial_tempo: 0,
        channel_volume: [0; 32],
        channel_panning: [0; 32],
        orders: [0; 256],
        instruments: 0 as *mut Instrument,
        samples: 0 as *mut Sample,
        patterns: 0 as *mut Pattern,
    };
    let mut f_ext: libc::c_int = 0;
    if file_open_read(filename) != 0 {
        printf(
            b"Cannot open %s for reading! Skipping.\n\0" as *const u8 as *const libc::c_char,
            filename,
        );
        return;
    }
    f_ext = get_ext(filename);
    match f_ext {
        0 => {
            Load_MOD(&mut mod_0, verbose);
            MSL_PrintDefinition(
                filename,
                MSL_AddModule(&mut mod_0),
                b"MOD_\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            );
            Delete_Module(&mut mod_0);
        }
        1 => {
            Load_S3M(&mut mod_0, verbose);
            MSL_PrintDefinition(
                filename,
                MSL_AddModule(&mut mod_0),
                b"MOD_\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            );
            Delete_Module(&mut mod_0);
        }
        2 => {
            Load_XM(&mut mod_0, verbose);
            MSL_PrintDefinition(
                filename,
                MSL_AddModule(&mut mod_0),
                b"MOD_\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            );
            Delete_Module(&mut mod_0);
        }
        3 => {
            Load_IT(&mut mod_0, verbose);
            MSL_PrintDefinition(
                filename,
                MSL_AddModule(&mut mod_0),
                b"MOD_\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            );
            Delete_Module(&mut mod_0);
        }
        4 => {
            Load_WAV(
                &mut wav,
                verbose,
                (0 as libc::c_int == 0) as libc::c_int as bool_0,
            );
            wav.filename[0 as libc::c_int as usize] = '#' as i32 as libc::c_char;
            MSL_PrintDefinition(
                filename,
                MSL_AddSample(&mut wav),
                b"SFX_\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
            );
            free(wav.data);
        }
        _ => {
            printf(
                b"Unknown file %s...\n\0" as *const u8 as *const libc::c_char,
                filename,
            );
        }
    }
    file_close_read();
}
#[no_mangle]
pub unsafe extern "C" fn MSL_Create(
    mut argv: *mut *mut libc::c_char,
    mut argc: libc::c_int,
    mut output: *mut libc::c_char,
    mut header: *mut libc::c_char,
    mut verbose: bool_0,
) -> libc::c_int {
    let mut x: libc::c_int = 0;
    MSL_Erase();
    str_msl[0 as libc::c_int as usize] = 0 as libc::c_int as libc::c_char;
    F_HEADER = 0 as *mut FILE;
    if !header.is_null() {
        F_HEADER = fopen(header, b"wb\0" as *const u8 as *const libc::c_char);
    }
    file_open_write(
        b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    );
    file_close_write();
    file_open_write(
        b"songDJ34957FAI.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char,
    );
    file_close_write();
    x = 1 as libc::c_int;
    while x < argc {
        if !(*(*argv.offset(x as isize)).offset(0 as libc::c_int as isize) as libc::c_int
            == '-' as i32)
        {
            MSL_LoadFile(*argv.offset(x as isize), verbose);
        }
        x += 1;
    }
    MSL_Export(output);
    if !F_HEADER.is_null() {
        fprintf(
            F_HEADER,
            b"#define MSL_NSONGS\t%i\r\n\0" as *const u8 as *const libc::c_char,
            MSL_NSONGS as libc::c_int,
        );
        fprintf(
            F_HEADER,
            b"#define MSL_NSAMPS\t%i\r\n\0" as *const u8 as *const libc::c_char,
            MSL_NSAMPS as libc::c_int,
        );
        fprintf(
            F_HEADER,
            b"#define MSL_BANKSIZE\t%i\r\n\0" as *const u8 as *const libc::c_char,
            MSL_NSAMPS as libc::c_int + MSL_NSONGS as libc::c_int,
        );
        fclose(F_HEADER);
        F_HEADER = 0 as *mut FILE;
    }
    file_delete(b"sampJ328G54AU3.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
    file_delete(b"songDJ34957FAI.tmp\0" as *const u8 as *const libc::c_char as *mut libc::c_char);
    return 0 as libc::c_int;
}
