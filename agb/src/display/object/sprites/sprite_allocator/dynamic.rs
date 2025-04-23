use core::{alloc::Allocator, ptr::NonNull};

use alloc::boxed::Box;

use crate::display::object::{
    Size,
    sprites::{BYTES_PER_TILE_4BPP, BYTES_PER_TILE_8BPP},
};

use super::{
    LoaderError, PaletteVramMulti, PaletteVramSingle,
    sprite::{SpriteAllocator, SpriteLocation, SpriteVram, SpriteVramInner},
};

macro_rules! dynamic_sprite_defn {
    ($name: ident, $multi: literal, $palette: ty) => {
        /// A mutable dynamic sprite that can be generated at run time
        pub struct $name {
            data: Box<[u16], SpriteAllocator>,
            size: Size,
        }

        // this is explicitly written out so that the extreme alignment conditions are correcly passed to the allocator
        impl Clone for $name {
            fn clone(&self) -> Self {
                let allocation = SpriteAllocator
                    .allocate(self.size.layout($multi))
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
                }
            }
        }

        impl $name {
            /// Attempts to allocate a dynamic sprite returning an error should
            /// there be no more space available for the sprite to be allocated
            /// into.
            pub fn try_new(size: Size) -> Result<Self, LoaderError> {
                let allocation = SpriteAllocator
                    .allocate_zeroed(size.layout($multi))
                    .map_err(|_| LoaderError::SpriteFull)?;

                let allocation = core::ptr::slice_from_raw_parts_mut(
                    allocation.as_ptr() as *mut _,
                    allocation.len() / 2,
                );

                let data = unsafe { Box::from_raw_in(allocation, SpriteAllocator) };

                Ok(Self { data, size })
            }

            #[must_use]
            /// Creates a new dynamic sprite of a given size
            ///
            /// # Panics
            /// Panics if there is no space to allocate sprites into
            pub fn new(size: Size) -> Self {
                Self::try_new(size).expect("couldn't allocate dynamic sprite")
            }

            /// Set the pixel of a sprite to a given colour index from the palette.
            ///
            /// # Panics
            /// Panics if the pixel would be outside the range of the palette
            pub fn set_pixel(&mut self, x: usize, y: usize, paletted_pixel: usize) {
                if !$multi {
                    assert!(paletted_pixel < 16);

                    let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
                    assert!(x < sprite_pixel_x, "x too big for sprite size");
                    assert!(y < sprite_pixel_y, "y too big for sprite size");

                    let (sprite_tile_x, _) = self.size.to_tiles_width_height();

                    let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

                    let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

                    let (x_in_tile, y_in_tile) = (x % 8, y % 8);

                    let half_word_to_modify_in_tile = x_in_tile / 4 + y_in_tile * 2;

                    let half_word_to_modify = tile_number_to_modify * BYTES_PER_TILE_4BPP / 2
                        + half_word_to_modify_in_tile;
                    let mut half_word = self.data[half_word_to_modify];

                    let nibble_to_modify = (x % 4) * 4;

                    half_word = (half_word & !(0b1111 << nibble_to_modify))
                        | ((paletted_pixel as u16) << nibble_to_modify);
                    self.data[half_word_to_modify] = half_word;
                } else {
                    assert!(paletted_pixel < 256);

                    let (sprite_pixel_x, sprite_pixel_y) = self.size.to_width_height();
                    assert!(x < sprite_pixel_x, "x too big for sprite size");
                    assert!(y < sprite_pixel_y, "y too big for sprite size");

                    let (sprite_tile_x, _) = self.size.to_tiles_width_height();

                    let (adjust_tile_x, adjust_tile_y) = (x / 8, y / 8);

                    let tile_number_to_modify = adjust_tile_x + adjust_tile_y * sprite_tile_x;

                    let (x_in_tile, y_in_tile) = (x % 8, y % 8);

                    let half_word_to_modify_in_tile = x_in_tile / 2 + y_in_tile * 2;

                    let half_word_to_modify = tile_number_to_modify * BYTES_PER_TILE_8BPP / 2
                        + half_word_to_modify_in_tile;
                    let mut half_word = self.data[half_word_to_modify];

                    let byte_to_modify = (x % 2) * 8;

                    half_word = (half_word & !(0b11111111 << byte_to_modify))
                        | ((paletted_pixel as u16) << byte_to_modify);
                    self.data[half_word_to_modify] = half_word;
                }
            }

            /// Wipes the sprite clearing it with a specified pixel
            ///
            /// # Panics
            /// Panics if the pixel would be outside the range of the palette
            pub fn clear(&mut self, paletted_pixel: usize) {
                if !$multi {
                    assert!(paletted_pixel < 16);
                    let reset = (paletted_pixel
                        | (paletted_pixel << 4)
                        | (paletted_pixel << 8)
                        | (paletted_pixel << 12)) as u16;
                    self.data.fill(reset);
                } else {
                    assert!(paletted_pixel < 256);
                    let reset = (paletted_pixel | (paletted_pixel << 8)) as u16;
                    self.data.fill(reset);
                }
            }

            #[must_use]
            /// Transforms the sprite to a reference counted immutable sprite usable by objects
            pub fn to_vram(self, palette: $palette) -> SpriteVram {
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

            /// Raw access to the inner data
            pub fn data(&mut self) -> &mut [u16] {
                &mut self.data
            }
        }
    };
}

dynamic_sprite_defn!(DynamicSprite16, false, PaletteVramSingle);
dynamic_sprite_defn!(DynamicSprite256, true, PaletteVramMulti);
