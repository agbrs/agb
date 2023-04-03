use modular_bitfield::BitfieldSpecifier;

use crate::display::Priority;

use self::attributes::{
    ObjectAttribute0, ObjectAttribute1Affine, ObjectAttribute1Standard, ObjectAttribute2,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Attributes {
    a0: ObjectAttribute0,
    a1s: ObjectAttribute1Standard,
    a1a: ObjectAttribute1Affine,
    a2: ObjectAttribute2,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            a0: ObjectAttribute0::from_bytes([0, 0b10]),
            a1s: Default::default(),
            a1a: Default::default(),
            a2: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AffineMode {
    Affine = 1,
    AffineDouble = 3,
}

impl Attributes {
    pub fn bytes(self) -> [u8; 6] {
        let mode = self.a0.object_mode();
        let attrs = match mode {
            ObjectMode::Normal => [
                self.a0.into_bytes(),
                self.a1s.into_bytes(),
                self.a2.into_bytes(),
            ],
            _ => [
                self.a0.into_bytes(),
                self.a1a.into_bytes(),
                self.a2.into_bytes(),
            ],
        };

        // Safety: length and alignment are the same, and every possible value is valid
        unsafe { core::mem::transmute(attrs) }
    }

    pub fn is_visible(self) -> bool {
        self.a0.object_mode() != ObjectMode::Disabled
    }

    pub fn show(&mut self) -> &mut Self {
        self.a0.set_object_mode(ObjectMode::Normal);

        self
    }

    pub fn show_affine(&mut self, affine_mode: AffineMode) -> &mut Self {
        self.a0.set_object_mode(match affine_mode {
            AffineMode::Affine => ObjectMode::Affine,
            AffineMode::AffineDouble => ObjectMode::AffineDouble,
        });

        self
    }

    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        self.a1s.set_horizontal_flip(flip);

        self
    }

    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        self.a1s.set_vertical_flip(flip);

        self
    }

    pub fn set_x(&mut self, x: u16) -> &mut Self {
        self.a1a.set_x(x.rem_euclid(1 << 9));
        self.a1s.set_x(x.rem_euclid(1 << 9));

        self
    }

    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.a2.set_priority(priority);

        self
    }

    pub fn hide(&mut self) -> &mut Self {
        self.a0.set_object_mode(ObjectMode::Disabled);

        self
    }

    pub fn set_y(&mut self, y: u16) -> &mut Self {
        self.a0.set_y(y as u8);

        self
    }

    pub fn set_palette(&mut self, palette_id: u16) -> &mut Self {
        self.a2.set_palette_bank(palette_id as u8);

        self
    }

    pub fn set_affine_matrix(&mut self, affine_matrix_id: u16) -> &mut Self {
        self.a1a.set_affine_index(affine_matrix_id as u8);

        self
    }

    pub fn set_sprite(&mut self, sprite_id: u16, shape: u16, size: u16) -> &mut Self {
        self.a2.set_tile_index(sprite_id);
        self.a1a.set_size(size as u8);
        self.a1s.set_size(size as u8);
        self.a0.set_shape(shape as u8);

        self
    }
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug, PartialEq, Eq)]
enum ObjectMode {
    Normal,
    Affine,
    Disabled,
    AffineDouble,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug, PartialEq, Eq)]
#[bits = 2]
enum GraphicsMode {
    Normal,
    AlphaBlending,
    Window,
}

#[derive(BitfieldSpecifier, Clone, Copy, Debug, PartialEq, Eq)]
enum ColourMode {
    Four,
    Eight,
}

// this mod is not public, so the internal parts don't need documenting.
#[allow(dead_code)]
#[allow(clippy::all)]
#[allow(clippy::map_unwrap_or)]
mod attributes {
    use modular_bitfield::{
        bitfield,
        specifiers::{B10, B2, B3, B4, B5, B8, B9},
    };

    use crate::display::Priority;

    use super::*;
    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute0 {
        pub y: B8,
        pub object_mode: ObjectMode,
        pub graphics_mode: GraphicsMode,
        pub mosaic: bool,
        pub colour_mode: ColourMode,
        pub shape: B2,
    }

    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute1Standard {
        pub x: B9,
        #[skip]
        __: B3,
        pub horizontal_flip: bool,
        pub vertical_flip: bool,
        pub size: B2,
    }

    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute1Affine {
        pub x: B9,
        pub affine_index: B5,
        pub size: B2,
    }

    #[bitfield]
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute2 {
        pub tile_index: B10,
        pub priority: Priority,
        pub palette_bank: B4,
    }
}
