#![no_std]
#![deny(clippy::all)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::missing_const_for_fn)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]

//! This crate contains definitions of the hardware registers used in the Game
//! Boy Advance. It contains bitfields that define the registers as well as
//! pointers too them.
//!
//! This crate may only be used on the GBA, use elsewhere is a very quick way to
//! get UB.
//!
//! Do note that registers may overlap in certain modes.

use bilge::prelude::*;

pub trait UpdateVolatile<T: Copy> {
    /// Performs a volatile read, lets you modify it, then performs a volatile
    /// write with the updated value.
    ///
    /// ```no_run
    /// use agb_reg::{DISPLAY_CONTROL, UpdateVolatile};
    ///
    /// unsafe {
    ///     DISPLAY_CONTROL.update_volatile(|x| x.set_background_at(2, true.into()));
    /// }
    /// ```
    ///
    /// # Safety
    /// This is designed to volatily read and write to a raw pointer. All the
    /// usual raw pointer rules apply.
    unsafe fn update_volatile<F>(self, f: F)
    where
        F: FnOnce(&mut T);
}

impl<T: Copy> UpdateVolatile<T> for *mut T {
    #[inline(always)]
    unsafe fn update_volatile<F>(self, f: F)
    where
        F: FnOnce(&mut T),
    {
        // Safety: the safety guarentees of this function mean I can do this
        let mut t = unsafe { self.read_volatile() };
        f(&mut t);
        unsafe { self.write_volatile(t) };
    }
}

pub trait PointerAt<T> {
    /// ```no_run
    /// use agb_reg::{BACKGROUND_CONTROL, UpdateVolatile, PointerAt};
    ///
    /// unsafe {
    ///     BACKGROUND_CONTROL
    ///         .at(1)
    ///         .update_volatile(|x| x.set_mosaic(true.into()));
    /// }
    /// ```
    ///
    /// Panics if out of bounds.
    ///
    /// # Safety
    /// This is designed to dereference raw pointers, so the normal pointer
    /// rules apply.
    unsafe fn at(self, idx: usize) -> *mut T;

    /// ```no_run
    /// use agb_reg::{BACKGROUND_CONTROL, UpdateVolatile, PointerAt};
    ///
    /// unsafe {
    ///     BACKGROUND_CONTROL
    ///         .at_unchecked(1)
    ///         .update_volatile(|x| x.set_mosaic(true.into()));
    /// }
    /// ```
    ///
    /// # Safety
    /// This is designed to dereference raw pointers, so the normal pointer
    /// rules apply. Also no bounds checking is performed.
    unsafe fn at_unchecked(self, idx: usize) -> *mut T;
}

impl<T, const N: usize> PointerAt<T> for *mut [T; N] {
    #[inline(always)]
    unsafe fn at(self, idx: usize) -> *mut T {
        unsafe { (&mut ((*self)[idx])) as *mut T }
    }

    #[inline(always)]
    unsafe fn at_unchecked(self, idx: usize) -> *mut T {
        unsafe { ((*self).get_unchecked_mut(idx)) as *mut T }
    }
}

impl<T> PointerAt<T> for *mut [T] {
    #[inline(always)]
    unsafe fn at(self, idx: usize) -> *mut T {
        unsafe { (&mut ((*self)[idx])) as *mut T }
    }

    #[inline(always)]
    unsafe fn at_unchecked(self, idx: usize) -> *mut T {
        unsafe { ((*self).get_unchecked_mut(idx)) as *mut T }
    }
}

pub const DISPLAY_CONTROL: *mut DisplayControl = 0x0400_0000 as *mut _;
pub const VERTICAL_COUNT: *mut u16 = 0x0400_0006 as *mut _;

pub const fn background_control(bg: usize) -> *mut BackgroundControl {
    assert!(bg < 4, "background must be in range 0..=3");

    background_control_unchecked(bg)
}

pub const fn background_control_unchecked(bg: usize) -> *mut BackgroundControl {
    (0x0400_0008 + bg * core::mem::size_of::<BackgroundControl>()) as *mut _
}

pub const BACKGROUND_0_CONTROL: *mut BackgroundControl = background_control(0);
pub const BACKGROUND_1_CONTROL: *mut BackgroundControl = background_control(1);
pub const BACKGROUND_2_CONTROL: *mut BackgroundControl = background_control(2);
pub const BACKGROUND_3_CONTROL: *mut BackgroundControl = background_control(3);

