use core::{alloc::Allocator, marker::PhantomData, ptr::NonNull};

use alloc::boxed::Box;

use crate::display::object::{
    Size,
    sprites::{BYTES_PER_TILE_4BPP, BYTES_PER_TILE_8BPP},
};

use super::{
    LoaderError,
    palette::{PaletteVram, PaletteVramMulti, PaletteVramSingle},
    sprite::{SpriteAllocator, SpriteLocation, SpriteVram, SpriteVramInner},
};

/// Sprite data that can be used to create sprites in vram.
pub struct DynamicSprite<PaletteKind: PaletteVramInterface> {
    data: Box<[u16], SpriteAllocator>,
    size: Size,
    palette_kind: PhantomData<PaletteKind>,
}

impl<T: PaletteVramInterface> Clone for DynamicSprite<T> {
    fn clone(&self) -> Self {
        let allocation = SpriteAllocator
            .allocate(self.size.layout(false))
            .expect("cannot allocate dynamic sprite");

        let allocation = core::ptr::slice_from_raw_parts_mut(
            allocation.as_ptr() as *mut _,
            allocation.len() / 2,
        );

        let mut data = unsafe { Box::from_raw_in(allocation, SpriteAllocator) };

        data.clone_from_slice(&self.data);

        Self {
            data,
            size: self.size,
            palette_kind: PhantomData,
        }
    }
}

/// A palette in vram. This trait is sealed and cannot be implemented by the user.
pub trait PaletteVramInterface: private::Sealed {
    /// The maximum value of a pixel
    const PIXEL_SIZE: usize;
    /// The palette in vram
    fn palette(self) -> PaletteVram;
}

mod private {
    use super::{PaletteVramMulti, PaletteVramSingle};

    pub trait Sealed {}
    impl Sealed for PaletteVramSingle {}
    impl Sealed for PaletteVramMulti {}
}

impl PaletteVramInterface for PaletteVramSingle {
    const PIXEL_SIZE: usize = 16;

    fn palette(self) -> PaletteVram {
        self.palette()
    }
}

impl PaletteVramInterface for PaletteVramMulti {
    const PIXEL_SIZE: usize = 256;
    fn palette(self) -> PaletteVram {
        self.palette()
    }
}

impl<T: PaletteVramInterface> DynamicSprite<T> {
    /// Creates a new dynamic sprite of a given size
    pub fn try_new(size: Size) -> Result<Self, LoaderError> {
        let allocation = SpriteAllocator
            .allocate_zeroed(size.layout(false))
            .map_err(|_| LoaderError::SpriteFull)?;

        let allocation = core::ptr::slice_from_raw_parts_mut(
            allocation.as_ptr() as *mut _,
            allocation.len() / 2,
        );

        let data = unsafe { Box::from_raw_in(allocation, SpriteAllocator) };

        Ok(DynamicSprite {
            data,
            size,
            palette_kind: PhantomData,
        })
    }

    #[must_use]
    /// Creates a new dynamic sprite of a given size
    pub fn new(size: Size) -> Self {
        Self::try_new(size).expect("couldn't allocate dynamic sprite")
    }

    /// Set the pixel of a sprite to a given paletted pixel. Panics if the
    /// coordinate is out of range of the sprite or if the paletted pixel is
    /// greater than 4 bits.
    pub fn set_pixel(&mut self, x: usize, y: usize, paletted_pixel: usize) {
        match T::PIXEL_SIZE {
            16 => {
                assert!(paletted_pixel < T::PIXEL_SIZE);

                let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
                assert!(x < sprite_pixel_x, "x too big for sprite size");
                assert!(y < sprite_pixel_y, "y too big for sprite size");

                let (sprite_tile_x, _) = self.size.to_tiles_width_height();

                let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

                let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

                let (x_in_tile, y_in_tile) = (x % 8, y % 8);

                let half_word_to_modify_in_tile = x_in_tile / 4 + y_in_tile * 2;

                let half_word_to_modify =
                    tile_number_to_modify * BYTES_PER_TILE_4BPP / 2 + half_word_to_modify_in_tile;
                let mut half_word = self.data[half_word_to_modify];

                let nibble_to_modify = (x % 4) * 4;

                half_word = (half_word & !(0b1111 << nibble_to_modify))
                    | ((paletted_pixel as u16) << nibble_to_modify);
                self.data[half_word_to_modify] = half_word;
            }
            256 => {
                assert!(paletted_pixel < T::PIXEL_SIZE);

                let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
                assert!(x < sprite_pixel_x, "x too big for sprite size");
                assert!(y < sprite_pixel_y, "y too big for sprite size");

                let (sprite_tile_x, _) = self.size.to_tiles_width_height();

                let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

                let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

                let (x_in_tile, y_in_tile) = (x % 8, y % 8);

                let half_word_to_modify_in_tile = x_in_tile / 2 + y_in_tile * 2;

                let half_word_to_modify =
                    tile_number_to_modify * BYTES_PER_TILE_8BPP / 2 + half_word_to_modify_in_tile;
                let mut half_word = self.data[half_word_to_modify];

                let byte_to_modify = (x % 2) * 8;

                half_word = (half_word & !(0b11111111 << byte_to_modify))
                    | ((paletted_pixel as u16) << byte_to_modify);
                self.data[half_word_to_modify] = half_word;
            }
            _ => unreachable!(),
        }
    }

    /// Wipes the sprite
    pub fn clear(&mut self, paletted_pixel: usize) {
        assert!(paletted_pixel < 0x10);
        let reset = (paletted_pixel
            | (paletted_pixel << 4)
            | (paletted_pixel << 8)
            | (paletted_pixel << 12)) as u16;
        self.data.fill(reset);
    }

    #[must_use]
    /// Tries to copy the sprite to vram to be used to set object sprites.
    /// Panics if it cannot be allocated.
    pub fn to_vram(self, palette: T) -> SpriteVram {
        let data = unsafe { NonNull::new_unchecked(Box::leak(self.data).as_mut_ptr()) };

        let palette = palette.palette();

        let inner = unsafe {
            SpriteVramInner::new_from_allocated(
                SpriteLocation::from_ptr(data.cast()),
                self.size,
                palette.is_multi(),
            )
        };
        SpriteVram::new(inner, palette)
    }
}
