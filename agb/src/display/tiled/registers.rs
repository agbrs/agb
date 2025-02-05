use bilge::prelude::*;

use crate::display::Priority;

use super::{
    AffineBackgroundSize, AffineBackgroundWrapBehaviour, RegularBackgroundSize, TileFormat,
};

#[bitsize(16)]
#[derive(FromBits)]
pub(crate) struct DisplayControlRegister {
    pub video_mode: u3,
    _reserved: u1,
    _display_frame_select: u1,
    hblank_interval_free: bool,
    obj_character_mapping: bool,
    forced_blank: bool,
    pub enabled_backgrounds: u4,
    obj_display: bool,
    window0_display: bool,
    window1_display: bool,
    obj_window_display: bool,
}

#[bitsize(1)]
#[derive(Clone, Copy, FromBits, Default)]
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

#[bitsize(1)]
#[derive(Clone, Copy, FromBits, Default)]
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

#[bitsize(2)]
#[derive(Clone, Copy, FromBits, Default)]
pub(crate) struct BackgroundControlScreenSize(u2);

impl From<RegularBackgroundSize> for BackgroundControlScreenSize {
    fn from(value: RegularBackgroundSize) -> Self {
        Self::new(u2::new(value as u8))
    }
}

impl From<AffineBackgroundSize> for BackgroundControlScreenSize {
    fn from(value: AffineBackgroundSize) -> Self {
        Self::new(u2::new(value as u8))
    }
}

#[bitsize(2)]
#[derive(Clone, Copy, FromBits, Default)]
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

#[bitsize(16)]
#[derive(Clone, Copy, FromBits, Default)]
pub(crate) struct BackgroundControlRegister {
    pub priority: BackgroundControlPriority,
    pub char_base_block: u2,
    _zero: u2,
    pub mosaic: bool,
    pub tile_format: BackgroundControlTileFormat,
    pub screen_base_block: u5,
    pub overflow_behaviour: BackgroundControlAffineOverflowBehaviour,
    pub screen_size: BackgroundControlScreenSize,
}
