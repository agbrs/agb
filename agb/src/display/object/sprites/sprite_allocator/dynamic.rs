use core::{
    alloc::{Allocator, Layout},
    ptr::NonNull,
};

use crate::{InternalAllocator, display::object::PaletteVramMulti};

use alloc::{alloc::AllocError, boxed::Box};

use crate::display::object::{
    Size,
    sprites::{
        BYTES_PER_TILE_4BPP, BYTES_PER_TILE_8BPP, sprite_allocator::garbage_collect_sprite_loader,
    },
};

use super::{
    PaletteVramSingle,
    sprite::{SpriteAllocator, SpriteLocation, SpriteVram, SpriteVramInner},
};

fn allocate_with_retry(layout: Layout) -> Result<NonNull<[u8]>, alloc::alloc::AllocError> {
    if let Ok(x) = SpriteAllocator.allocate(layout) {
        return Ok(x);
    }
    unsafe {
        garbage_collect_sprite_loader();
    }

    SpriteAllocator.allocate(layout)
}

/// A mutable dynamic sprite buffer that can be generated at run time
#[derive(Clone)]
pub struct DynamicSprite16<A: Allocator = InternalAllocator> {
    data: Box<[u32], A>,
    size: Size,
}

impl<A: Allocator> DynamicSprite16<A> {
    fn allocation_size(size: Size) -> usize {
        size.size_bytes_16()
    }

    fn layout(&self) -> Layout {
        self.size.layout(false)
    }

    /// Set the pixel of a sprite to a given colour index from the palette.
    ///
    /// # Panics
    /// Panics if the pixel would be outside the range of the palette
    /// or the coordinate is outside the sprite.
    pub fn set_pixel(&mut self, x: usize, y: usize, paletted_pixel: u8) {
        assert!(paletted_pixel < 16);

        let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
        assert!(x < sprite_pixel_x, "x too big for sprite size");
        assert!(y < sprite_pixel_y, "y too big for sprite size");

        let (sprite_tile_x, _) = self.size.to_tiles_width_height();

        let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

        let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

        let (x_in_tile, y_in_tile) = (x % 8, y % 8);

        let byte_to_modify_in_tile = x_in_tile / 2 + y_in_tile * 4;

        let byte_to_modify = tile_number_to_modify * BYTES_PER_TILE_4BPP + byte_to_modify_in_tile;
        let mut byte = self.data()[byte_to_modify];

        let nibble_to_modify = (x % 2) * 4;

        byte = (byte & !(0b1111 << nibble_to_modify)) | (paletted_pixel << nibble_to_modify);
        self.data_mut()[byte_to_modify] = byte;
    }

    /// Copies the sprite data to sprite vram
    pub fn try_to_vram(
        &self,
        palette: impl Into<PaletteVramSingle>,
    ) -> Result<SpriteVram, AllocError> {
        let data = allocate_with_retry(self.layout())?;

        unsafe {
            let dest = data.cast().as_ptr();
            crate::agbabi::memcpy_16(self.data.as_ptr().cast(), dest, self.data.len() * 2);
        }

        let palette = palette.into().palette();

        let inner = unsafe {
            SpriteVramInner::new_from_allocated(
                SpriteLocation::from_ptr(data.cast()),
                self.size,
                palette.is_multi(),
            )
        };
        Ok(SpriteVram::new(inner, palette))
    }

    /// Copies the sprite data to sprite vram
    pub fn to_vram(&self, palette: impl Into<PaletteVramSingle>) -> SpriteVram {
        self.try_to_vram(palette)
            .expect("should be able to allocate sprite buffer")
    }

    /// Wipes the sprite clearing it with a specified pixel
    ///
    /// # Panics
    /// Panics if the pixel would be outside the range of the palette
    pub fn clear(&mut self, paletted_pixel: usize) {
        assert!(paletted_pixel < 16);
        let reset = (paletted_pixel | (paletted_pixel << 4)) as u8;
        self.data_mut().fill(reset);
    }
}

/// A mutable dynamic sprite buffer that can be generated at run time
#[derive(Clone)]
pub struct DynamicSprite256<A: Allocator = InternalAllocator> {
    data: Box<[u32], A>,
    size: Size,
}

impl<A: Allocator> DynamicSprite256<A> {
    fn allocation_size(size: Size) -> usize {
        size.size_bytes_256()
    }

    fn layout(&self) -> Layout {
        self.size.layout(true)
    }

    /// Set the pixel of a sprite to a given colour index from the palette.
    ///
    /// # Panics
    /// Panics if the pixel would be outside the range of the palette
    /// or the coordinate is outside the sprite.
    pub fn set_pixel(&mut self, x: usize, y: usize, paletted_pixel: u8) {
        let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
        assert!(x < sprite_pixel_x, "x too big for sprite size");
        assert!(y < sprite_pixel_y, "y too big for sprite size");

        let (sprite_tile_x, _) = self.size.to_tiles_width_height();

        let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

        let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

        let (x_in_tile, y_in_tile) = (x % 8, y % 8);

        let byte_to_modify_in_tile = x_in_tile + y_in_tile * 8;

        let byte_to_modify = tile_number_to_modify * BYTES_PER_TILE_8BPP + byte_to_modify_in_tile;

        self.data_mut()[byte_to_modify] = paletted_pixel;
    }

