use ::libc;
extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
}
pub type u16_0 = libc::c_ushort;
pub type u32_0 = libc::c_uint;
pub type s16 = libc::c_short;
pub type u8_0 = libc::c_uchar;
pub type s8 = libc::c_schar;
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
pub static mut IndexTable: [s8; 8] = [
    -(1 as libc::c_int) as s8,
    -(1 as libc::c_int) as s8,
    -(1 as libc::c_int) as s8,
    -(1 as libc::c_int) as s8,
    2 as libc::c_int as s8,
    4 as libc::c_int as s8,
    6 as libc::c_int as s8,
    8 as libc::c_int as s8,
];
#[no_mangle]
pub static mut AdpcmTable: [u16_0; 89] = [
    7 as libc::c_int as u16_0,
    8 as libc::c_int as u16_0,
    9 as libc::c_int as u16_0,
    10 as libc::c_int as u16_0,
    11 as libc::c_int as u16_0,
    12 as libc::c_int as u16_0,
    13 as libc::c_int as u16_0,
    14 as libc::c_int as u16_0,
    16 as libc::c_int as u16_0,
    17 as libc::c_int as u16_0,
    19 as libc::c_int as u16_0,
    21 as libc::c_int as u16_0,
    23 as libc::c_int as u16_0,
    25 as libc::c_int as u16_0,
    28 as libc::c_int as u16_0,
    31 as libc::c_int as u16_0,
    34 as libc::c_int as u16_0,
    37 as libc::c_int as u16_0,
    41 as libc::c_int as u16_0,
    45 as libc::c_int as u16_0,
    50 as libc::c_int as u16_0,
    55 as libc::c_int as u16_0,
    60 as libc::c_int as u16_0,
    66 as libc::c_int as u16_0,
    73 as libc::c_int as u16_0,
    80 as libc::c_int as u16_0,
    88 as libc::c_int as u16_0,
    97 as libc::c_int as u16_0,
    107 as libc::c_int as u16_0,
    118 as libc::c_int as u16_0,
    130 as libc::c_int as u16_0,
    143 as libc::c_int as u16_0,
    157 as libc::c_int as u16_0,
    173 as libc::c_int as u16_0,
    190 as libc::c_int as u16_0,
    209 as libc::c_int as u16_0,
    230 as libc::c_int as u16_0,
    253 as libc::c_int as u16_0,
    279 as libc::c_int as u16_0,
    307 as libc::c_int as u16_0,
    337 as libc::c_int as u16_0,
    371 as libc::c_int as u16_0,
    408 as libc::c_int as u16_0,
    449 as libc::c_int as u16_0,
    494 as libc::c_int as u16_0,
    544 as libc::c_int as u16_0,
    598 as libc::c_int as u16_0,
    658 as libc::c_int as u16_0,
    724 as libc::c_int as u16_0,
    796 as libc::c_int as u16_0,
    876 as libc::c_int as u16_0,
    963 as libc::c_int as u16_0,
    1060 as libc::c_int as u16_0,
    1166 as libc::c_int as u16_0,
    1282 as libc::c_int as u16_0,
    1411 as libc::c_int as u16_0,
    1552 as libc::c_int as u16_0,
    1707 as libc::c_int as u16_0,
    1878 as libc::c_int as u16_0,
    2066 as libc::c_int as u16_0,
    2272 as libc::c_int as u16_0,
    2499 as libc::c_int as u16_0,
    2749 as libc::c_int as u16_0,
    3024 as libc::c_int as u16_0,
    3327 as libc::c_int as u16_0,
    3660 as libc::c_int as u16_0,
    4026 as libc::c_int as u16_0,
    4428 as libc::c_int as u16_0,
    4871 as libc::c_int as u16_0,
    5358 as libc::c_int as u16_0,
    5894 as libc::c_int as u16_0,
    6484 as libc::c_int as u16_0,
    7132 as libc::c_int as u16_0,
    7845 as libc::c_int as u16_0,
    8630 as libc::c_int as u16_0,
    9493 as libc::c_int as u16_0,
    10442 as libc::c_int as u16_0,
    11487 as libc::c_int as u16_0,
    12635 as libc::c_int as u16_0,
    13899 as libc::c_int as u16_0,
    15289 as libc::c_int as u16_0,
    16818 as libc::c_int as u16_0,
    18500 as libc::c_int as u16_0,
    20350 as libc::c_int as u16_0,
    22385 as libc::c_int as u16_0,
    24623 as libc::c_int as u16_0,
    27086 as libc::c_int as u16_0,
    29794 as libc::c_int as u16_0,
    32767 as libc::c_int as u16_0,
];
unsafe extern "C" fn read_sample(
    mut sample: *mut Sample,
    mut position: libc::c_int,
) -> libc::c_int {
    let mut s: libc::c_int = 0;
    if (*sample).format as libc::c_int & 0x1 as libc::c_int != 0 {
        s = *((*sample).data as *mut s16).offset(position as isize) as libc::c_int;
    } else {
        let mut a = *((*sample).data as *mut s8).offset(position as isize) as libc::c_int;
        s = a << 8 as libc::c_int;
    }
    if s < -(32767 as libc::c_int) {
        s = -(32767 as libc::c_int);
    }
    return s;
}
unsafe extern "C" fn minmax(
    mut value: libc::c_int,
    mut low: libc::c_int,
    mut high: libc::c_int,
) -> libc::c_int {
    if value < low {
        value = low;
    }
    if value > high {
        value = high;
    }
    return value;
}
unsafe extern "C" fn calc_delta(mut diff: libc::c_int, mut step: libc::c_int) -> libc::c_int {
    let mut delta = step >> 3 as libc::c_int;
    if diff >= step {
        diff -= step;
        delta += step;
    }
    if diff >= step >> 1 as libc::c_int {
        diff -= step >> 1 as libc::c_int;
        delta += step >> 1 as libc::c_int;
    }
    if diff >= step >> 2 as libc::c_int {
        diff -= step >> 2 as libc::c_int;
        delta += step >> 2 as libc::c_int;
    }
    return delta;
}
#[no_mangle]
pub unsafe extern "C" fn adpcm_compress_sample(mut sample: *mut Sample) {
    let mut x: u32_0 = 0;
    let mut output = 0 as *mut u8_0;
    let mut prev_value: libc::c_int = 0;
    let mut curr_value: libc::c_int = 0;
    let mut diff: libc::c_int = 0;
    let mut data: libc::c_int = 0;
    let mut delta: libc::c_int = 0;
    let mut index: libc::c_int = 0;
    let mut step: libc::c_int = 0;
    output = malloc(
        ((*sample).sample_length)
            .wrapping_div(2 as libc::c_int as libc::c_uint)
            .wrapping_add(4 as libc::c_int as libc::c_uint) as libc::c_ulong,
    ) as *mut u8_0;
    prev_value = read_sample(sample, 0 as libc::c_int);
    index = 0 as libc::c_int;
    let mut i: libc::c_int = 0;
    let mut smallest_error: libc::c_int = 0;
    let mut tmp_error: libc::c_int = 0;
    smallest_error = 9999999 as libc::c_int;
    diff = read_sample(sample, 1 as libc::c_int) - read_sample(sample, 0 as libc::c_int);
    i = 0 as libc::c_int;
    while i < 88 as libc::c_int {
        tmp_error = calc_delta(diff, i) - diff;
        if tmp_error < smallest_error {
            smallest_error = tmp_error;
            index = i;
        }
        i += 1;
    }
    *(output as *mut u32_0) = (prev_value | index << 16 as libc::c_int) as u32_0;
    step = AdpcmTable[index as usize] as libc::c_int;
    x = 0 as libc::c_int as u32_0;
    while x < (*sample).sample_length {
        curr_value = read_sample(sample, x as libc::c_int);
        diff = curr_value - prev_value;
        if diff < 0 as libc::c_int {
            diff = -diff;
            data = 8 as libc::c_int;
        } else {
            data = 0 as libc::c_int;
        }
        delta = step >> 3 as libc::c_int;
        if diff >= step {
            data |= 4 as libc::c_int;
            diff -= step;
            delta += step;
        }
        if diff >= step >> 1 as libc::c_int {
            data |= 2 as libc::c_int;
            diff -= step >> 1 as libc::c_int;
            delta += step >> 1 as libc::c_int;
        }
        if diff >= step >> 2 as libc::c_int {
            data |= 1 as libc::c_int;
            diff -= step >> 2 as libc::c_int;
            delta += step >> 2 as libc::c_int;
        }
        prev_value += if data & 8 as libc::c_int != 0 {
            -delta
        } else {
            delta
        };
        prev_value = minmax(prev_value, -(0x7fff as libc::c_int), 0x7fff as libc::c_int);
        index = minmax(
            index + IndexTable[(data & 7 as libc::c_int) as usize] as libc::c_int,
            0 as libc::c_int,
            88 as libc::c_int,
        );
        step = AdpcmTable[index as usize] as libc::c_int;
        if x & 1 as libc::c_int as libc::c_uint != 0 {
            let ref mut fresh0 = *output.offset(
                (x >> 1 as libc::c_int).wrapping_add(4 as libc::c_int as libc::c_uint) as isize,
            );
            *fresh0 = (*fresh0 as libc::c_int | data << 4 as libc::c_int) as u8_0;
        } else {
            *output.offset(
                (x >> 1 as libc::c_int).wrapping_add(4 as libc::c_int as libc::c_uint) as isize,
            ) = data as u8_0;
        }
        x = x.wrapping_add(1);
    }
    free((*sample).data);
    let ref mut fresh1 = (*sample).data;
    *fresh1 = output as *mut libc::c_void;
    (*sample).format = 0x4 as libc::c_int as u8_0;
    (*sample).sample_length = ((*sample).sample_length)
        .wrapping_div(2 as libc::c_int as libc::c_uint)
        .wrapping_add(4 as libc::c_int as libc::c_uint);
    let ref mut fresh2 = (*sample).loop_start;
    *fresh2 =
        (*fresh2 as libc::c_uint).wrapping_div(2 as libc::c_int as libc::c_uint) as u32_0 as u32_0;
    let ref mut fresh3 = (*sample).loop_end;
    *fresh3 =
        (*fresh3 as libc::c_uint).wrapping_div(2 as libc::c_int as libc::c_uint) as u32_0 as u32_0;
    let ref mut fresh4 = (*sample).loop_start;
    *fresh4 =
        (*fresh4 as libc::c_uint).wrapping_add(4 as libc::c_int as libc::c_uint) as u32_0 as u32_0;
    let ref mut fresh5 = (*sample).loop_end;
    *fresh5 =
        (*fresh5 as libc::c_uint).wrapping_add(4 as libc::c_int as libc::c_uint) as u32_0 as u32_0;
}
