#![warn(missing_docs)]
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
mod managed;
mod sprites;
mod unmanaged;

pub use sprites::{
    include_aseprite, DynamicSprite, Graphics, PaletteVram, Size, Sprite, SpriteLoader, SpriteVram,
    Tag, TagMap,
};

pub use affine::AffineMatrixInstance;
pub use managed::{OamManaged, Object, OrderedStore, OrderedStoreIterator};
pub use unmanaged::{AffineMode, OamIterator, OamSlot, OamUnmanaged, ObjectUnmanaged};

pub use font::{ChangeColour, ObjectTextRender, TextAlignment};

use super::DISPLAY_CONTROL;

const OBJECT_ATTRIBUTE_MEMORY: *mut u16 = 0x0700_0000 as *mut u16;

#[deprecated = "use OamManaged directly instead"]
/// The old name for [`OamManaged`] kept around for easier migration.
/// This will be removed in a future release.
pub type ObjectController<'a> = OamManaged<'a>;

pub(super) unsafe fn initilise_oam() {
    for i in 0..128 {
        let ptr = (OBJECT_ATTRIBUTE_MEMORY).add(i * 4);
        ptr.write_volatile(0b10 << 8);
    }

    DISPLAY_CONTROL.set_bits(1, 1, 0x6);
    DISPLAY_CONTROL.set_bits(1, 1, 0xC);
    DISPLAY_CONTROL.set_bits(0, 1, 0x7);
}
