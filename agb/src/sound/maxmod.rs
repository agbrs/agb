pub use agb_sound_converter::include_sounds;
use alloc::{boxed::Box, vec, vec::Vec};

use crate::InternalAllocator;

extern "C" {
    fn mmInit(gba_system: *mut MaxModGbaSystem);
    fn mmStart(id: i32, play_mode: i32);
    fn mmVBlank();
    fn mmFrame();
}

#[derive(Clone, Copy, Debug)]
pub enum MixMode {
    Hz8,
    Hz10,
    Hz13,
    Hz16,
    Hz18,
    Hz21,
    Hz27,
    Hz31,
}

impl MixMode {
    const fn buf_size(self) -> usize {
        use MixMode::*;

        match self {
            Hz8 => 544,
            Hz10 => 704,
            Hz13 => 896,
            Hz16 => 1056,
            Hz18 => 1216,
            Hz21 => 1408,
            Hz27 => 1792,
            Hz31 => 2112,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ModFile(i32);

impl ModFile {
    #[must_use]
    #[doc(hidden)]
    pub const unsafe fn new(id: i32) -> Self {
        Self(id)
    }
}

#[repr(C)]
struct MaxModGbaSystem {
    mix_mode: i32,
    mod_channel_count: u32,
    mix_channel_count: u32,
    module_channels: *mut u8,
    active_channels: *mut u8,
    mixing_channels: *mut u8,
    mixing_memory: *mut u8,
    wave_memory: *mut u8,
    soundbank: *const u8,
}

const MM_SIZEOF_MODCH: isize = 40;
const MM_SIZEOF_ACTCH: isize = 28;
const MM_SIZEOF_MIXCH: isize = 24;

pub fn init(soundbank: &'static [u8], num_channels: i32, mix_mode: MixMode) {
    let num_channels = num_channels as isize;
    let buf_size = mix_mode.buf_size();

    let buffer: Vec<u8> = vec![
        0;
        (num_channels * (MM_SIZEOF_MODCH + MM_SIZEOF_ACTCH + MM_SIZEOF_MIXCH))
            as usize
            + buf_size
    ];
    let buffer = Box::into_raw(buffer.into_boxed_slice()) as *mut u8;

    let mut mixing_memory =
        Vec::<u8, InternalAllocator>::with_capacity_in(buf_size, InternalAllocator);
    mixing_memory.resize(buf_size, 0);
    let mixing_memory = Box::into_raw(mixing_memory.into_boxed_slice()) as *mut u8;

    unsafe {
        let mut max_mod_system = MaxModGbaSystem {
            mix_mode: mix_mode as i32,
            mod_channel_count: num_channels as u32,
            mix_channel_count: num_channels as u32,
            module_channels: buffer,
            active_channels: buffer.offset(num_channels * MM_SIZEOF_MODCH),
            mixing_channels: buffer.offset(num_channels * (MM_SIZEOF_MODCH + MM_SIZEOF_ACTCH)),
            mixing_memory,
            wave_memory: buffer
                .offset(num_channels * (MM_SIZEOF_MODCH + MM_SIZEOF_ACTCH + MM_SIZEOF_MIXCH)),
            soundbank: soundbank.as_ptr(),
        };

        mmInit(&mut max_mod_system as *mut _);
    }
}

pub fn start(id: ModFile) {
    unsafe {
        mmStart(id.0, 0);
    }
}

pub fn vblank() {
    unsafe {
        mmVBlank();
    }
}

pub fn frame() {
    unsafe {
        mmFrame();
    }
}
