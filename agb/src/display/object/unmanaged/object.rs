use core::marker::PhantomData;

use agb_fixnum::Vector2D;
use alloc::{boxed::Box, vec, vec::Vec};

use crate::display::{
    GraphicsFrame, Priority,
    object::{
        AffineMatrixInstance, OBJECT_ATTRIBUTE_MEMORY, affine::AffineMatrixVram,
        sprites::SpriteVram,
    },
};

use super::attributes::{AffineMode, AttributesAffine, AttributesRegular, GraphicsMode};

struct Frame {
    sprites: Vec<SpriteVram>,
    shadow_oam: Box<[u16]>,
    object_count: usize,
    frame_count: u32,
    affine_matrix_count: u32,
}

impl Frame {
    fn new() -> Self {
        Self {
            sprites: Vec::new(),
            shadow_oam: (vec![0u16; 128 * 4]).into(),
            object_count: 0,
            frame_count: 0,
            affine_matrix_count: 0,
        }
    }
}

/// This handles the unmanaged oam system which gives more control to the OAM slots.
/// This is utilised by calling the iter function and writing objects to those slots.
pub(crate) struct Oam<'gba> {
    phantom: PhantomData<&'gba ()>,
    previous_frame_sprites: Vec<SpriteVram>,
    frame: Frame,
}

pub(crate) struct OamFrame<'oam>(&'oam mut Frame);

impl OamFrame<'_> {
    pub fn commit(self) {
        // get the maximum of sprites and affine matrices to copy as little as possible
        let copy_count = self
            .0
            .object_count
            .max(self.0.affine_matrix_count as usize * 4);

        unsafe {
            OBJECT_ATTRIBUTE_MEMORY
                .copy_from_nonoverlapping(self.0.shadow_oam.as_mut_ptr(), copy_count * 4);
        }
        for idx in self.0.object_count..128 {
            unsafe {
                OBJECT_ATTRIBUTE_MEMORY
                    .add(idx * 4)
                    .write_volatile(0b10 << 8);
            }
        }
    }

    fn show_regular(&mut self, object: &Object) {
        if self.0.object_count >= 128 {
            return;
        }

        object
            .attributes
            .write(unsafe { self.0.shadow_oam.as_mut_ptr().add(self.0.object_count * 4) });

        self.0.sprites.push(object.sprite.clone());
        self.0.object_count += 1;
    }

    fn show_affine(&mut self, object: &ObjectAffine) {
        if self.0.object_count >= 128 {
            return;
        }

        let mut attributes = object.attributes;
        let affine_matrix = &object.matrix;

        if affine_matrix.frame_count() != self.0.frame_count {
            affine_matrix.set_frame_count(self.0.frame_count);
            assert!(
                self.0.affine_matrix_count <= 32,
                "too many affine matrices in one frame"
            );
            affine_matrix.set_location(self.0.affine_matrix_count);
            self.0.affine_matrix_count += 1;
            affine_matrix.write_to_location(self.0.shadow_oam.as_mut_ptr());
        }

        attributes.set_affine_matrix(affine_matrix.location() as u16);

        attributes.write(unsafe { self.0.shadow_oam.as_mut_ptr().add(self.0.object_count * 4) });

        self.0.sprites.push(object.sprite.clone());
        self.0.object_count += 1;
    }
}

impl Oam<'_> {
    /// Returns the OamSlot iterator for this frame.
    pub(crate) fn frame(&mut self) -> OamFrame<'_> {
        self.frame.frame_count = self.frame.frame_count.wrapping_add(1);
        self.frame.affine_matrix_count = 0;
        self.frame.object_count = 0;

        core::mem::swap(&mut self.frame.sprites, &mut self.previous_frame_sprites);
        self.frame.sprites.clear();

        OamFrame(&mut self.frame)
    }

    pub(crate) fn new() -> Self {
        Self {
            frame: Frame::new(),
            phantom: PhantomData,
            previous_frame_sprites: Default::default(),
        }
    }
}

/// An object that can be shown on the screen
#[derive(Debug, Clone)]
pub struct Object {
    attributes: AttributesRegular,
    sprite: SpriteVram,
}

impl Object {
    /// Show the object on the current frame
    pub fn show(&self, frame: &mut GraphicsFrame) {
        frame.oam_frame.show_regular(self);
    }

