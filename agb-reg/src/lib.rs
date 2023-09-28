#![no_std]

//! This crate contains definitions of the hardware registers used in the Game
//! Boy Advance. It contains bitfields that define the registers as well as
//! pointers too them.
//!
//! This crate may only be used on the GBA, use elsewhere is a very quick way to
//! get UB.
//!
//! Do note that registers may overlap in certain modes.

use bilge::prelude::*;

pub const DISPLAY_CONTROL: *mut DisplayControl = 0x0400_0000 as *mut _;
pub const VERTICAL_COUNT: *mut u16 = 0x0400_0006 as *mut _;

pub const fn background_control(bg: usize) -> *mut BackgroundControl {
    assert!(bg < 4, "background must be in range 0..=3");
    (0x0400_0008 + bg * core::mem::size_of::<BackgroundControl>()) as *mut _
}

pub const BACKGROUND_0_CONTROL: *mut BackgroundControl = background_control(0);
pub const BACKGROUND_1_CONTROL: *mut BackgroundControl = background_control(1);
pub const BACKGROUND_2_CONTROL: *mut BackgroundControl = background_control(2);
pub const BACKGROUND_3_CONTROL: *mut BackgroundControl = background_control(3);

pub const BACKGROUND_CONTROL: *mut [BackgroundControl; 4] = background_control(0).cast();

pub const fn background_offset(bg: usize) -> *mut Offset {
    assert!(bg < 4, "background must be in range 0..=3");

    (0x0400_0010 + bg * core::mem::size_of::<Offset>()) as *mut _
}

pub const fn background_offset_horizontal(bg: usize) -> *mut u16 {
    assert!(bg < 4, "background must be in range 0..=3");

    (0x0400_0010 + bg * core::mem::size_of::<Offset>()) as *mut u16
}

pub const fn background_offset_vertical(bg: usize) -> *mut u16 {
    assert!(bg < 4, "background must be in range 0..=3");

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

pub const BACKGROUND_OFFSET: *mut [Offset; 4] = 0x0400_0010 as *mut _;

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Offset {
    pub horizontal: u16,
    pub vertical: u16,
}

#[bitsize(16)]
#[derive(TryFromBits, Clone, Copy, PartialEq, DebugBits)]
pub struct DisplayControl {
    mode: DisplayMode,
    is_game_boy_colour: bool,
    page_select: Page,
    hblank_in_oam: bool,
    object_mapping: ObjectMappingMode,
    force_blank: bool,
    background_0: IsEnabled,
    background_1: IsEnabled,
    background_2: IsEnabled,
    background_3: IsEnabled,
    object: IsEnabled,
    window_0: IsEnabled,
    window_1: IsEnabled,
    window_object: IsEnabled,
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
#[derive(FromBits, Clone, Copy, PartialEq, DebugBits)]
pub struct BackgroundControl {
    priority: Priority,
    tile_block: u2,
    reserved: u2,
    mosaic: IsEnabled,
    colour_mode: ColourMode,
    map_block: u5,
    affine_wrapping: IsEnabled,
    size: BackgroundSize,
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy, PartialEq, DebugBits)]
pub struct Priority(u2);

#[bitsize(1)]
#[derive(FromBits, Clone, Copy, PartialEq, Debug)]
pub enum ColourMode {
    FourBitPerPixel,
    EightBitPerPixel,
}

#[bitsize(2)]
#[derive(FromBits, Clone, Copy, PartialEq, DebugBits)]
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
