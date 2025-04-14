//! # Sprites and objects
#![deny(missing_docs)]

mod affine;
mod sprites;
mod unmanaged;

pub use sprites::{
    AnimationIterator, DynamicSprite16, DynamicSprite256, PaletteMulti, PaletteVram,
    PaletteVramMulti, PaletteVramSingle, Size, Sprite, SpriteVram, Tag, include_aseprite,
};

pub(crate) use sprites::SPRITE_LOADER;

pub use affine::AffineMatrixInstance;
pub use unmanaged::{AffineMode, GraphicsMode, Object, ObjectAffine};
pub(crate) use unmanaged::{Oam, OamFrame};

use super::DISPLAY_CONTROL;

const OBJECT_ATTRIBUTE_MEMORY: *mut u16 = 0x0700_0000 as *mut u16;

pub(super) unsafe fn initilise_oam() {
    for i in 0..128 {
        unsafe {
            let ptr = (OBJECT_ATTRIBUTE_MEMORY).add(i * 4);
            ptr.write_volatile(0b10 << 8);
        }
    }

    let mut display_control = DISPLAY_CONTROL.get();
    display_control.set_obj_character_mapping(true);
    display_control.set_obj_display(true);

    DISPLAY_CONTROL.set(display_control);
}
