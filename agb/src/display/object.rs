//! # Sprites and objects

mod affine;
mod font;
mod sprites;
mod unmanaged;

pub use sprites::{
    include_aseprite, DynamicSprite, Graphics, IntoSpritePaletteVram, IntoSpriteVram, PaletteMulti,
    PaletteVram, PaletteVramInterface, PaletteVramMulti, PaletteVramSingle, Size, Sprite,
    SpriteVram, Tag, TagMap,
};

pub(crate) use sprites::SPRITE_LOADER;

pub use affine::AffineMatrixInstance;
pub use unmanaged::{AffineMode, GraphicsMode, Object, ObjectAffine};
pub(crate) use unmanaged::{Oam, OamFrame};

pub use font::{ChangeColour, ObjectTextRender, TextAlignment};

use super::DISPLAY_CONTROL;

const OBJECT_ATTRIBUTE_MEMORY: *mut u16 = 0x0700_0000 as *mut u16;

pub(super) unsafe fn initilise_oam() {
    for i in 0..128 {
        let ptr = (OBJECT_ATTRIBUTE_MEMORY).add(i * 4);
        ptr.write_volatile(0b10 << 8);
    }

    DISPLAY_CONTROL.set_bits(1, 1, 0x6);
    DISPLAY_CONTROL.set_bits(1, 1, 0xC);
    DISPLAY_CONTROL.set_bits(0, 1, 0x7);
}
