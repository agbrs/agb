mod affine;
mod managed;
mod sprites;
mod unmanaged;

pub use sprites::{
    include_aseprite, DynamicSprite, Graphics, Size, Sprite, SpriteVram, StaticSpriteLoader, Tag,
    TagMap,
};

pub use affine::AffineMatrix;
pub use managed::{OAMManager, Object};
pub use unmanaged::{AffineMode, OAMIterator, OAMSlot, UnmanagedOAM, UnmanagedObject};

pub(crate) use affine::init_affine;

use super::DISPLAY_CONTROL;

const OBJECT_ATTRIBUTE_MEMORY: usize = 0x0700_0000;

pub(super) unsafe fn initilise_oam() {
    for i in 0..128 {
        let ptr = (OBJECT_ATTRIBUTE_MEMORY as *mut u16).add(i * 4);
        ptr.write_volatile(0b10 << 8);
    }

    DISPLAY_CONTROL.set_bits(1, 1, 0x6);
    DISPLAY_CONTROL.set_bits(1, 1, 0xC);
    DISPLAY_CONTROL.set_bits(0, 1, 0x7);
}