pub const BACKGROUND_CONTROL: *mut [BackgroundControl; 4] = background_control(0).cast();

pub const fn background_offset(bg: usize) -> *mut Offset {
    assert!(bg < 4, "background must be in range 0..=3");

    background_offset_unchecked(bg)
}

pub const fn background_offset_unchecked(bg: usize) -> *mut Offset {
    (0x0400_0010 + bg * core::mem::size_of::<Offset>()) as *mut _
}

pub const fn background_offset_horizontal(bg: usize) -> *mut u16 {
    assert!(bg < 4, "background must be in range 0..=3");

    background_offset_horizontal_unchecked(bg)
}

pub const fn background_offset_horizontal_unchecked(bg: usize) -> *mut u16 {
    (0x0400_0010 + bg * core::mem::size_of::<Offset>()) as *mut u16
}

pub const fn background_offset_vertical(bg: usize) -> *mut u16 {
    assert!(bg < 4, "background must be in range 0..=3");

    background_offset_vertical_unchecked(bg)
}

pub const fn background_offset_vertical_unchecked(bg: usize) -> *mut u16 {
    (0x0400_0010 + bg * core::mem::size_of::<Offset>() + core::mem::size_of::<u16>()) as *mut u16
}

pub const BACKGROUND_0_OFFSET: *mut Offset = background_offset(0);
pub const BACKGROUND_0_OFFSET_HORIZONTAL: *mut u16 = background_offset_horizontal(0);
pub const BACKGROUND_0_OFFSET_VERTICAL: *mut u16 = background_offset_vertical(0);

pub const BACKGROUND_1_OFFSET: *mut Offset = background_offset(1);
pub const BACKGROUND_1_OFFSET_HORIZONTAL: *mut u16 = background_offset_horizontal(1);
pub const BACKGROUND_1_OFFSET_VERTICAL: *mut u16 = background_offset_vertical(1);

pub const BACKGROUND_2_OFFSET: *mut Offset = background_offset(2);
pub const BACKGROUND_2_OFFSET_HORIZONTAL: *mut u16 = background_offset_horizontal(2);
pub const BACKGROUND_2_OFFSET_VERTICAL: *mut u16 = background_offset_vertical(2);

pub const BACKGROUND_3_OFFSET: *mut Offset = background_offset(3);
pub const BACKGROUND_3_OFFSET_HORIZONTAL: *mut u16 = background_offset_horizontal(3);
pub const BACKGROUND_3_OFFSET_VERTICAL: *mut u16 = background_offset_vertical(3);

pub const BACKGROUND_OFFSET: *mut [Offset; 4] = background_offset(0).cast();

pub const fn window_horizontal(window: usize) -> *mut WindowHorizontal {
    assert!(window < 2, "window must be less than 2");

    window_horizontal_unchecked(window)
}

pub const fn window_horizontal_unchecked(window: usize) -> *mut WindowHorizontal {
    (0x0400_0040 + window * core::mem::size_of::<WindowHorizontal>()) as *mut _
}

pub const fn window_vertical(window: usize) -> *mut WindowVertical {
    assert!(window < 2, "window must be less than 2");

    window_vertical_unchecked(window)
}

pub const fn window_vertical_unchecked(window: usize) -> *mut WindowVertical {
    (0x0400_0044 + window * core::mem::size_of::<WindowVertical>()) as *mut _
}

pub const WINDOW_0_HORIZONTAL: *mut WindowHorizontal = window_horizontal(0);
pub const WINDOW_0_VERTICAL: *mut WindowVertical = window_vertical(0);

pub const WINDOW_1_HORIZONTAL: *mut WindowHorizontal = window_horizontal(1);
pub const WINDOW_1_VERTICAL: *mut WindowVertical = window_vertical(1);

pub const WINDOW_INNER: *mut WindowInner = 0x0400_0048 as *mut _;
pub const WINDOW_OUTER: *mut WindowOuter = 0x0400_004A as *mut _;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackgroundAffineMatrixParameter {
    A,
    B,
    C,
    D,
    X,
    Y,
}

pub const fn background_affine_matrix_parameter(
    bg: usize,
    parameter: BackgroundAffineMatrixParameter,
) -> *mut u16 {
    assert!(bg >= 2 && bg < 4, "background must be in range 2..=3");

    background_affine_matrix_parameter_unchecked(bg, parameter)
}

