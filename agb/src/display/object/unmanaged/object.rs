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

use super::attributes::{AffineMode, Attributes, GraphicsMode};

#[derive(Debug)]
struct OamFrameModifyables {
    this_frame_sprites: Vec<SpriteVram>,
    frame: u32,
    affine_matrix_count: u32,
    previous_index: usize,
}

/// This handles the unmanaged oam system which gives more control to the OAM slots.
/// This is utilised by calling the iter function and writing objects to those slots.
pub struct OamUnmanaged<'gba> {
    phantom: PhantomData<&'gba ()>,
    frame_data: UnsafeCell<OamFrameModifyables>,
    previous_frame_sprites: Vec<SpriteVram>,
}

/// The iterator over the OAM slots. Dropping this will finalise the frame. To
/// use, iterate over and write to each slot.
///
/// For example, it could look like this:
///
/// ```no_run
/// # #![no_main]
/// # #![no_std]
/// use agb::display::object::{OamIterator, ObjectUnmanaged};
///
/// fn write_to_oam(oam_iterator: OamIterator, objects: &[ObjectUnmanaged]) {
///     for (object, slot) in objects.iter().zip(oam_iterator) {
///         slot.set(&object);
///     }
/// }
/// ```
///
/// # Pitfalls
/// You *must* use each OamSlot you obtain, this can be an issue if instead of
/// the above you write
///
/// ```no_run
/// # #![no_main]
/// # #![no_std]
/// use agb::display::object::{OamIterator, ObjectUnmanaged};
///
/// fn write_to_oam(oam_iterator: OamIterator, objects: &[ObjectUnmanaged]) {
///     for (slot, object) in oam_iterator.zip(objects.iter()) {
///         slot.set(&object);
///     }
/// }
/// ```
///
/// This will panic if called because when you run out of objects the zip will
/// have already grabbed the next OamSlot before realising there are no more
/// objects.

pub struct OamIterator<'oam> {
    index: usize,
    frame_data: &'oam UnsafeCell<OamFrameModifyables>,
}

impl<'oam> OamIterator<'oam> {
    /// Sets the next oam slot with the provided `object`.
    ///
    /// Is equivalent to the following:
    /// ```no_run
    /// # #![no_main]
    /// # #![no_std]
    /// # use agb::display::object::{OamIterator, ObjectUnmanaged};
    /// # fn set_next_example(oam_iterator: &mut OamIterator, object: &ObjectUnmanaged) {
    /// if let Some(slot) = oam_iterator.next() {
    ///     slot.set(object);
    /// }
    /// # }
    /// ```
    pub fn set_next(&mut self, object: &ObjectUnmanaged) {
        if let Some(slot) = self.next() {
            slot.set(object);
        }
    }
}

