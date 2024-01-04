use core::cell::{Cell, UnsafeCell};

use agb_fixnum::Vector2D;

use crate::{
    arena::{Arena, ArenaKey},
    display::Priority,
};

use super::{
    AffineMatrixInstance, AffineMode, OamUnmanaged, ObjectUnmanaged, Sprite, SpriteLoader,
    SpriteVram,
};

type ObjectKey = ArenaKey;

#[derive(Clone, Copy)]
struct Ordering {
    next: Option<ObjectKey>,
    previous: Option<ObjectKey>,
}

struct ObjectItem {
    object: UnsafeCell<ObjectUnmanaged>,
    z_order: Cell<Ordering>,
    z_index: Cell<i32>,
}

struct Store {
    store: UnsafeCell<Arena<ObjectItem>>,
    first_z: Cell<Option<ObjectKey>>,
}

struct StoreIterator<'store> {
    store: &'store Arena<ObjectItem>,
    current: Option<ObjectKey>,
}

impl<'store> Iterator for StoreIterator<'store> {
    type Item = &'store ObjectItem;

    fn next(&mut self) -> Option<Self::Item> {
        let to_output = unsafe { self.store.get(self.current?) };
        self.current = to_output.z_order.get().next;
        Some(to_output)
    }
}

impl Store {
    /// SAFETY: while this exists, no other store related operations should be
    /// performed. Notably this means you shouldn't drop the ObjectItem as this
    /// implementation will touch this.
    unsafe fn iter(&self) -> StoreIterator {
        StoreIterator {
            store: unsafe { &*self.store.get() },
            current: self.first_z.get(),
        }
    }

    #[cfg(test)]
    fn is_all_ordered_right(&self) -> bool {
        let mut previous_z = i32::MIN;
        let mut current_index = self.first_z.get();

        while let Some(ci) = current_index {
            let obj = self.get_object(ci);
            let this_z = obj.z_index.get();
            if this_z < previous_z {
                return false;
            }
            previous_z = this_z;
            current_index = obj.z_order.get().next;
        }

        true
    }

    fn insert_object(&self, object: ObjectUnmanaged) -> Object {
        let object_item = ObjectItem {
            object: UnsafeCell::new(object),
            z_order: Cell::new(Ordering {
                next: None,
                previous: None,
            }),
            z_index: Cell::new(0),
        };
        let idx = {
            let data = unsafe { &mut *self.store.get() };
            unsafe { data.insert(object_item) }
        };

        if let Some(first) = self.first_z.get() {
            let mut this_index = first;
            while self.get_object(this_index).z_index.get() < 0 {
                if let Some(idx) = self.get_object(this_index).z_order.get().next {
                    this_index = idx;
                } else {
                    break;
                }
            }
            if self.get_object(this_index).z_index.get() < 0 {
                add_after_element(self, idx, this_index);
            } else {
                add_before_element(self, idx, this_index);
            }
        } else {
            self.first_z.set(Some(idx));
        }

        Object {
            me: idx,
            store: self,
        }
    }

    fn remove_object(&self, object: ObjectKey) {
        remove_from_linked_list(self, object);

        let data = unsafe { &mut *self.store.get() };
        unsafe { data.remove(object) };
    }

    fn get_object(&self, key: ObjectKey) -> &ObjectItem {
        unsafe { (*self.store.get()).get(key) }
    }
}

/// OAM that manages z ordering and commit all visible objects in one call. This
/// is simpler to use than the [`OamUnmanaged`], but is less performant
/// depending on how objects are stored.
///
/// Use this if:
/// * You don't want to handle z ordering.
/// * You don't want to deal with the complexity of committing all objects during vblank.
///
/// Otherwise I'd recommend using [`OamUnmanaged`].
pub struct OamManaged<'gba> {
    object_store: OrderedStore,
    sprite_loader: UnsafeCell<SpriteLoader>,
    unmanaged: UnsafeCell<OamUnmanaged<'gba>>,
}