pub const fn background_affine_matrix_parameter_unchecked(
    bg: usize,
    parameter: BackgroundAffineMatrixParameter,
) -> *mut u16 {
    (0x0400_0020
        + (bg - 2) * core::mem::size_of::<BackgroundAffineMatrix>()
        + (parameter as usize) * core::mem::size_of::<u16>()) as *mut _
}

pub const BACKGROUND_2_AFFINE: *mut BackgroundAffineMatrix =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::A).cast();
pub const BACKGROUND_2_AFFINE_A: *mut u16 =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::A);
pub const BACKGROUND_2_AFFINE_B: *mut u16 =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::B);
pub const BACKGROUND_2_AFFINE_C: *mut u16 =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::C);
pub const BACKGROUND_2_AFFINE_D: *mut u16 =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::D);
pub const BACKGROUND_2_AFFINE_X: *mut u16 =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::X);
pub const BACKGROUND_2_AFFINE_Y: *mut u16 =
    background_affine_matrix_parameter(2, BackgroundAffineMatrixParameter::Y);

pub const BACKGROUND_3_AFFINE: *mut BackgroundAffineMatrix =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::A).cast();
pub const BACKGROUND_3_AFFINE_A: *mut u16 =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::A);
pub const BACKGROUND_3_AFFINE_B: *mut u16 =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::B);
pub const BACKGROUND_3_AFFINE_C: *mut u16 =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::C);
pub const BACKGROUND_3_AFFINE_D: *mut u16 =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::D);
pub const BACKGROUND_3_AFFINE_X: *mut u16 =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::X);
pub const BACKGROUND_3_AFFINE_Y: *mut u16 =
    background_affine_matrix_parameter(3, BackgroundAffineMatrixParameter::Y);

pub const BLEND_CONTROL: *mut BlendControl = 0x0400_0050 as *mut _;
pub const BLEND_ALPHA: *mut BlendAlpha = 0x0400_0052 as *mut _;
pub const BLEND_BRIGHTNESS: *mut BlendBrighness = 0x0400_0054 as *mut _;

pub const MOSAIC: *mut Mosaic = 0x0400_004C as *mut _;

pub const SOUND_1_CONTROL_TONE_SWEEP: *mut SoundToneSweep = 0x0400_0060 as *mut _;
pub const SOUND_1_CONTROL_DUTY_LEN_ENVELOPE: *mut SoundDutyLenEnvelope = 0x0400_0062 as *mut _;
pub const SOUND_1_CONTROL_FREQUENCY: *mut SoundFrequencyControl = 0x0400_0064 as *mut _;

pub const SOUND_2_CONTROL_DUTY_LEN_ENVELOPE: *mut SoundDutyLenEnvelope = 0x0400_0068 as *mut _;
pub const SOUND_2_CONTROL_FREQUENCY: *mut SoundFrequencyControl = 0x0400_006C as *mut _;

pub const SOUND_3_CONTROL_WAVE: *mut SoundWaveSelect = 0x0400_0070 as *mut _;
pub const SOUND_3_CONTROL_LENGTH_VOLUME: *mut SoundLengthVolume = 0x0400_0072 as *mut _;
pub const SOUND_3_CONTROL_FREQUNECY: *mut SoundFrequencyControl = 0x0400_0074 as *mut _;

pub const SOUND_3_WAVE_PATTERN: *mut [u16; 8] = 0x0400_0090 as *mut _;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Offset {
    pub horizontal: u16,
    pub vertical: u16,
}

#[bitsize(16)]
#[derive(TryFromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct DisplayControl {
    pub mode: DisplayMode,
    pub is_game_boy_colour: bool,
    pub page_select: Page,
    pub hblank_in_oam: bool,
    pub object_mapping: ObjectMappingMode,
    pub force_blank: bool,
    pub background: [IsEnabled; 4],
    pub object: IsEnabled,
    pub window: [IsEnabled; 2],
    pub window_object: IsEnabled,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectMappingMode {
    Map2D,
    Map1D,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Front,
    Back,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, Debug, PartialEq, Eq)]
pub enum IsEnabled {
    Disabled,
    Enabled,
}

impl From<bool> for IsEnabled {
    fn from(value: bool) -> Self {
        match value {
            true => IsEnabled::Enabled,
            false => IsEnabled::Disabled,
        }
    }
}

