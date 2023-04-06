use core::{cell::UnsafeCell, marker::PhantomData};

use agb_fixnum::Vector2D;
use alloc::vec::Vec;

use crate::display::{
    object::{
        affine::AffineMatrixVram, sprites::SpriteVram, AffineMatrixInstance,
        OBJECT_ATTRIBUTE_MEMORY,
    },
    Priority,
};

use super::attributes::{AffineMode, Attributes};

#[derive(Default, Debug)]
struct OamFrameModifyables {
    this_frame_sprites: Vec<SpriteVram>,
    frame: u32,
    affine_matrix_count: u32,
}

pub struct OamUnmanaged<'gba> {
    phantom: PhantomData<&'gba ()>,
    frame_data: UnsafeCell<OamFrameModifyables>,
    previous_frame_sprites: Vec<SpriteVram>,
}

pub struct OamIterator<'oam> {
    index: usize,
    frame_data: &'oam UnsafeCell<OamFrameModifyables>,
}

/// A slot in Oam that you can write to. Note that you must call [OamSlot::set]
/// or else it is a bug and will panic when dropped.
pub struct OamSlot<'oam> {
    slot: usize,
    frame_data: &'oam UnsafeCell<OamFrameModifyables>,
}

impl Drop for OamSlot<'_> {
    #[track_caller]
    fn drop(&mut self) {
        panic!("Dropping an OamSlot is a bug in your code. Use the slot by calling set (this consumes the slot) or don't obtain one. See documentation for notes on potential pitfalls.")
    }
}

impl OamSlot<'_> {
    /// Set the slot in OAM to contain the sprite given.
    pub fn set(mut self, object: &ObjectUnmanaged) {
        let mut attributes = object.attributes;
        // SAFETY: This function is not reentrant and we currently hold a mutable borrow of the [UnmanagedOAM].
        let frame_data = unsafe { &mut *self.frame_data.get() };

        Self::handle_affine(&mut attributes, frame_data, object);
        self.set_bytes(attributes.bytes());

        frame_data.this_frame_sprites.push(object.sprite.clone());

        // don't call the drop implementation.
        // okay as none of the fields we have have drop implementations.
        core::mem::forget(self);
    }

    fn handle_affine(
        attributes: &mut Attributes,
        frame_data: &mut OamFrameModifyables,
        object: &ObjectUnmanaged,
    ) {
        if let Some(affine_matrix) = &object.affine_matrix {
            if affine_matrix.frame_count() != frame_data.frame {
                affine_matrix.set_frame_count(frame_data.frame);
                assert!(
                    frame_data.affine_matrix_count <= 32,
                    "too many affine matricies in one frame"
                );
                affine_matrix.set_location(frame_data.affine_matrix_count);
                frame_data.affine_matrix_count += 1;
                affine_matrix.write_to_location(OBJECT_ATTRIBUTE_MEMORY);
            }

            attributes.set_affine_matrix(affine_matrix.location() as u16);
        }
    }

    fn set_bytes(&mut self, bytes: [u8; 6]) {
        unsafe {
            let address = (OBJECT_ATTRIBUTE_MEMORY as *mut u8).add(self.slot * 8);
            address.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
        }
    }
}

impl<'oam> Iterator for OamIterator<'oam> {
    type Item = OamSlot<'oam>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.index;
        self.index += 1;

        if idx >= 128 {
            None
        } else {
            Some(OamSlot {
                slot: idx,
                frame_data: self.frame_data,
            })
        }
    }
}

impl Drop for OamIterator<'_> {
    fn drop(&mut self) {
        let number_writen = self.index;

        for idx in number_writen..128 {
            unsafe {
                let ptr = (OBJECT_ATTRIBUTE_MEMORY as *mut u16).add(idx * 4);
                ptr.write_volatile(0b10 << 8);
            }
        }
    }
}