    #[must_use]
    /// Creates an unmanaged object from a sprite in vram.
    pub fn new(sprite: impl Into<SpriteVram>) -> Self {
        fn new(sprite: SpriteVram) -> Object {
            let sprite_location = sprite.location();
            let (shape, size) = sprite.size().shape_size();
            let palette_location = sprite.single_palette_index();

            let mut object = Object {
                attributes: AttributesRegular::default(),
                sprite,
            };

            if let Some(palette_location) = palette_location {
                object
                    .attributes
                    .set_palette(palette_location.into())
                    .set_colour_mode(super::attributes::ColourMode::Four);
            } else {
                object
                    .attributes
                    .set_colour_mode(super::attributes::ColourMode::Eight);
            }

            object
                .attributes
                .set_sprite(sprite_location.idx(), shape, size);

            object
        }

        let sprite = sprite.into();

        new(sprite)
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

    /// Sets the position of the object.  
    /// Use [position](Self::position) to get the value
    pub fn set_position(&mut self, position: impl Into<Vector2D<i32>>) -> &mut Self {
        let position = position.into();
        self.attributes.set_y(position.y.rem_euclid(1 << 9) as u16);
        self.attributes.set_x(position.x.rem_euclid(1 << 9) as u16);

        self
    }

    /// Returns the position of the object  
    /// Use [set_position](Self::set_position) to set the value
    #[must_use]
    pub fn position(&self) -> Vector2D<i32> {
        Vector2D::new(self.attributes.x() as i32, self.attributes.y() as i32)
    }

    fn set_sprite_attributes(&mut self, sprite: &SpriteVram) -> &mut Self {
        let size = sprite.size();
        let (shape, size) = size.shape_size();

        self.attributes
            .set_sprite(sprite.location().idx(), shape, size);
        if let Some(palette_location) = sprite.single_palette_index() {
            self.attributes
                .set_palette(palette_location.into())
                .set_colour_mode(super::attributes::ColourMode::Four);
        } else {
            self.attributes
                .set_colour_mode(super::attributes::ColourMode::Eight);
        }
        self
    }

    /// Sets the current sprite for the object.
    pub fn set_sprite(&mut self, sprite: impl Into<SpriteVram>) -> &mut Self {
        let sprite = sprite.into();
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

/// An affine object, an object that can be transformed by an affine matrix (scaled, rotated, etc.).
pub struct ObjectAffine {
    attributes: AttributesAffine,
    sprite: SpriteVram,
    matrix: AffineMatrixVram,
}

impl ObjectAffine {
    /// Show the affine object on the screen
    pub fn show(&self, frame: &mut GraphicsFrame) {
        frame.oam_frame.show_affine(self);
    }

    #[must_use]
    /// Creates an unmanaged object from a sprite in vram.
    pub fn new(
        sprite: impl Into<SpriteVram>,
        affine_matrix: AffineMatrixInstance,
        affine_mode: AffineMode,
    ) -> Self {
        fn new(
            sprite: SpriteVram,
            affine_matrix: AffineMatrixInstance,
            affine_mode: AffineMode,
        ) -> ObjectAffine {
            let sprite_location = sprite.location();
            let (shape, size) = sprite.size().shape_size();
            let palette_location = sprite.single_palette_index();

            let mut object = ObjectAffine {
                attributes: AttributesAffine::new(affine_mode),
                sprite,
                matrix: affine_matrix.vram(),
            };

            if let Some(palette_location) = palette_location {
                object
                    .attributes
                    .set_palette(palette_location.into())
                    .set_colour_mode(super::attributes::ColourMode::Four);
            } else {
                object
                    .attributes
                    .set_colour_mode(super::attributes::ColourMode::Eight);
            }

            object
                .attributes
                .set_sprite(sprite_location.idx(), shape, size);

            object
        }
        let sprite = sprite.into();

        new(sprite, affine_matrix, affine_mode)
    }

    /// Sets the affine mode
    pub fn set_affine_mode(&mut self, mode: AffineMode) -> &mut Self {
        self.attributes.set_affine_mode(mode);

        self
    }

    /// Sets the affine matrix to an instance of a matrix
    pub fn set_affine_matrix(&mut self, matrix: AffineMatrixInstance) -> &mut Self {
        self.matrix = matrix.vram();

        self
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

    /// Sets the position of the object.  
    /// Use [position](Self::position) to get the value
    pub fn set_position(&mut self, position: impl Into<Vector2D<i32>>) -> &mut Self {
        let position = position.into();
        self.attributes.set_y(position.y.rem_euclid(1 << 9) as u16);
        self.attributes.set_x(position.x.rem_euclid(1 << 9) as u16);

        self
    }

    /// Returns the position of the object  
    /// Use [set_position](Self::set_position) to set the value
    #[must_use]
    pub fn position(&self) -> Vector2D<i32> {
        Vector2D::new(self.attributes.x() as i32, self.attributes.y() as i32)
    }

    fn set_sprite_attributes(&mut self, sprite: &SpriteVram) -> &mut Self {
        let size = sprite.size();
        let (shape, size) = size.shape_size();

        self.attributes
            .set_sprite(sprite.location().idx(), shape, size);
        if let Some(palette_location) = sprite.single_palette_index() {
            self.attributes
                .set_palette(palette_location.into())
                .set_colour_mode(super::attributes::ColourMode::Four);
        } else {
            self.attributes
                .set_colour_mode(super::attributes::ColourMode::Eight);
        }
        self
    }

    /// Sets the current sprite for the object.
    pub fn set_sprite(&mut self, sprite: impl Into<SpriteVram>) -> &mut Self {
        let sprite = sprite.into();
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
    use crate::include_aseprite;

    use super::*;

    #[test_case]
    fn object_usage(gba: &mut crate::Gba) {
        include_aseprite!(
            mod sprites,
            "../examples/the-purple-night/gfx/objects.aseprite",
            "../examples/the-purple-night/gfx/boss.aseprite"
        );

        let mut gfx = gba.display.graphics.get();

        {
            let mut frame = gfx.frame();
            let obj = Object::new(sprites::BOSS.sprite(2));

            obj.show(&mut frame);
            obj.show(&mut frame);

            frame.commit();
        }
    }
}