    /// Copies the sprite data to sprite vram
    pub fn try_to_vram(
        &self,
        palette: impl Into<PaletteVramMulti>,
    ) -> Result<SpriteVram, AllocError> {
        let data = allocate_with_retry(self.layout())?;

        unsafe {
            let dest = data.cast().as_ptr();
            crate::agbabi::memcpy_16(self.data.as_ptr().cast(), dest, self.data.len() * 2);
        }

        let palette = palette.into().palette();

        let inner = unsafe {
            SpriteVramInner::new_from_allocated(
                SpriteLocation::from_ptr(data.cast()),
                self.size,
                palette.is_multi(),
            )
        };
        Ok(SpriteVram::new(inner, palette))
    }

    /// Copies the sprite data to sprite vram
    pub fn to_vram(&self, palette: impl Into<PaletteVramMulti>) -> SpriteVram {
        self.try_to_vram(palette)
            .expect("should be able to allocate sprite buffer")
    }

    /// Wipes the sprite clearing it with a specified pixel
    ///
    /// # Panics
    /// Panics if the pixel would be outside the range of the palette
    pub fn clear(&mut self, paletted_pixel: usize) {
        assert!(paletted_pixel < 256);
        self.data_mut().fill(paletted_pixel as u8);
    }
}

macro_rules! common_impls {
    ($name: ident) => {
        impl $name {
            /// Creates a new sprite buffer in iwram
            pub fn try_new(size: Size) -> Result<Self, AllocError> {
                Self::try_new_in(size, InternalAllocator)
            }

            /// Creates a new sprite buffer in iwram
            pub fn new(size: Size) -> Self {
                Self::new_in(size, InternalAllocator)
            }

            /// Copies data from the byte buffer into a new allocation
            pub fn from_bytes(size: Size, bytes: &[u8]) -> Self {
                Self::from_bytes_in(size, bytes, InternalAllocator)
            }

            /// Copies data from the byte buffer into a new allocation
            pub fn try_from_bytes(size: Size, bytes: &[u8]) -> Result<Self, AllocError> {
                Self::try_from_bytes_in(size, bytes, InternalAllocator)
            }
        }

        impl<A: Allocator> $name<A> {
            /// Creates a new sprite buffer in the given allocator
            pub fn try_new_in(size: Size, allocator: A) -> Result<Self, AllocError> {
                let data =
                    Box::try_new_zeroed_slice_in(Self::allocation_size(size) / 4, allocator)?;
                let data = unsafe { data.assume_init() };

                Ok(Self { data, size })
            }

            /// Copies data from the byte buffer into a new allocation
            pub fn try_from_bytes_in(
                size: Size,
                bytes: &[u8],
                allocator: A,
            ) -> Result<Self, AllocError> {
                let allocation_size = Self::allocation_size(size);
                assert_eq!(
                    bytes.len(),
                    allocation_size,
                    "buffer length should match sprite size"
                );

                let mut data =
                    Box::<[u32], A>::try_new_uninit_slice_in(allocation_size / 4, allocator)?;

                let data = unsafe {
                    // cast the data ptr to a u32 ptr and memcpy
                    let raw = data.as_mut_ptr() as *mut u32;
                    let raw = raw as *mut u8;
                    core::ptr::copy_nonoverlapping(bytes.as_ptr(), raw, allocation_size);
                    data.assume_init()
                };

                Ok(Self { data, size })
            }

            /// Copies data from the byte buffer into a new allocation
            pub fn from_bytes_in(size: Size, bytes: &[u8], allocator: A) -> Self {
                Self::try_from_bytes_in(size, bytes, allocator)
                    .expect("should be able to allocate sprite buffer")
            }

            /// Creates a new sprite buffer in the given allocator
            pub fn new_in(size: Size, allocator: A) -> Self {
                Self::try_new_in(size, allocator).expect("should be able to allocate sprite buffer")
            }

            /// Creates a copy of the sprite data, this can potentially be in another allocator.
            pub fn try_clone_in<B: Allocator>(&self, allocator: B) -> Result<$name<B>, AllocError> {
                let mut data =
                    Box::<[u32], B>::try_new_uninit_slice_in(self.data.len(), allocator)?;

                let data = unsafe {
                    // cast the data ptr to a u32 ptr and memcpy
                    let raw = data.as_mut_ptr() as *mut u32;
                    core::ptr::copy_nonoverlapping(self.data.as_ptr(), raw, self.data.len());
                    data.assume_init()
                };

                Ok($name {
                    data,
                    size: self.size,
                })
            }

            /// Creates a copy of the sprite data, this can potentially be in another allocator.
            pub fn clone_in<B: Allocator>(&self, allocator: B) -> $name<B> {
                self.try_clone_in(allocator)
                    .expect("should be able to allocate sprite buffer")
            }

            /// Access the underlying sprite buffer as a byte slice.
            /// The data is guaranteed to be aligned to a 4 byte boundary.
            pub fn data(&self) -> &[u8] {
                unsafe {
                    let raw = self.data.as_ptr();
                    core::slice::from_raw_parts(raw.cast(), self.data.len() * 4)
                }
            }

            /// Access the underlying sprite buffer as a mutable byte slice.
            /// The data is guaranteed to be aligned to a 4 byte boundary.
            pub fn data_mut(&mut self) -> &mut [u8] {
                unsafe {
                    let raw = self.data.as_mut_ptr();
                    core::slice::from_raw_parts_mut(raw.cast(), self.data.len() * 4)
                }
            }
        }
    };
}

