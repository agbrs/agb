use core::marker::PhantomData;

use agb_fixnum::Vector2D;
use alloc::{boxed::Box, vec, vec::Vec};

use crate::display::{
    object::{
        affine::AffineMatrixVram, sprites::SpriteVram, AffineMatrixInstance, IntoSpriteVram,
        OBJECT_ATTRIBUTE_MEMORY,
    },
    Priority,
};

use super::attributes::{AffineMode, Attributes, GraphicsMode};

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
pub struct Oam<'gba> {
    phantom: PhantomData<&'gba ()>,
    previous_frame_sprites: Vec<SpriteVram>,
    frame: Frame,
}

pub struct OamFrame<'oam>(&'oam mut Frame);

impl OamFrame<'_> {
    pub fn show(&mut self, object: &Object) {
        self.set_inner(object);
    }

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

    fn set_inner(&mut self, object: &Object) {
        if self.0.object_count >= 128 {
            return;
        }

        let mut attributes = object.attributes;

        if let Some(affine_matrix) = &object.affine_matrix {
            self.handle_affine(&mut attributes, affine_matrix);
        }
        attributes.write(unsafe { self.0.shadow_oam.as_mut_ptr().add(self.0.object_count * 4) });

        self.0.sprites.push(object.sprite.clone());
        self.0.object_count += 1;
    }

    fn handle_affine(&mut self, attributes: &mut Attributes, affine_matrix: &AffineMatrixVram) {
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
    }
}

impl Oam<'_> {
    /// Returns the OamSlot iterator for this frame.
    pub fn frame(&mut self) -> OamFrame<'_> {
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

#[derive(Debug, Clone)]
/// An object to be used by the [`OamUnmanaged`] system. Changes made here are
/// reflected when set to an OamSlot using [`OamSlot::set`].
pub struct Object {
    attributes: Attributes,
    sprite: SpriteVram,
    affine_matrix: Option<AffineMatrixVram>,
}

impl Object {
    #[must_use]
    /// Creates an unmanaged object from a sprite in vram.
    pub fn new(sprite: impl IntoSpriteVram) -> Self {
        let sprite = sprite.into();

        let sprite_location = sprite.location();
        let (shape, size) = sprite.size().shape_size();
        let palette_location = sprite.single_palette_index();

        let mut sprite = Self {
            attributes: Attributes::default(),
            sprite,
            affine_matrix: None,
        };

        if let Some(palette_location) = palette_location {
            sprite
                .attributes
                .set_palette(palette_location.into())
                .set_colour_mode(super::attributes::ColourMode::Four);
        } else {
            sprite
                .attributes
                .set_colour_mode(super::attributes::ColourMode::Eight);
        }

        sprite
            .attributes
            .set_sprite(sprite_location.idx(), shape, size);

        sprite
    }

    #[must_use]
    /// Checks whether the object is not marked as hidden. Note that it could be
    /// off screen or completely transparent and still claimed to be visible.
    pub fn is_visible(&self) -> bool {
        self.attributes.is_visible()
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
    pub fn set_position(&mut self, position: impl Into<Vector2D<i32>>) -> &mut Self {
        let position = position.into();
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
    pub fn set_sprite(&mut self, sprite: impl IntoSpriteVram) -> &mut Self {
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

        let mut gfx = gba.display.object.get();

        {
            let mut frame = gfx.frame();
            let obj = Object::new(BOSS.sprite(2));

            frame.show(&obj);
            frame.show(&obj);

            frame.commit();
        }
    }
}
