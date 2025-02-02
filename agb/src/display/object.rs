//! # Sprites and objects
//!
//! There are two implementations of objects depending on how you want to make
//! your game. There is the *Managed* and *Unmanaged* systems, given by
//! [OamManaged] and [OamUnmanaged] respectively. The managed Oam is easier to
//! use and has built in support for setting the `z` coordinate. The unmanaged
//! Oam is simpler and more efficient with the tradeoff that it is slightly
//! harder to integrate into your games depending on how they are architectured.

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
pub use unmanaged::{AffineMode, GraphicsMode, Oam, OamFrame, Object};

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
