use core::marker::PhantomData;

use agb_fixnum::Num;
pub use agb_sound_converter::include_sounds;
use alloc::{boxed::Box, vec, vec::Vec};

use crate::{
    interrupt::{add_interrupt_handler, Interrupt, InterruptHandler},
    InternalAllocator,
};

extern "C" {
    fn mmInit(gba_system: *mut MaxModGbaSystem);
    fn mmStart(id: i32, play_mode: i32);
    fn mmVBlank();
    fn mmFrame();

    fn mmEffectEx(sound_effect: *const MaxModSoundEffect) -> u16;
}

#[doc(hidden)]
pub unsafe trait TrackerId: Copy {
    fn id(self) -> i32;
}

#[doc(hidden)]
pub unsafe trait TrackerOutput {
    type ModId;
    type SfxId;
    fn sound_bank() -> &'static [u8];
}

#[non_exhaustive]
pub struct Tracker<'a, Output: TrackerOutput> {
    _tracker: PhantomData<Output>,
    _interrupt_handler: InterruptHandler<'a>,
}

impl<'a, Output: TrackerOutput> Tracker<'a, Output>
where
    Output::ModId: TrackerId,
    Output::SfxId: TrackerId,
{
    pub(crate) unsafe fn new(num_channels: i32, mix_mode: MixMode) -> Self {
        init(Output::sound_bank(), num_channels, mix_mode);
        let vblank_handler = add_interrupt_handler(Interrupt::VBlank, |_cs| unsafe { vblank() });

        Self {
            _tracker: PhantomData,
            _interrupt_handler: vblank_handler,
        }
    }

    pub fn start(&self, music: Output::ModId) {
        unsafe {
            start(music.id());
        }
    }

    pub fn frame(&self) {
        unsafe {
            frame();
        }
    }

    pub fn effect(&self, effect: SoundEffectOptions<Output::SfxId>) -> SoundEffectHandle {
        let handle = unsafe { play_effect(&effect.into_maxmod()) };

        SoundEffectHandle(handle)
    }
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

static mut HAS_INITED: bool = false;

unsafe fn init(soundbank: &'static [u8], num_channels: i32, mix_mode: MixMode) {
    if HAS_INITED {
        panic!("Can only init tracker once");
    }
    HAS_INITED = true;

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

unsafe fn start(id: i32) {
    unsafe {
        mmStart(id, 0);
    }
}

static mut HAS_RUN_VBLANK: bool = false;

unsafe fn vblank() {
    unsafe {
        mmVBlank();
        HAS_RUN_VBLANK = true;
    }
}

unsafe fn frame() {
    unsafe {
        if HAS_RUN_VBLANK {
            HAS_RUN_VBLANK = false;
            mmFrame();
        }
    }
}

unsafe fn play_effect(effect: &MaxModSoundEffect) -> u16 {
    unsafe { mmEffectEx(effect) }
}

#[repr(C)]
struct MaxModSoundEffect {
    id: i32,
    rate: Num<u16, 10>,
    handle_to_recycle: u16,
    volume: u8,
    panning: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct SoundEffectHandle(u16);

pub struct SoundEffectOptions<T> {
    id: T,
    rate: Num<u16, 10>,
    handle_to_recycle: Option<SoundEffectHandle>,
    volume: u8,
    panning: u8,
}

impl<T> SoundEffectOptions<T>
where
    T: TrackerId,
{
    pub fn new(sfx_id: T) -> Self {
        Self {
            id: sfx_id,
            rate: 1.into(),
            handle_to_recycle: None,
            volume: 128,
            panning: 128,
        }
    }

    pub fn rate(&mut self, new_rate: Num<u16, 10>) -> &mut Self {
        self.rate = new_rate;
        self
    }

    pub fn volume(&mut self, new_volume: u8) -> &mut Self {
        self.volume = new_volume;
        self
    }
    
    pub fn panning(&mut self, new_panning: u8) -> &mut Self {
        self.panning = new_panning;
        self
    }

    pub fn recycle(&mut self, handle: SoundEffectHandle) -> &mut Self {
        self.handle_to_recycle = Some(handle);
        self
    }

    fn into_maxmod(self) -> MaxModSoundEffect {
        MaxModSoundEffect {
            id: self.id.id(),
            rate: self.rate,
            handle_to_recycle: self.handle_to_recycle.map_or(0, |h| h.0),
            volume: self.volume,
            panning: self.panning,
        }
    }
}