impl OamUnmanaged<'_> {
    pub fn iter(&mut self) -> OamIterator<'_> {
        let frame_data = self.frame_data.get_mut();
        frame_data.frame = frame_data.frame.wrapping_add(1);
        frame_data.affine_matrix_count = 0;

        // We drain the previous frame sprites here to reuse the Vecs allocation and remove the now unused sprites.
        // Any sprites currently being shown will now be put in the new Vec.
        self.previous_frame_sprites.drain(..);
        core::mem::swap(
            &mut frame_data.this_frame_sprites,
            &mut self.previous_frame_sprites,
        );

        OamIterator {
            index: 0,
            frame_data: &self.frame_data,
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            frame_data: Default::default(),
            phantom: PhantomData,
            previous_frame_sprites: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectUnmanaged {
    attributes: Attributes,
    sprite: SpriteVram,
    affine_matrix: Option<AffineMatrixVram>,
}

impl ObjectUnmanaged {
    #[must_use]
    pub fn new(sprite: SpriteVram) -> Self {
        let sprite_location = sprite.location();
        let palette_location = sprite.palette_location();
        let (shape, size) = sprite.size().shape_size();

        let mut sprite = Self {
            attributes: Attributes::default(),
            sprite,
            affine_matrix: None,
        };

        sprite.attributes.set_sprite(sprite_location, shape, size);
        sprite.attributes.set_palette(palette_location);

        sprite
    }

    #[must_use]
    pub fn is_visible(&self) -> bool {
        self.attributes.is_visible()
    }

    pub fn show(&mut self) -> &mut Self {
        self.attributes.show();

        self
    }

    pub fn show_affine(&mut self, affine_mode: AffineMode) -> &mut Self {
        assert!(
            self.affine_matrix.is_some(),
            "affine matrix must be set before enabling affine matrix!"
        );

        self.attributes.show_affine(affine_mode);

        self
    }

    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        self.attributes.set_hflip(flip);

        self
    }

    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        self.attributes.set_vflip(flip);

        self
    }

    pub fn set_x(&mut self, x: u16) -> &mut Self {
        self.attributes.set_x(x);

        self
    }

    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.attributes.set_priority(priority);

        self
    }

    pub fn hide(&mut self) -> &mut Self {
        self.attributes.hide();

        self
    }

    pub fn set_y(&mut self, y: u16) -> &mut Self {
        self.attributes.set_y(y);

        self
    }

    pub fn set_position(&mut self, position: Vector2D<i32>) -> &mut Self {
        self.set_y(position.y.rem_euclid(1 << 9) as u16);
        self.set_x(position.x.rem_euclid(1 << 9) as u16);

        self
    }

    pub fn set_affine_matrix(&mut self, affine_matrix: AffineMatrixInstance) -> &mut Self {
        let vram = affine_matrix.vram();
        self.affine_matrix = Some(vram);

        self
    }

    fn set_sprite_attributes(&mut self, sprite: &SpriteVram) -> &mut Self {
        let size = sprite.size();
        let (shape, size) = size.shape_size();

        self.attributes.set_sprite(sprite.location(), shape, size);
        self.attributes.set_palette(sprite.palette_location());

        self
    }

    pub fn set_sprite(&mut self, sprite: SpriteVram) -> &mut Self {
        self.set_sprite_attributes(&sprite);

        self.sprite = sprite;

        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        display::object::{Graphics, Tag},
        include_aseprite,
    };

    use super::*;

    #[test_case]
    fn object_usage(gba: &mut crate::Gba) {
        const GRAPHICS: &Graphics = include_aseprite!(
            "../examples/the-purple-night/gfx/objects.aseprite",
            "../examples/the-purple-night/gfx/boss.aseprite"
        );

        const BOSS: &Tag = GRAPHICS.tags().get("Boss");

        let (mut gfx, mut loader) = gba.display.object.get_unmanaged();

        {
            let mut slotter = gfx.iter();

            let slot_a = slotter.next().unwrap();
            let slot_b = slotter.next().unwrap();

            let mut obj = ObjectUnmanaged::new(loader.get_vram_sprite(BOSS.sprite(2)));

            obj.show();

            slot_b.set(&obj);
            slot_a.set(&obj);
        }
    }
}
