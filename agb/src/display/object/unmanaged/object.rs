use core::{cell::UnsafeCell, marker::PhantomData};

use agb_fixnum::Vector2D;

use crate::display::{
    object::{sprites::SpriteVram, OBJECT_ATTRIBUTE_MEMORY},
    Priority,
};

use super::attributes::{AffineMode, Attributes};

pub struct UnmanagedOAM<'gba> {
    phantom: PhantomData<&'gba ()>,
}

pub struct OAMIterator<'oam> {
    phantom: PhantomData<&'oam ()>,
    index: usize,
}

pub struct OAMSlot<'oam> {
    phantom: PhantomData<&'oam ()>,
    slot: usize,
}

impl OAMSlot<'_> {
    pub fn set(&mut self, object: &UnmanagedObject) {
        self.set_bytes(object.attributes.bytes());

        // SAFETY: This is called here and in set_sprite, neither of which call the other.
        let sprites = unsafe { &mut *object.sprites.get() };

        sprites.previous_sprite = Some(sprites.sprite.clone());
    }

    fn set_bytes(&mut self, bytes: [u8; 6]) {
        unsafe {
            let address = (OBJECT_ATTRIBUTE_MEMORY as *mut u8).add(self.slot * 8);
            address.copy_from_nonoverlapping(bytes.as_ptr(), bytes.len());
        }
    }
}

impl<'oam> Iterator for OAMIterator<'oam> {
    type Item = OAMSlot<'oam>;

    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.index;
        self.index += 1;

        if idx >= 128 {
            None
        } else {
            Some(OAMSlot {
                phantom: PhantomData,
                slot: idx,
            })
        }
    }
}

impl UnmanagedOAM<'_> {
    pub fn iter(&mut self) -> OAMIterator<'_> {
        OAMIterator {
            phantom: PhantomData,
            index: 0,
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    pub fn clear_from(&self, from: usize) {
        if from >= 128 {
            return;
        }

        for i in from..128 {
            unsafe {
                let ptr = (OBJECT_ATTRIBUTE_MEMORY as *mut u16).add(i * 4);
                ptr.write_volatile(0b10 << 8);
            }
        }
    }
}

#[derive(Debug)]
struct VramSprites {
    sprite: SpriteVram,
    previous_sprite: Option<SpriteVram>,
}

#[derive(Debug)]
pub struct UnmanagedObject {
    attributes: Attributes,
    sprites: UnsafeCell<VramSprites>,
}

impl UnmanagedObject {
    #[must_use]
    pub fn new(sprite: SpriteVram) -> Self {
        let sprite_location = sprite.location();
        let palette_location = sprite.palette_location();
        let (shape, size) = sprite.size().shape_size();

        let mut sprite = Self {
            attributes: Attributes::default(),
            sprites: UnsafeCell::new(VramSprites {
                sprite,
                previous_sprite: None,
            }),
        };

        sprite.attributes.set_sprite(sprite_location, shape, size);
        sprite.attributes.set_palette(palette_location);

        sprite
    }

    pub fn is_visible(&self) -> bool {
        self.attributes.is_visible()
    }

    pub fn show(&mut self) -> &mut Self {
        self.attributes.show();

        self
    }

    pub fn show_affine(&mut self, affine_mode: AffineMode) -> &mut Self {
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

    fn set_sprite_attributes(&mut self, sprite: &SpriteVram) -> &mut Self {
        let size = sprite.size();
        let (shape, size) = size.shape_size();

        self.attributes.set_sprite(sprite.location(), shape, size);
        self.attributes.set_palette(sprite.palette_location());

        self
    }

    pub fn set_sprite(&mut self, sprite: SpriteVram) -> &mut Self {
        self.set_sprite_attributes(&sprite);

        // SAFETY: This is called here and in OAMSlot set, neither of which call the other.
        let sprites = unsafe { &mut *self.sprites.get() };
        sprites.sprite = sprite;

        self
    }
}
