use arbitrary_int::*;
use bitbybit::{bitenum, bitfield};

use crate::display::Priority;

use super::{
    AffineBackgroundSize, AffineBackgroundWrapBehaviour, RegularBackgroundSize, TileFormat,
};

#[bitfield(u16)]
#[derive(Default)]
pub(crate) struct DisplayControlRegister {
    #[bits(0..=2, rw)]
    pub video_mode: u3,
    #[bits(4..=4, rw)]
    _display_frame_select: u1,
    #[bits(5..=5, rw)]
    hblank_interval_free: bool,
    #[bits(6..=6, rw)]
    pub obj_character_mapping: bool,
    #[bits(7..=7, rw)]
    pub forced_blank: bool,
    #[bits(8..=11, rw)]
    pub enabled_backgrounds: u4,
    #[bits(12..=12, rw)]
    pub obj_display: bool,
    #[bits(13..=13, rw)]
    pub window0_display: bool,
    #[bits(14..=14, rw)]
    pub window1_display: bool,
    #[bits(15..=15, rw)]
    pub obj_window_display: bool,
}

#[bitenum(u1, exhaustive = true)]
#[derive(Default)]
pub(crate) enum BackgroundControlTileFormat {
    #[default]
    FourBpp = 0,
    EightBpp = 1,
}

impl From<TileFormat> for BackgroundControlTileFormat {
    fn from(value: TileFormat) -> Self {
        match value {
            TileFormat::FourBpp => Self::FourBpp,
            TileFormat::EightBpp => Self::EightBpp,
        }
    }
}

#[bitenum(u1, exhaustive = true)]
#[derive(Default)]
pub(crate) enum BackgroundControlAffineOverflowBehaviour {
    #[default]
    Transparent = 0,
    Wraparound = 1,
}

impl From<AffineBackgroundWrapBehaviour> for BackgroundControlAffineOverflowBehaviour {
    fn from(value: AffineBackgroundWrapBehaviour) -> Self {
        match value {
            AffineBackgroundWrapBehaviour::NoWrap => Self::Transparent,
            AffineBackgroundWrapBehaviour::Wrap => Self::Wraparound,
        }
    }
}

#[bitfield(u2)]
#[derive(Default)]
pub(crate) struct BackgroundControlScreenSize {
    #[bits(0..=1, rw)]
    pub value: u2,
}

impl From<RegularBackgroundSize> for BackgroundControlScreenSize {
    fn from(value: RegularBackgroundSize) -> Self {
        Self::builder().with_value(u2::new(value as u8)).build()
    }
}

impl From<AffineBackgroundSize> for BackgroundControlScreenSize {
    fn from(value: AffineBackgroundSize) -> Self {
        Self::builder().with_value(u2::new(value as u8)).build()
    }
}

#[bitenum(u2, exhaustive = true)]
#[derive(Default)]
pub(crate) enum BackgroundControlPriority {
    #[default]
    P0,
    P1,
    P2,
    P3,
}

impl From<Priority> for BackgroundControlPriority {
    fn from(value: Priority) -> Self {
        match value {
            Priority::P0 => Self::P0,
            Priority::P1 => Self::P1,
            Priority::P2 => Self::P2,
            Priority::P3 => Self::P3,
        }
    }
}

#[bitfield(u16)]
#[derive(Default)]
pub(crate) struct BackgroundControlRegister {
    #[bits(0..=1, rw)]
    pub priority: BackgroundControlPriority,
    #[bits(2..=3, rw)]
    pub char_base_block: u2,
    #[bits(6..=6, rw)]
    pub mosaic: bool,
    #[bits(7..=7, rw)]
    pub tile_format: BackgroundControlTileFormat,
    #[bits(8..=12, rw)]
    pub screen_base_block: u5,
    #[bits(13..=13, rw)]
    pub overflow_behaviour: BackgroundControlAffineOverflowBehaviour,
    #[bits(14..=15, rw)]
    pub screen_size: BackgroundControlScreenSize,
}