/// A slot in Oam that you can write to. Note that you must call [OamSlot::set]
/// or else it is a bug and will panic when dropped.
///
/// See [`OamIterator`] for potential pitfalls.
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
    #[inline(always)]
    pub fn set(self, object: &ObjectUnmanaged) {
        self.set_inner(object);

        // don't call the drop implementation.
        // okay as none of the fields we have have drop implementations.
        core::mem::forget(self);
    }

    /// By writing these as two separate functions, one inlined and one not, the
    /// compiler doesn't have to copy around the slot structure while still
    /// keeping move semantics. This is slightly faster in benchmarks.
    fn set_inner(&self, object: &ObjectUnmanaged) {
        let mut attributes = object.attributes;
        // SAFETY: This function is not reentrant and we currently hold a mutable borrow of the [UnmanagedOAM].
        let frame_data = unsafe { &mut *self.frame_data.get() };

        if let Some(affine_matrix) = &object.affine_matrix {
            Self::handle_affine(&mut attributes, frame_data, affine_matrix);
        }
        attributes.write(unsafe { OBJECT_ATTRIBUTE_MEMORY.add(self.slot * 4) });

        frame_data.this_frame_sprites.push(object.sprite.clone());
    }

    fn handle_affine(
        attributes: &mut Attributes,
        frame_data: &mut OamFrameModifyables,
        affine_matrix: &AffineMatrixVram,
    ) {
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

impl<'oam> Iterator for OamIterator<'oam> {
    type Item = OamSlot<'oam>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.index;
        if idx == 128 {
            None
        } else {
            self.index += 1;
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
        let last_frame_written = unsafe { &mut (*self.frame_data.get()).previous_index };

        for idx in number_writen..*last_frame_written {
            unsafe {
                let ptr = OBJECT_ATTRIBUTE_MEMORY.add(idx * 4);
                ptr.write_volatile(0b10 << 8);
            }
        }
        *last_frame_written = number_writen;
    }
}

impl OamUnmanaged<'_> {
    /// Returns the OamSlot iterator for this frame.
    pub fn iter(&mut self) -> OamIterator<'_> {
        let frame_data = self.frame_data.get_mut();
        frame_data.frame = frame_data.frame.wrapping_add(1);
        frame_data.affine_matrix_count = 0;

        // We drain the previous frame sprites here to reuse the Vecs allocation and remove the now unused sprites.
        // Any sprites currently being shown will now be put in the new Vec.
        self.previous_frame_sprites.clear();
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
            frame_data: UnsafeCell::new(OamFrameModifyables {
                this_frame_sprites: Vec::new(),
                frame: 0,
                affine_matrix_count: 0,
                previous_index: 0,
            }),
            phantom: PhantomData,
            previous_frame_sprites: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
/// An object to be used by the [`OamUnmanaged`] system. Changes made here are
/// reflected when set to an OamSlot using [`OamSlot::set`].
pub struct ObjectUnmanaged {
    attributes: Attributes,
    sprite: SpriteVram,
    affine_matrix: Option<AffineMatrixVram>,
}

impl ObjectUnmanaged {
    #[must_use]
    /// Creates an unmanaged object from a sprite in vram.
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
    /// Checks whether the object is not marked as hidden. Note that it could be
    /// off screen or completely transparent and still claimed to be visible.
    pub fn is_visible(&self) -> bool {
        self.attributes.is_visible()
    }

    /// Display the sprite in Normal mode.
    pub fn show(&mut self) -> &mut Self {
        self.attributes.show();

        self
    }

    /// Display the sprite in Affine mode.
    pub fn show_affine(&mut self, affine_mode: AffineMode) -> &mut Self {
        assert!(
            self.affine_matrix.is_some(),
            "affine matrix must be set before enabling affine matrix!"
        );

        self.attributes.show_affine(affine_mode);

        self
    }

    /// Sets the horizontal flip, note that this only has a visible affect in Normal mode.  
    /// Use [hflip](Self::hflip) to get the value
    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        self.attributes.set_hflip(flip);

        self
    }

    /// Returns the horizontal flip  
    /// Use [set_hflip](Self::set_hflip) to set the value
    #[must_use]
    pub fn hflip(&self) -> bool {
        self.attributes.hflip()
    }

    /// Sets the vertical flip, note that this only has a visible affect in Normal mode.  
    /// Use [vflip](Self::vflip) to get the value
    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        self.attributes.set_vflip(flip);

        self
    }

    /// Returns the vertical flip  
    /// Use [set_vflip](Self::set_vflip) to set the value
    #[must_use]
    pub fn vflip(&self) -> bool {
        self.attributes.vflip()
    }

    /// Sets the priority of the object relative to the backgrounds priority.  
    /// Use [priority](Self::priority) to get the value
    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        self.attributes.set_priority(priority);

        self
    }

    /// Returns the priority of the object  
    /// Use [set_priority](Self::set_priority) to set the value
    #[must_use]
    pub fn priority(&self) -> Priority {
        self.attributes.priority()
    }

    /// Changes the sprite mode to be hidden, can be changed to Normal or Affine  
    /// modes using [`show`][ObjectUnmanaged::show] and  
    /// [`show_affine`][ObjectUnmanaged::show_affine] respectively.
    pub fn hide(&mut self) -> &mut Self {
        self.attributes.hide();

        self
    }

    /// Sets the x position of the object.  
    /// Use [x](Self::x) to get the value  
    /// Use [set_position](Self::set_position) to set both `x` and `y`
    pub fn set_x(&mut self, x: u16) -> &mut Self {
        self.attributes.set_x(x);

        self
    }

    /// Returns the x position of the object  
    /// Use [set_x](Self::set_x) to set the value
    #[must_use]
    pub fn x(&self) -> u16 {
        self.attributes.x()
    }

    /// Sets the y position of the object.  
    /// Use [y](Self::y) to get the value  
    /// Use [set_position](Self::set_position) to set both `x` and `y`
    pub fn set_y(&mut self, y: u16) -> &mut Self {
        self.attributes.set_y(y);

        self
    }

    /// Returns the y position of the object  
    /// Use [set_y](Self::set_y) to set the value
    #[must_use]
    pub fn y(&self) -> u16 {
        self.attributes.y()
    }

    /// Sets the position of the object.  
    /// Use [position](Self::position) to get the value
    pub fn set_position(&mut self, position: Vector2D<i32>) -> &mut Self {
        self.set_y(position.y.rem_euclid(1 << 9) as u16);
        self.set_x(position.x.rem_euclid(1 << 9) as u16);

        self
    }

    /// Returns the position of the object  
    /// Use [set_position](Self::set_position) to set the value
    #[must_use]
    pub fn position(&self) -> Vector2D<i32> {
        Vector2D::new(self.x() as i32, self.y() as i32)
    }

    /// Sets the affine matrix. This only has an affect in Affine mode.
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

    /// Sets the current sprite for the object.
    pub fn set_sprite(&mut self, sprite: SpriteVram) -> &mut Self {
        self.set_sprite_attributes(&sprite);

        self.sprite = sprite;

        self
    }

    /// Sets the graphics mode of the object
    pub fn set_graphics_mode(&mut self, mode: GraphicsMode) -> &mut Self {
        self.attributes.set_graphics_mode(mode);

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
        static GRAPHICS: &Graphics = include_aseprite!(
            "../examples/the-purple-night/gfx/objects.aseprite",
            "../examples/the-purple-night/gfx/boss.aseprite"
        );

        static BOSS: &Tag = GRAPHICS.tags().get("Boss");

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