common_impls!(DynamicSprite16);
common_impls!(DynamicSprite256);

#[cfg(test)]
mod tests {
    use crate::{
        display::{
            HEIGHT, Palette16, Rgb, Rgb15, WIDTH,
            object::{DynamicSprite16, DynamicSprite256, Object, PaletteMulti, Size},
            tiled::VRAM_MANAGER,
        },
        test_runner::assert_image_output,
    };

    #[test_case]
    fn check_dynamic_sprite_16(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();
        let mut frame = gfx.frame();

        VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb::new(0xff, 0, 0xff).to_rgb15());

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        let mut sprite = DynamicSprite16::new(Size::S8x8);

        sprite.set_pixel(2, 2, 1);
        sprite.set_pixel(6, 2, 1);

        sprite.set_pixel(1, 6, 1);
        sprite.set_pixel(2, 7, 1);
        sprite.set_pixel(3, 7, 1);
        sprite.set_pixel(4, 7, 1);
        sprite.set_pixel(5, 7, 1);
        sprite.set_pixel(6, 7, 1);
        sprite.set_pixel(7, 6, 1);

        let sprite = sprite.to_vram(&PALETTE);

        Object::new(sprite)
            .set_pos((WIDTH / 2 - 4, HEIGHT / 2 - 4))
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/object/dynamic_sprite.png");
    }

    #[test_case]
    fn check_dynamic_sprite_256(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();
        let mut frame = gfx.frame();

        VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb::new(0xff, 0, 0xff).to_rgb15());

        static PALETTE: PaletteMulti = const {
            static PALETTE: &[Palette16] = &[const {
                let palette = [Rgb15::WHITE; 16];
                Palette16::new(palette)
            }];
            PaletteMulti::new(PALETTE)
        };

        let mut sprite = DynamicSprite256::new(Size::S8x8);
        let colour = PALETTE.first_colour_index();

        sprite.set_pixel(2, 2, colour);
        sprite.set_pixel(6, 2, colour);

        sprite.set_pixel(1, 6, colour);
        sprite.set_pixel(2, 7, colour);
        sprite.set_pixel(3, 7, colour);
        sprite.set_pixel(4, 7, colour);
        sprite.set_pixel(5, 7, colour);
        sprite.set_pixel(6, 7, colour);
        sprite.set_pixel(7, 6, colour);

        let sprite = sprite.to_vram(&PALETTE);

        Object::new(sprite)
            .set_pos((WIDTH / 2 - 4, HEIGHT / 2 - 4))
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/object/dynamic_sprite_256.png");
    }

    #[test_case]
    fn check_dynamic_sprite_copy(gba: &mut crate::Gba) {
        let mut gfx = gba.graphics.get();
        let mut frame = gfx.frame();

        VRAM_MANAGER.set_background_palette_colour(0, 0, Rgb::new(0xff, 0, 0xff).to_rgb15());

        static PALETTE: Palette16 = const {
            let mut palette = [Rgb15::BLACK; 16];
            palette[1] = Rgb15::WHITE;
            palette[2] = Rgb15(0x10_7C);
            Palette16::new(palette)
        };

        let mut sprite = DynamicSprite16::new(Size::S8x8);

        sprite.set_pixel(2, 2, 1);
        sprite.set_pixel(6, 2, 1);

        sprite.set_pixel(1, 6, 1);
        sprite.set_pixel(2, 7, 1);
        sprite.set_pixel(3, 7, 1);
        sprite.set_pixel(4, 7, 1);
        sprite.set_pixel(5, 7, 1);
        sprite.set_pixel(6, 7, 1);
        sprite.set_pixel(7, 6, 1);

        let copy = sprite.clone();
        let sprite = copy.to_vram(&PALETTE);

        Object::new(sprite)
            .set_pos((WIDTH / 2 - 4, HEIGHT / 2 - 4))
            .show(&mut frame);

        frame.commit();

        assert_image_output("gfx/test_output/object/dynamic_sprite_copy.png");
    }
}