/// Stores a bunch of objects and manages the z ordering for you.
///
/// An alternate to consider is using an arena, storing keys in a vector, and
/// sorting that vector by key in the arena.
pub struct OrderedStore {
    object_store: Store,
}

/// An iterator over the visible objects in the object store.
pub struct OrderedStoreIterator<'store> {
    iter: StoreIterator<'store>,
}

impl<'store> Iterator for OrderedStoreIterator<'store> {
    type Item = &'store ObjectUnmanaged;

    fn next(&mut self) -> Option<Self::Item> {
        for next in self.iter.by_ref() {
            let item = unsafe { &*next.object.get() };
            if item.is_visible() {
                return Some(item);
            }
        }

        None
    }
}

impl<'a> IntoIterator for &'a OrderedStore {
    type Item = &'a ObjectUnmanaged;

    type IntoIter = OrderedStoreIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        OrderedStoreIterator {
            iter: unsafe { self.object_store.iter() },
        }
    }
}

impl OrderedStore {
    /// Creates a new empty ordered object store.
    #[must_use]
    pub fn new() -> Self {
        Self {
            object_store: Store {
                store: UnsafeCell::new(Arena::new()),
                first_z: Cell::new(None),
            },
        }
    }

    /// Creates an object from the sprite in vram.
    pub fn object(&self, sprite: SpriteVram) -> Object<'_> {
        self.object_store
            .insert_object(ObjectUnmanaged::new(sprite))
    }

    /// Iter over the ordered store in order
    pub fn iter(&self) -> OrderedStoreIterator {
        self.into_iter()
    }
}

impl Default for OrderedStore {
    fn default() -> Self {
        Self::new()
    }
}

impl OamManaged<'_> {
    pub(crate) fn new() -> Self {
        Self {
            object_store: OrderedStore::new(),
            sprite_loader: UnsafeCell::new(SpriteLoader::new()),
            unmanaged: UnsafeCell::new(OamUnmanaged::new()),
        }
    }

    /// SAFETY:
    /// Do not reenter or recurse or otherwise use sprite loader cell during this.
    unsafe fn do_work_with_sprite_loader<C, T>(&self, c: C) -> T
    where
        C: Fn(&mut SpriteLoader) -> T,
    {
        let sprite_loader = unsafe { &mut *self.sprite_loader.get() };

        c(sprite_loader)
    }

    /// Commits all the visible objects. Call during vblank to make changes made
    /// to objects visible.
    pub fn commit(&self) {
        // safety: commit is not reentrant
        let unmanaged = unsafe { &mut *self.unmanaged.get() };

        let mut unmanaged = unmanaged.iter();

        unmanaged.set(&self.object_store);

        // safety: not reentrant
        unsafe {
            self.do_work_with_sprite_loader(SpriteLoader::garbage_collect);
        }
    }

    /// Creates an object from the sprite in vram.
    pub fn object(&self, sprite: SpriteVram) -> Object<'_> {
        self.object_store.object(sprite)
    }

    /// Creates a sprite in vram from a static sprite from [`include_aseprite`][crate::include_aseprite].
    pub fn sprite(&self, sprite: &'static Sprite) -> SpriteVram {
        // safety: not reentrant
        unsafe {
            self.do_work_with_sprite_loader(|sprite_loader| sprite_loader.get_vram_sprite(sprite))
        }
    }

    /// Creates a sprite in vram and uses it to make an object from a static sprite from [`include_aseprite`][crate::include_aseprite].
    pub fn object_sprite(&self, sprite: &'static Sprite) -> Object<'_> {
        self.object(self.sprite(sprite))
    }
}

/// A managed object used with the [`OamManaged`] interface.
pub struct Object<'controller> {
    me: ObjectKey,
    store: &'controller Store,
}

impl Drop for Object<'_> {
    fn drop(&mut self) {
        self.store.remove_object(self.me);
    }
}

