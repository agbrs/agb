use arbitrary_int::*;
use bitbybit::{bitenum, bitfield};

use crate::display::Priority;

use self::attributes::{
    ObjectAttribute0, ObjectAttribute1Affine, ObjectAttribute1Standard, ObjectAttribute2,
};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct AttributesRegular {
    a0: ObjectAttribute0,
    a1: ObjectAttribute1Standard,
    a2: ObjectAttribute2,
}

impl Default for AttributesRegular {
    fn default() -> Self {
        Self {
            a0: ObjectAttribute0::builder()
                .with_y(0)
                .with_object_mode(ObjectMode::Normal)
                .with_graphics_mode(GraphicsModeInternal::Normal)
                .with_mosaic(false)
                .with_colour_mode(ColourMode::Four)
                .with_shape(u2::new(0u8))
                .build(),
            a1: Default::default(),
            a2: Default::default(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct AttributesAffine {
    a0: ObjectAttribute0,
    a1: ObjectAttribute1Affine,
    a2: ObjectAttribute2,
}

impl AttributesAffine {
    pub fn new(mode: AffineMode) -> Self {
        Self {
            a0: ObjectAttribute0::builder()
                .with_y(0)
                .with_object_mode(match mode {
                    AffineMode::Affine => ObjectMode::Affine,
                    AffineMode::AffineDouble => ObjectMode::AffineDouble,
                })
                .with_graphics_mode(GraphicsModeInternal::Normal)
                .with_mosaic(false)
                .with_colour_mode(ColourMode::Four)
                .with_shape(u2::new(0u8))
                .build(),
            a1: Default::default(),
            a2: Default::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
/// The affine mode
pub enum AffineMode {
    /// Normal affine, this is where the area of the affine is equal to the sprite size
    Affine = 1,
    /// Double affine, this is where the area of the affine is double that of the sprite
    AffineDouble = 3,
}

impl AttributesRegular {
    pub fn write(self, ptr: *mut u16) {
        let attrs = [
            self.a0.raw_value(),
            self.a1.raw_value(),
            self.a2.raw_value(),
        ];

        unsafe {
            ptr.add(0).write_volatile(attrs[0]);
            ptr.add(1).write_volatile(attrs[1]);
            ptr.add(2).write_volatile(attrs[2]);
        }
    }

    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        self.a1.set_horizontal_flip(flip);

        self
    }

    pub fn hflip(self) -> bool {
        self.a1.horizontal_flip()
    }

    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        self.a1.set_vertical_flip(flip);

        self
    }

    pub fn vflip(self) -> bool {
        self.a1.vertical_flip()
    }

    pub fn set_x(&mut self, x: u16) -> &mut Self {
        self.a1.set_x(u9::new(x.rem_euclid(1 << 9)));

        self
    }

    pub fn x(self) -> u16 {
        u16::from(self.a1.x())
    }

    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.a2.set_priority(priority);

        self
    }

    pub fn priority(self) -> Priority {
        self.a2.priority()
    }

    pub fn set_y(&mut self, y: u16) -> &mut Self {
        self.a0.set_y(y as u8);

        self
    }

    pub fn y(self) -> u16 {
        u16::from(self.a0.y())
    }

    pub fn set_palette(&mut self, palette_id: u16) -> &mut Self {
        self.a2.set_palette_bank(u4::new(palette_id as u8));

        self
    }

    pub fn set_sprite(&mut self, sprite_id: u16, shape: u16, size: u16) -> &mut Self {
        self.a2.set_tile_index(u10::new(sprite_id));
        self.a1.set_size(u2::new(size as u8));
        self.a0.set_shape(u2::new(shape as u8));

        self
    }

    pub fn set_colour_mode(&mut self, mode: ColourMode) -> &mut Self {
        self.a0.set_colour_mode(mode);

        self
    }

    pub fn set_graphics_mode(&mut self, mode: GraphicsMode) -> &mut Self {
        self.a0.set_graphics_mode(match mode {
            GraphicsMode::Normal => GraphicsModeInternal::Normal,
            GraphicsMode::AlphaBlending => GraphicsModeInternal::AlphaBlending,
            GraphicsMode::Window => GraphicsModeInternal::Window,
        });

        self
    }
}

impl AttributesAffine {
    pub fn write(self, ptr: *mut u16) {
        let attrs = [
            self.a0.raw_value(),
            self.a1.raw_value(),
            self.a2.raw_value(),
        ];

        unsafe {
            ptr.add(0).write_volatile(attrs[0]);
            ptr.add(1).write_volatile(attrs[1]);
            ptr.add(2).write_volatile(attrs[2]);
        }
    }

    pub fn set_affine_mode(&mut self, affine_mode: AffineMode) -> &mut Self {
        self.a0.set_object_mode(match affine_mode {
            AffineMode::Affine => ObjectMode::Affine,
            AffineMode::AffineDouble => ObjectMode::AffineDouble,
        });

        self
    }

    pub fn set_x(&mut self, x: u16) -> &mut Self {
        self.a1.set_x(u9::new(x.rem_euclid(1 << 9)));

        self
    }

    pub fn x(self) -> u16 {
        u16::from(self.a1.x())
    }

    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.a2.set_priority(priority);

        self
    }

    pub fn priority(self) -> Priority {
        self.a2.priority()
    }

    pub fn set_y(&mut self, y: u16) -> &mut Self {
        self.a0.set_y(y as u8);

        self
    }

    pub fn y(self) -> u16 {
        u16::from(self.a0.y())
    }

    pub fn set_palette(&mut self, palette_id: u16) -> &mut Self {
        self.a2.set_palette_bank(u4::new(palette_id as u8));

        self
    }

    pub fn set_affine_matrix(&mut self, affine_matrix_id: u16) -> &mut Self {
        self.a1.set_affine_index(u5::new(affine_matrix_id as u8));

        self
    }

    pub fn set_sprite(&mut self, sprite_id: u16, shape: u16, size: u16) -> &mut Self {
        self.a2.set_tile_index(u10::new(sprite_id));
        self.a1.set_size(u2::new(size as u8));
        self.a0.set_shape(u2::new(shape as u8));

        self
    }

    pub fn set_colour_mode(&mut self, mode: ColourMode) -> &mut Self {
        self.a0.set_colour_mode(mode);

        self
    }

    pub fn set_graphics_mode(&mut self, mode: GraphicsMode) -> &mut Self {
        self.a0.set_graphics_mode(match mode {
            GraphicsMode::Normal => GraphicsModeInternal::Normal,
            GraphicsMode::AlphaBlending => GraphicsModeInternal::AlphaBlending,
            GraphicsMode::Window => GraphicsModeInternal::Window,
        });

        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// Graphics modes control how it gets rendered
pub enum GraphicsMode {
    #[default]
    /// The sprite rendered as you expect
    Normal,
    /// This object will be alpha blended if the relevant [`Blend`](crate::display::Blend) mode is enabled.
    AlphaBlending,
    /// This object is a mask of the object window
    ///
    /// The object is not rendered at all, and instead any non-transparent pixel is considered part
    /// of the [`object window`](crate::display::Windows::win_obj).
    Window,
}

#[bitenum(u2, exhaustive = true)]
#[derive(Debug, PartialEq, Eq, Default)]
enum ObjectMode {
    Normal,
    Affine,
    #[default]
    Disabled,
    AffineDouble,
}

#[bitenum(u2, exhaustive = false)]
#[derive(Debug, PartialEq, Eq, Default)]
enum GraphicsModeInternal {
    #[default]
    Normal,
    AlphaBlending,
    Window,
}

#[bitenum(u1, exhaustive = true)]
#[derive(Debug, PartialEq, Eq, Default)]
pub enum ColourMode {
    #[default]
    Four,
    Eight,
}

#[allow(clippy::module_inception)]
mod attributes {
    use super::*;

    #[bitfield(u16)]
    #[derive(Debug, PartialEq, Eq, Default)]
    pub(super) struct ObjectAttribute0 {
        #[bits(0..=7, rw)]
        pub y: u8,
        #[bits(8..=9, rw)]
        pub object_mode: ObjectMode,
        #[bits(10..=11, rw)]
        pub graphics_mode: Option<GraphicsModeInternal>,
        #[bit(12, rw)]
        pub mosaic: bool,
        #[bit(13, rw)]
        pub colour_mode: ColourMode,
        #[bits(14..=15, rw)]
        pub shape: u2,
    }

    #[bitfield(u16)]
    #[derive(PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute1Standard {
        #[bits(0..=8, rw)]
        pub x: u9,
        #[bit(12, rw)]
        pub horizontal_flip: bool,
        #[bit(13, rw)]
        pub vertical_flip: bool,
        #[bits(14..=15, rw)]
        pub size: u2,
    }

    #[bitfield(u16)]
    #[derive(PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute1Affine {
        #[bits(0..=8, rw)]
        pub x: u9,
        #[bits(9..=13, rw)]
        pub affine_index: u5,
        #[bits(14..=15, rw)]
        pub size: u2,
    }

    #[bitfield(u16)]
    #[derive(PartialEq, Eq, Debug, Default)]
    pub(super) struct ObjectAttribute2 {
        #[bits(0..=9, rw)]
        pub tile_index: u10,
        #[bits(10..=11, rw)]
        pub priority: Priority,
        #[bits(12..=15, rw)]
        pub palette_bank: u4,
    }
}
