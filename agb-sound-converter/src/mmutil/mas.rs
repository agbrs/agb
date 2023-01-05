use ::libc;
extern "C" {
    fn free(_: *mut libc::c_void);
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn file_get_byte_count() -> libc::c_int;
    fn file_tell_write() -> libc::c_int;
    fn file_seek_write(offset: libc::c_int, mode: libc::c_int) -> libc::c_int;
    fn write8(p_v: u8_0);
    fn write16(p_v: u16_0);
    fn write32(p_v: u32_0);
    fn align32();
    fn sample_dsformat(samp: *mut Sample) -> u8_0;
    fn sample_dsreptype(samp: *mut Sample) -> u8_0;
    static mut target_system: libc::c_int;
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
pub static mut MAS_OFFSET: u32_0 = 0;
#[no_mangle]
pub static mut MAS_FILESIZE: u32_0 = 0;
unsafe extern "C" fn CalcEnvelopeSize(mut env: *mut Instrument_Envelope) -> libc::c_int {
    return (*env).node_count as libc::c_int * 4 as libc::c_int + 8 as libc::c_int;
}
unsafe extern "C" fn CalcInstrumentSize(mut instr: *mut Instrument) -> libc::c_int {
    let mut size: libc::c_int = 0;
    size = 12 as libc::c_int;
    if (*instr).env_flags as libc::c_int & 1 as libc::c_int != 0 {
        size += CalcEnvelopeSize(&mut (*instr).envelope_volume);
    }
    if (*instr).env_flags as libc::c_int & 2 as libc::c_int != 0 {
        size += CalcEnvelopeSize(&mut (*instr).envelope_pan);
    }
    if (*instr).env_flags as libc::c_int & 4 as libc::c_int != 0 {
        size += CalcEnvelopeSize(&mut (*instr).envelope_pitch);
    }
    return size;
}
#[no_mangle]
pub unsafe extern "C" fn Write_Instrument_Envelope(mut env: *mut Instrument_Envelope) {
    let mut x: libc::c_int = 0;
    write8(((*env).node_count as libc::c_int * 4 as libc::c_int + 8 as libc::c_int) as u8_0);
    write8((*env).loop_start);
    write8((*env).loop_end);
    write8((*env).sus_start);
    write8((*env).sus_end);
    write8((*env).node_count);
    write8((*env).env_filter);
    write8(0xba as libc::c_int as u8_0);
    if (*env).node_count as libc::c_int > 1 as libc::c_int {
        let mut delta: libc::c_int = 0;
        let mut base: libc::c_int = 0;
        let mut range: libc::c_int = 0;
        x = 0 as libc::c_int;
        while x < (*env).node_count as libc::c_int {
            base = (*env).node_y[x as usize] as libc::c_int;
            if x != (*env).node_count as libc::c_int - 1 as libc::c_int {
                range = (*env).node_x[(x + 1 as libc::c_int) as usize] as libc::c_int
                    - (*env).node_x[x as usize] as libc::c_int;
                if range > 511 as libc::c_int {
                    range = 511 as libc::c_int;
                }
                if range < 1 as libc::c_int {
                    range = 1 as libc::c_int;
                }
                delta = (((*env).node_y[(x + 1 as libc::c_int) as usize] as libc::c_int - base)
                    * 512 as libc::c_int
                    + range / 2 as libc::c_int)
                    / range;
                if delta > 32767 as libc::c_int {
                    delta = 32767 as libc::c_int;
                }
                if delta < -(32768 as libc::c_int) {
                    delta = -(32768 as libc::c_int);
                }
                while base + (delta * range >> 9 as libc::c_int) > 64 as libc::c_int {
                    delta -= 1;
                }
                while base + (delta * range >> 9 as libc::c_int) < 0 as libc::c_int {
                    delta += 1;
                }
            } else {
                range = 0 as libc::c_int;
                delta = 0 as libc::c_int;
            }
            write16(delta as u16_0);
            write16((base | range << 7 as libc::c_int) as u16_0);
            x += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Write_Instrument(mut inst: *mut Instrument) {
    let mut y: libc::c_int = 0;
    let mut full_notemap: libc::c_int = 0;
    let mut first_notemap_samp: libc::c_int = 0;
    align32();
    (*inst).parapointer = (file_tell_write() as libc::c_uint).wrapping_sub(MAS_OFFSET);
    write8((*inst).global_volume);
    write8((*inst).fadeout as u8_0);
    write8((*inst).random_volume);
    write8((*inst).dct);
    write8((*inst).nna);
    write8((*inst).env_flags);
    write8((*inst).setpan);
    write8((*inst).dca);
    full_notemap = 0 as libc::c_int;
    first_notemap_samp =
        (*inst).notemap[0 as libc::c_int as usize] as libc::c_int >> 8 as libc::c_int;
    y = 0 as libc::c_int;
    while y < 120 as libc::c_int {
        if (*inst).notemap[y as usize] as libc::c_int & 0xff as libc::c_int != y
            || (*inst).notemap[y as usize] as libc::c_int >> 8 as libc::c_int != first_notemap_samp
        {
            full_notemap = 1 as libc::c_int;
            break;
        } else {
            y += 1;
        }
    }
    if full_notemap != 0 {
        write16(CalcInstrumentSize(inst) as u16_0);
    } else {
        write16((0x8000 as libc::c_int | first_notemap_samp) as u16_0);
    }
    write16(0 as libc::c_int as u16_0);
    if (*inst).env_flags as libc::c_int & 1 as libc::c_int != 0 {
        Write_Instrument_Envelope(&mut (*inst).envelope_volume);
    }
    if (*inst).env_flags as libc::c_int & 2 as libc::c_int != 0 {
        Write_Instrument_Envelope(&mut (*inst).envelope_pan);
    }
    if (*inst).env_flags as libc::c_int & 4 as libc::c_int != 0 {
        Write_Instrument_Envelope(&mut (*inst).envelope_pitch);
    }
    if full_notemap != 0 {
        y = 0 as libc::c_int;
        while y < 120 as libc::c_int {
            write16((*inst).notemap[y as usize]);
            y += 1;
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn Write_SampleData(mut samp: *mut Sample) {
    let mut x: u32_0 = 0;
    let mut sample_length = (*samp).sample_length;
    let mut sample_looplen = ((*samp).loop_end).wrapping_sub((*samp).loop_start);
    if target_system == 0 as libc::c_int {
        write32(sample_length);
        write32(if (*samp).loop_type as libc::c_int != 0 {
            sample_looplen
        } else {
            0xffffffff as libc::c_uint
        });
        write8(0 as libc::c_int as u8_0);
        write8(0xba as libc::c_int as u8_0);
        write16(
            ((*samp).frequency)
                .wrapping_mul(1024 as libc::c_int as libc::c_uint)
                .wrapping_add((15768 as libc::c_int / 2 as libc::c_int) as libc::c_uint)
                .wrapping_div(15768 as libc::c_int as libc::c_uint) as u16_0,
        );
    } else {
        if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
            if (*samp).loop_type != 0 {
                write32(((*samp).loop_start).wrapping_div(2 as libc::c_int as libc::c_uint));
                write32(
                    ((*samp).loop_end)
                        .wrapping_sub((*samp).loop_start)
                        .wrapping_div(2 as libc::c_int as libc::c_uint),
                );
            } else {
                write32(0 as libc::c_int as u32_0);
                write32(sample_length.wrapping_div(2 as libc::c_int as libc::c_uint));
            }
        } else if (*samp).loop_type != 0 {
            write32(((*samp).loop_start).wrapping_div(4 as libc::c_int as libc::c_uint));
            write32(
                ((*samp).loop_end)
                    .wrapping_sub((*samp).loop_start)
                    .wrapping_div(4 as libc::c_int as libc::c_uint),
            );
        } else {
            write32(0 as libc::c_int as u32_0);
            write32(sample_length.wrapping_div(4 as libc::c_int as libc::c_uint));
        }
        write8(sample_dsformat(samp));
        write8(sample_dsreptype(samp));
        write16(
            ((*samp).frequency)
                .wrapping_mul(1024 as libc::c_int as libc::c_uint)
                .wrapping_add((32768 as libc::c_int / 2 as libc::c_int) as libc::c_uint)
                .wrapping_div(32768 as libc::c_int as libc::c_uint) as u16_0,
        );
        write32(0 as libc::c_int as u32_0);
    }
    if (*samp).format as libc::c_int & 0x1 as libc::c_int != 0 {
        x = 0 as libc::c_int as u32_0;
        while x < sample_length {
            write16(*((*samp).data as *mut u16_0).offset(x as isize));
            x = x.wrapping_add(1);
        }
        if (*samp).loop_type as libc::c_int != 0
            && sample_length >= ((*samp).loop_start).wrapping_add(2 as libc::c_int as libc::c_uint)
        {
            write16(*((*samp).data as *mut u16_0).offset((*samp).loop_start as isize));
            write16(*((*samp).data as *mut u16_0).offset(
                ((*samp).loop_start).wrapping_add(1 as libc::c_int as libc::c_uint) as isize,
            ));
        } else {
            write16(0 as libc::c_int as u16_0);
            write16(0 as libc::c_int as u16_0);
        }
    } else {
        x = 0 as libc::c_int as u32_0;
        while x < sample_length {
            write8(*((*samp).data as *mut u8_0).offset(x as isize));
            x = x.wrapping_add(1);
        }
        if (*samp).loop_type as libc::c_int != 0
            && sample_length >= ((*samp).loop_start).wrapping_add(4 as libc::c_int as libc::c_uint)
        {
            write8(*((*samp).data as *mut u8_0).offset((*samp).loop_start as isize));
            write8(*((*samp).data as *mut u8_0).offset(
                ((*samp).loop_start).wrapping_add(1 as libc::c_int as libc::c_uint) as isize,
            ));
            write8(*((*samp).data as *mut u8_0).offset(
                ((*samp).loop_start).wrapping_add(2 as libc::c_int as libc::c_uint) as isize,
            ));
            write8(*((*samp).data as *mut u8_0).offset(
                ((*samp).loop_start).wrapping_add(3 as libc::c_int as libc::c_uint) as isize,
            ));
        } else {
            x = 0 as libc::c_int as u32_0;
            while x < 4 as libc::c_int as libc::c_uint {
                write8(
                    (if target_system == 0 as libc::c_int {
                        128 as libc::c_int
                    } else {
                        0 as libc::c_int
                    }) as u8_0,
                );
                x = x.wrapping_add(1);
            }
        }
    };
}
#[no_mangle]
pub unsafe extern "C" fn Write_Sample(mut samp: *mut Sample) {
    align32();
    (*samp).parapointer = (file_tell_write() as libc::c_uint).wrapping_sub(MAS_OFFSET);
    write8((*samp).default_volume);
    write8((*samp).default_panning);
    write16(((*samp).frequency).wrapping_div(4 as libc::c_int as libc::c_uint) as u16_0);
    write8((*samp).vibtype);
    write8((*samp).vibdepth);
    write8((*samp).vibspeed);
    write8((*samp).global_volume);
    write16((*samp).vibrate as u16_0);
    write16((*samp).msl_index);
    if (*samp).msl_index as libc::c_int == 0xffff as libc::c_int {
        Write_SampleData(samp);
    }
}
#[no_mangle]
pub unsafe extern "C" fn Write_Pattern(mut patt: *mut Pattern, mut xm_vol: bool_0) {
    let mut row: libc::c_int = 0;
    let mut col: libc::c_int = 0;
    let mut last_mask: [u16_0; 32] = [0; 32];
    let mut last_note: [u16_0; 32] = [0; 32];
    let mut last_inst: [u16_0; 32] = [0; 32];
    let mut last_vol: [u16_0; 32] = [0; 32];
    let mut last_fx: [u16_0; 32] = [0; 32];
    let mut last_param: [u16_0; 32] = [0; 32];
    let mut chanvar: u8_0 = 0;
    let mut maskvar: u8_0 = 0;
    let mut emptyvol: u8_0 = 0;
    let mut pe = 0 as *mut PatternEntry;
    (*patt).parapointer = (file_tell_write() as libc::c_uint).wrapping_sub(MAS_OFFSET);
    write8(((*patt).nrows as libc::c_int - 1 as libc::c_int) as u8_0);
    (*patt).cmarks[0 as libc::c_int as usize] = (0 as libc::c_int == 0) as libc::c_int as bool_0;
    emptyvol = (if xm_vol as libc::c_int != 0 {
        0 as libc::c_int
    } else {
        255 as libc::c_int
    }) as u8_0;
    row = 0 as libc::c_int;
    while row < (*patt).nrows as libc::c_int {
        if (*patt).cmarks[row as usize] != 0 {
            col = 0 as libc::c_int;
            while col < 32 as libc::c_int {
                last_mask[col as usize] = 256 as libc::c_int as u16_0;
                last_note[col as usize] = 256 as libc::c_int as u16_0;
                last_inst[col as usize] = 256 as libc::c_int as u16_0;
                last_vol[col as usize] = 256 as libc::c_int as u16_0;
                last_fx[col as usize] = 256 as libc::c_int as u16_0;
                last_param[col as usize] = 256 as libc::c_int as u16_0;
                col += 1;
            }
        }
        col = 0 as libc::c_int;
        while col < 32 as libc::c_int {
            pe = &mut *((*patt).data)
                .as_mut_ptr()
                .offset((row * 32 as libc::c_int + col) as isize)
                as *mut PatternEntry;
            if (*pe).note as libc::c_int != 250 as libc::c_int
                || (*pe).inst as libc::c_int != 0 as libc::c_int
                || (*pe).vol as libc::c_int != emptyvol as libc::c_int
                || (*pe).fx as libc::c_int != 0 as libc::c_int
                || (*pe).param as libc::c_int != 0 as libc::c_int
            {
                maskvar = 0 as libc::c_int as u8_0;
                chanvar = (col + 1 as libc::c_int) as u8_0;
                if (*pe).note as libc::c_int != 250 as libc::c_int {
                    maskvar =
                        (maskvar as libc::c_int | (1 as libc::c_int | 16 as libc::c_int)) as u8_0;
                }
                if (*pe).inst as libc::c_int != 0 as libc::c_int {
                    maskvar =
                        (maskvar as libc::c_int | (2 as libc::c_int | 32 as libc::c_int)) as u8_0;
                }
                if (*pe).note as libc::c_int > 250 as libc::c_int {
                    maskvar =
                        (maskvar as libc::c_int & !(16 as libc::c_int | 32 as libc::c_int)) as u8_0;
                }
                if (*pe).vol as libc::c_int != emptyvol as libc::c_int {
                    maskvar =
                        (maskvar as libc::c_int | (4 as libc::c_int | 64 as libc::c_int)) as u8_0;
                }
                if (*pe).fx as libc::c_int != 0 as libc::c_int
                    || (*pe).param as libc::c_int != 0 as libc::c_int
                {
                    maskvar =
                        (maskvar as libc::c_int | (8 as libc::c_int | 128 as libc::c_int)) as u8_0;
                }
                if maskvar as libc::c_int & 1 as libc::c_int != 0 {
                    if (*pe).note as libc::c_int == last_note[col as usize] as libc::c_int {
                        maskvar = (maskvar as libc::c_int & !(1 as libc::c_int)) as u8_0;
                    } else {
                        last_note[col as usize] = (*pe).note as u16_0;
                        if last_note[col as usize] as libc::c_int == 254 as libc::c_int
                            || last_note[col as usize] as libc::c_int == 255 as libc::c_int
                        {
                            last_note[col as usize] = 256 as libc::c_int as u16_0;
                        }
                    }
                }
                if maskvar as libc::c_int & 2 as libc::c_int != 0 {
                    if (*pe).inst as libc::c_int == last_inst[col as usize] as libc::c_int {
                        maskvar = (maskvar as libc::c_int & !(2 as libc::c_int)) as u8_0;
                    } else {
                        last_inst[col as usize] = (*pe).inst as u16_0;
                    }
                }
                if maskvar as libc::c_int & 4 as libc::c_int != 0 {
                    if (*pe).vol as libc::c_int == last_vol[col as usize] as libc::c_int {
                        maskvar = (maskvar as libc::c_int & !(4 as libc::c_int)) as u8_0;
                    } else {
                        last_vol[col as usize] = (*pe).vol as u16_0;
                    }
                }
                if maskvar as libc::c_int & 8 as libc::c_int != 0 {
                    if (*pe).fx as libc::c_int == last_fx[col as usize] as libc::c_int
                        && (*pe).param as libc::c_int == last_param[col as usize] as libc::c_int
                    {
                        maskvar = (maskvar as libc::c_int & !(8 as libc::c_int)) as u8_0;
                    } else {
                        last_fx[col as usize] = (*pe).fx as u16_0;
                        last_param[col as usize] = (*pe).param as u16_0;
                    }
                }
                if maskvar as libc::c_int != last_mask[col as usize] as libc::c_int {
                    chanvar = (chanvar as libc::c_int | 128 as libc::c_int) as u8_0;
                    last_mask[col as usize] = maskvar as u16_0;
                }
                write8(chanvar);
                if chanvar as libc::c_int & 128 as libc::c_int != 0 {
                    write8(maskvar);
                }
                if maskvar as libc::c_int & 1 as libc::c_int != 0 {
                    write8((*pe).note);
                }
                if maskvar as libc::c_int & 2 as libc::c_int != 0 {
                    write8((*pe).inst);
                }
                if maskvar as libc::c_int & 4 as libc::c_int != 0 {
                    write8((*pe).vol);
                }
                if maskvar as libc::c_int & 8 as libc::c_int != 0 {
                    write8((*pe).fx);
                    write8((*pe).param);
                }
            }
            col += 1;
        }
        write8(0 as libc::c_int as u8_0);
        row += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mark_Pattern_Row(
    mut mod_0: *mut MAS_Module,
    mut order: libc::c_int,
    mut row: libc::c_int,
) {
    let mut p = 0 as *mut Pattern;
    if row >= 256 as libc::c_int {
        return;
    }
    if (*mod_0).orders[order as usize] as libc::c_int == 255 as libc::c_int {
        order = 0 as libc::c_int;
    }
    while (*mod_0).orders[order as usize] as libc::c_int >= 254 as libc::c_int {
        if (*mod_0).orders[order as usize] as libc::c_int == 255 as libc::c_int {
            return;
        }
        if (*mod_0).orders[order as usize] as libc::c_int == 254 as libc::c_int {
            order += 1;
        }
    }
    p = &mut *((*mod_0).patterns)
        .offset(*((*mod_0).orders).as_mut_ptr().offset(order as isize) as isize)
        as *mut Pattern;
    (*p).cmarks[row as usize] = (0 as libc::c_int == 0) as libc::c_int as bool_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mark_Patterns(mut mod_0: *mut MAS_Module) {
    let mut o: libc::c_int = 0;
    let mut p: libc::c_int = 0;
    let mut row: libc::c_int = 0;
    let mut col: libc::c_int = 0;
    let mut pe = 0 as *mut PatternEntry;
    o = 0 as libc::c_int;
    while o < (*mod_0).order_count as libc::c_int {
        p = (*mod_0).orders[o as usize] as libc::c_int;
        if p == 255 as libc::c_int {
            break;
        }
        if !(p == 254 as libc::c_int) {
            if !(p >= (*mod_0).patt_count as libc::c_int) {
                row = 0 as libc::c_int;
                while row < (*((*mod_0).patterns).offset(p as isize)).nrows as libc::c_int {
                    col = 0 as libc::c_int;
                    while col < 32 as libc::c_int {
                        pe = &mut *((*((*mod_0).patterns).offset(p as isize)).data)
                            .as_mut_ptr()
                            .offset((row * 32 as libc::c_int + col) as isize)
                            as *mut PatternEntry;
                        if (*pe).fx as libc::c_int == 3 as libc::c_int {
                            if (*pe).param as libc::c_int != 0 as libc::c_int {
                                Mark_Pattern_Row(
                                    mod_0,
                                    o + 1 as libc::c_int,
                                    (*pe).param as libc::c_int,
                                );
                            }
                        } else if (*pe).fx as libc::c_int == 19 as libc::c_int {
                            if (*pe).param as libc::c_int == 0xb0 as libc::c_int {
                                Mark_Pattern_Row(mod_0, o, row);
                            }
                        }
                        col += 1;
                    }
                    row += 1;
                }
            }
        }
        o += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Write_MAS(
    mut mod_0: *mut MAS_Module,
    mut verbose: bool_0,
    mut msl_dep: bool_0,
) -> libc::c_int {
    let mut x: libc::c_int = 0;
    let mut fpos_pointer: libc::c_int = 0;
    file_get_byte_count();
    write32(0xba as libc::c_int as u32_0);
    write8(0 as libc::c_int as u8_0);
    write8(0x18 as libc::c_int as u8_0);
    write8(0xba as libc::c_int as u8_0);
    write8(0xba as libc::c_int as u8_0);
    MAS_OFFSET = file_tell_write() as u32_0;
    write8((*mod_0).order_count as u8_0);
    write8((*mod_0).inst_count);
    write8((*mod_0).samp_count);
    write8((*mod_0).patt_count);
    write8(
        ((if (*mod_0).link_gxx as libc::c_int != 0 {
            1 as libc::c_int
        } else {
            0 as libc::c_int
        }) | (if (*mod_0).old_effects as libc::c_int != 0 {
            2 as libc::c_int
        } else {
            0 as libc::c_int
        }) | (if (*mod_0).freq_mode as libc::c_int != 0 {
            4 as libc::c_int
        } else {
            0 as libc::c_int
        }) | (if (*mod_0).xm_mode as libc::c_int != 0 {
            8 as libc::c_int
        } else {
            0 as libc::c_int
        }) | (if msl_dep as libc::c_int != 0 {
            16 as libc::c_int
        } else {
            0 as libc::c_int
        }) | (if (*mod_0).old_mode as libc::c_int != 0 {
            32 as libc::c_int
        } else {
            0 as libc::c_int
        })) as u8_0,
    );
    write8((*mod_0).global_volume);
    write8((*mod_0).initial_speed);
    write8((*mod_0).initial_tempo);
    write8((*mod_0).restart_pos);
    write8(0xba as libc::c_int as u8_0);
    write8(0xba as libc::c_int as u8_0);
    write8(0xba as libc::c_int as u8_0);
    x = 0 as libc::c_int;
    while x < 32 as libc::c_int {
        write8((*mod_0).channel_volume[x as usize]);
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < 32 as libc::c_int {
        write8((*mod_0).channel_panning[x as usize]);
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).order_count as libc::c_int {
        if ((*mod_0).orders[x as usize] as libc::c_int) < 254 as libc::c_int {
            if ((*mod_0).orders[x as usize] as libc::c_int) < (*mod_0).patt_count as libc::c_int {
                write8((*mod_0).orders[x as usize]);
            } else {
                write8(254 as libc::c_int as u8_0);
            }
        } else {
            write8((*mod_0).orders[x as usize]);
        }
        x += 1;
    }
    while x < 200 as libc::c_int {
        write8(255 as libc::c_int as u8_0);
        x += 1;
    }
    fpos_pointer = file_tell_write();
    x = 0 as libc::c_int;
    while x
        < (*mod_0).inst_count as libc::c_int * 4 as libc::c_int
            + (*mod_0).samp_count as libc::c_int * 4 as libc::c_int
            + (*mod_0).patt_count as libc::c_int * 4 as libc::c_int
    {
        write8(0xba as libc::c_int as u8_0);
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"Header: %i bytes\n\0" as *const u8 as *const libc::c_char,
            file_get_byte_count(),
        );
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).inst_count as libc::c_int {
        Write_Instrument(&mut *((*mod_0).instruments).offset(x as isize));
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).samp_count as libc::c_int {
        Write_Sample(&mut *((*mod_0).samples).offset(x as isize));
        x += 1;
    }
    if verbose != 0 {
        printf(
            b"Instruments: %i bytes\n\0" as *const u8 as *const libc::c_char,
            file_get_byte_count(),
        );
    }
    Mark_Patterns(mod_0);
    x = 0 as libc::c_int;
    while x < (*mod_0).patt_count as libc::c_int {
        Write_Pattern(
            &mut *((*mod_0).patterns).offset(x as isize),
            (*mod_0).xm_mode,
        );
        x += 1;
    }
    align32();
    if verbose != 0 {
        printf(
            b"Patterns: %i bytes\n\0" as *const u8 as *const libc::c_char,
            file_get_byte_count(),
        );
    }
    MAS_FILESIZE = (file_tell_write() as libc::c_uint).wrapping_sub(MAS_OFFSET);
    file_seek_write(
        MAS_OFFSET.wrapping_sub(8 as libc::c_int as libc::c_uint) as libc::c_int,
        0 as libc::c_int,
    );
    write32(MAS_FILESIZE);
    file_seek_write(fpos_pointer, 0 as libc::c_int);
    x = 0 as libc::c_int;
    while x < (*mod_0).inst_count as libc::c_int {
        write32((*((*mod_0).instruments).offset(x as isize)).parapointer);
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).samp_count as libc::c_int {
        printf(
            b"sample %s is at %d/%d of %d\n\0" as *const u8 as *const libc::c_char,
            ((*((*mod_0).samples).offset(x as isize)).name).as_mut_ptr(),
            (*((*mod_0).samples).offset(x as isize)).parapointer,
            file_tell_write(),
            (*((*mod_0).samples).offset(x as isize)).sample_length,
        );
        write32((*((*mod_0).samples).offset(x as isize)).parapointer);
        x += 1;
    }
    x = 0 as libc::c_int;
    while x < (*mod_0).patt_count as libc::c_int {
        write32((*((*mod_0).patterns).offset(x as isize)).parapointer);
        x += 1;
    }
    return MAS_FILESIZE as libc::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn Delete_Module(mut mod_0: *mut MAS_Module) {
    let mut x: libc::c_int = 0;
    if !((*mod_0).instruments).is_null() {
        free((*mod_0).instruments as *mut libc::c_void);
    }
    if !((*mod_0).samples).is_null() {
        x = 0 as libc::c_int;
        while x < (*mod_0).samp_count as libc::c_int {
            if !((*((*mod_0).samples).offset(x as isize)).data).is_null() {
                free((*((*mod_0).samples).offset(x as isize)).data);
            }
            x += 1;
        }
        free((*mod_0).samples as *mut libc::c_void);
    }
    if !((*mod_0).patterns).is_null() {
        free((*mod_0).patterns as *mut libc::c_void);
    }
}