fn remove_from_linked_list(store: &Store, to_remove: ObjectKey) {
    let my_current_neighbours = store.get_object(to_remove).z_order.get();

    if let Some(previous) = my_current_neighbours.previous {
        let stored_part = &store.get_object(previous).z_order;
        let mut neighbour_left = stored_part.get();
        neighbour_left.next = my_current_neighbours.next;
        stored_part.set(neighbour_left);
    } else {
        store.first_z.set(my_current_neighbours.next);
    }

    if let Some(next) = my_current_neighbours.next {
        let stored_part = &store.get_object(next).z_order;
        let mut neighbour_right = stored_part.get();
        neighbour_right.previous = my_current_neighbours.previous;
        stored_part.set(neighbour_right);
    }

    store.get_object(to_remove).z_order.set(Ordering {
        next: None,
        previous: None,
    });
}

fn add_before_element(store: &Store, elem: ObjectKey, before_this: ObjectKey) {
    assert_ne!(elem, before_this);

    let this_element_store = &store.get_object(elem).z_order;
    let mut this_element = this_element_store.get();

    let before_store = &store.get_object(before_this).z_order;
    let mut before = before_store.get();

    if let Some(previous) = before.previous {
        let neighbour_left_store = &store.get_object(previous).z_order;
        let mut neighbour_left = neighbour_left_store.get();
        neighbour_left.next = Some(elem);
        neighbour_left_store.set(neighbour_left);
    } else {
        store.first_z.set(Some(elem));
    }
    this_element.next = Some(before_this);
    this_element.previous = before.previous;

    before.previous = Some(elem);

    this_element_store.set(this_element);
    before_store.set(before);
}

fn add_after_element(store: &Store, elem: ObjectKey, after_this: ObjectKey) {
    assert_ne!(elem, after_this);

    let this_element_store = &store.get_object(elem).z_order;
    let mut this_element = this_element_store.get();

    let after_store = &store.get_object(after_this).z_order;
    let mut after = after_store.get();

    if let Some(next) = after.next {
        let neighbour_left_store = &store.get_object(next).z_order;
        let mut neighbour_right = neighbour_left_store.get();
        neighbour_right.previous = Some(elem);
        neighbour_left_store.set(neighbour_right);
    }

    this_element.previous = Some(after_this);
    this_element.next = after.next;

    after.next = Some(elem);

    this_element_store.set(this_element);
    after_store.set(after);
}

fn move_before(store: &Store, source: ObjectKey, before_this: ObjectKey) {
    assert_ne!(source, before_this);

    remove_from_linked_list(store, source);
    add_before_element(store, source, before_this);
}

fn move_after(store: &Store, source: ObjectKey, after_this: ObjectKey) {
    assert_ne!(source, after_this);

    remove_from_linked_list(store, source);
    add_after_element(store, source, after_this);
}