impl From<IsEnabled> for bool {
    fn from(value: IsEnabled) -> Self {
        match value {
            IsEnabled::Disabled => false,
            IsEnabled::Enabled => true,
        }
    }
}

#[bitsize(3)]
#[derive(TryFromBits, Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayMode {
    Tiled0,
    Tiled1,
    Tiled2,
    Bitmap3,
    Bitmap4,
    Bitmap5,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct BackgroundControl {
    pub priority: Priority,
    pub tile_block: u2,
    reserved: u2,
    pub mosaic: IsEnabled,
    pub colour_mode: ColourMode,
    pub map_block: u5,
    pub affine_wrapping: IsEnabled,
    pub size: BackgroundSize,
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct Priority(u2);

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColourMode {
    FourBitPerPixel,
    EightBitPerPixel,
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct BackgroundSize(u2);

impl From<BackgroundRegularSize> for BackgroundSize {
    fn from(value: BackgroundRegularSize) -> Self {
        BackgroundSize::new(u2::new(value as u8))
    }
}

impl From<BackgroundAffineSize> for BackgroundSize {
    fn from(value: BackgroundAffineSize) -> Self {
        BackgroundSize::new(u2::new(value as u8))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackgroundRegularSize {
    R32x32,
    R64x32,
    R32x64,
    R64x64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BackgroundAffineSize {
    A16x16,
    A32x32,
    A64x64,
    A128x128,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BackgroundAffineMatrix {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub x: u16,
    pub y: u16,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct WindowHorizontal {
    pub right: u8,
    pub left: u8,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct WindowVertical {
    pub bottom: u8,
    pub top: u8,
}

#[bitsize(8)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct Window {
    pub background: [IsEnabled; 4],
    pub object: IsEnabled,
    pub effect: IsEnabled,
    reserved: u2,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct WindowInner {
    pub window: [Window; 2],
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct WindowOuter {
    pub outside: Window,
    pub object: Window,
}

#[bitsize(6)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct EffectTarget {
    pub background: [IsEnabled; 4],
    pub object: IsEnabled,
    pub backdrop: IsEnabled,
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlendMode {
    None,
    Alpha,
    Brighten,
    Darken,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct BlendControl {
    pub target: EffectTarget,
    pub mode: BlendMode,
    pub blend_target: EffectTarget,
    reserved: u2,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct BlendAlpha {
    pub first: u5,
    reserved: u3,
    pub second: u5,
    reserved: u3,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct BlendBrighness {
    pub brightness: u5,
    reserved: u11,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct Mosaic {
    pub background_horizontal: u4,
    pub background_vertical: u4,
    pub object_horizontal: u4,
    pub object_vertical: u4,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SweepFrequencyDirection {
    Increase,
    Decrease,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct SoundToneSweep {
    pub sweep_shift: u3,
    pub sweep_frequency_direction: SweepFrequencyDirection,
    pub sweep_time: u3,
    reserved: u9,
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DutyCycle {
    S12_5,
    S25,
    S50,
    S75,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum EnvelopeDirection {
    Decrease,
    Increase,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct SoundDutyLenEnvelope {
    pub length: u6,
    pub wave_duty: DutyCycle,
    pub envelope_step_time: u3,
    pub envelope_direction: EnvelopeDirection,
    pub initial_envelope_volume: u4,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Timed {
    Continue,
    Stop,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct SoundFrequencyControl {
    pub frequency: u11,
    reserved: u3,
    pub timed: Timed,
    pub restart: bool,
}

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum WaveTableSize {
    Single,
    Double,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct SoundWaveSelect {
    reserved: u5,
    pub wave_table_size: WaveTableSize,
    pub active_wave_table: u1,
    pub channel: IsEnabled,
    reserved: u8,
}

#[bitsize(3)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, Debug)]
pub enum SoundVolume {
    Mute = 0b000,
    Full = 0b001,
    Half = 0b010,
    Quarter = 0b011,
    #[fallback]
    ThreeQuarter = 0b100,
}

#[bitsize(16)]
#[derive(FromBits, Clone, Copy, PartialEq, Eq, DebugBits)]
pub struct SoundLengthVolume {
    pub length: u8,
    reserved: u5,
    pub volume: SoundVolume,
}