impl Object<'_> {
    /// Sets the z position of an object. This is not a GBA concept. It causes
    /// the order of rendering to be different, thus changing whether objects
    /// are rendered above eachother.
    ///
    /// Negative z is more towards the outside and positive z is further into
    /// the screen => an object with a more *negative* z is drawn on top of an
    /// object with a more *positive* z.
    pub fn set_z(&mut self, z_index: i32) -> &mut Self {
        let my_object = &self.store.get_object(self.me);

        let order = z_index.cmp(&my_object.z_index.get());

        match order {
            core::cmp::Ordering::Equal => {}
            core::cmp::Ordering::Less => {
                let mut previous_index = self.me;
                let mut current_index = self.me;
                while self.store.get_object(current_index).z_index.get() > z_index {
                    previous_index = current_index;
                    let previous = self.store.get_object(current_index).z_order.get().previous;
                    if let Some(previous) = previous {
                        current_index = previous;
                    } else {
                        break;
                    }
                }
                if previous_index != self.me {
                    move_before(self.store, self.me, previous_index);
                }
            }
            core::cmp::Ordering::Greater => {
                let mut previous_index = self.me;
                let mut current_index = self.me;
                while self.store.get_object(current_index).z_index.get() < z_index {
                    previous_index = current_index;
                    let next = self.store.get_object(current_index).z_order.get().next;
                    if let Some(next) = next {
                        current_index = next;
                    } else {
                        break;
                    }
                }
                if previous_index != self.me {
                    move_after(self.store, self.me, previous_index);
                }
            }
        }

        my_object.z_index.set(z_index);

        self
    }

    /// Safety:
    /// Only have *ONE* of these at a time, do not call any functions that modify the slot map while having this.
    unsafe fn object(&mut self) -> &mut ObjectUnmanaged {
        unsafe { &mut *self.store.get_object(self.me).object.get() }
    }

    /// Safety:
    /// Don't have a mutable one of these while having one of these, do not call any functions that modify the slot map while having this.
    unsafe fn object_shared(&self) -> &ObjectUnmanaged {
        unsafe { &*self.store.get_object(self.me).object.get() }
    }

    #[must_use]
    /// Checks whether the object is not marked as hidden. Note that it could be
    /// off screen or completely transparent and still claimed to be visible.
    pub fn is_visible(&self) -> bool {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object_shared() }.is_visible()
    }

    /// Display the sprite in Normal mode.
    pub fn show(&mut self) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().show() };

        self
    }

    /// Display the sprite in Affine mode.
    pub fn show_affine(&mut self, affine_mode: AffineMode) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().show_affine(affine_mode) };

        self
    }

    /// Sets the horizontal flip, note that this only has a visible affect in Normal mode.
    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_hflip(flip) };

        self
    }

    /// Sets the vertical flip, note that this only has a visible affect in Normal mode.
    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_vflip(flip) };

        self
    }

    /// Sets the priority of the object relative to the backgrounds priority.
    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_priority(priority) };

        self
    }

    /// Changes the sprite mode to be hidden, can be changed to Normal or Affine
    /// modes using [`show`][Object::show] and
    /// [`show_affine`][Object::show_affine] respectively.
    pub fn hide(&mut self) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().hide() };

        self
    }

    /// Sets the x position of the object.
    pub fn set_x(&mut self, x: u16) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_x(x) };

        self
    }

    /// Sets the y position of the object.
    pub fn set_y(&mut self, y: u16) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_y(y) };

        self
    }

    /// Sets the position of the object.
    pub fn set_position(&mut self, position: Vector2D<i32>) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_position(position) };

        self
    }

    /// Sets the affine matrix. This only has an affect in Affine mode.
    pub fn set_affine_matrix(&mut self, affine_matrix: AffineMatrixInstance) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_affine_matrix(affine_matrix) };

        self
    }

    /// Sets the current sprite for the object.
    pub fn set_sprite(&mut self, sprite: SpriteVram) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_sprite(sprite) };

        self
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use crate::{display::object::Graphics, include_aseprite};

    use super::*;

    const TEST_SPRITES: &Graphics = include_aseprite!("examples/gfx/tall.aseprite");

    const TEST_SPRITE: &Sprite = &TEST_SPRITES.sprites()[0];

    #[test_case]
    fn test_always_ordered(gba: &mut crate::Gba) {
        let managed = gba.display.object.get_managed();

        let sprite = managed.sprite(TEST_SPRITE);

        let mut objects = Vec::new();
        for _ in 0..200 {
            let obj = managed.object(sprite.clone());
            objects.push(obj);
        }

        for modification_number in 0..10_000 {
            let index_to_modify = (crate::rng::gen() as usize) % objects.len();
            let modify_to = crate::rng::gen();
            objects[index_to_modify].set_z(modify_to);

            assert!(
                managed.object_store.object_store.is_all_ordered_right(),
                "objects are unordered after {} modifications. Modified {} to {}.",
                modification_number + 1,
                index_to_modify,
                modify_to
            );
        }
    }
}
