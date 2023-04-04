use core::cell::{Cell, UnsafeCell};

use agb_fixnum::Vector2D;
use slotmap::{new_key_type, SlotMap};

use crate::display::Priority;

use super::{
    AffineMatrixInstance, AffineMode, OamUnmanaged, ObjectUnmanaged, Sprite, SpriteLoader,
    SpriteVram,
};

new_key_type! {struct ObjectKey; }

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
    store: UnsafeCell<slotmap::SlotMap<ObjectKey, ObjectItem>>,
    first_z: Cell<Option<ObjectKey>>,
}

struct StoreIterator<'store> {
    store: &'store slotmap::SlotMap<ObjectKey, ObjectItem>,
    current: Option<ObjectKey>,
}

impl<'store> Iterator for StoreIterator<'store> {
    type Item = &'store ObjectItem;

    fn next(&mut self) -> Option<Self::Item> {
        let to_output = &self.store[self.current?];
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
            data.insert(object_item)
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
        data.remove(object);
    }

    fn get_object(&self, key: ObjectKey) -> &ObjectItem {
        &(unsafe { &*self.store.get() }[key])
    }
}

pub struct OamManager<'gba> {
    object_store: Store,
    sprite_loader: UnsafeCell<SpriteLoader>,
    unmanaged: UnsafeCell<OamUnmanaged<'gba>>,
}

impl OamManager<'_> {
    pub(crate) fn new() -> Self {
        Self {
            object_store: Store {
                store: UnsafeCell::new(SlotMap::with_key()),
                first_z: Cell::new(None),
            },
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

    pub fn commit(&self) {
        // safety: commit is not reentrant
        let unmanaged = unsafe { &mut *self.unmanaged.get() };

        for (object, mut slot) in unsafe { self.object_store.iter() }
            .map(|item| unsafe { &*item.object.get() })
            .filter(|object| object.is_visible())
            .zip(unmanaged.iter())
        {
            slot.set(object);
        }

        // safety: not reentrant
        unsafe {
            self.do_work_with_sprite_loader(SpriteLoader::garbage_collect);
        }
    }

    pub fn add_object(&self, sprite: SpriteVram) -> Object<'_> {
        self.object_store
            .insert_object(ObjectUnmanaged::new(sprite))
    }

    pub fn get_vram_sprite(&self, sprite: &'static Sprite) -> SpriteVram {
        // safety: not reentrant
        unsafe {
            self.do_work_with_sprite_loader(|sprite_loader| sprite_loader.get_vram_sprite(sprite))
        }
    }

    pub fn add_object_static_sprite(&self, sprite: &'static Sprite) -> Object<'_> {
        self.add_object(self.get_vram_sprite(sprite))
    }
}

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
    pub fn is_visible(&self) -> bool {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object_shared() }.is_visible()
    }

    pub fn show(&mut self) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().show() };

        self
    }

    pub fn show_affine(&mut self, affine_mode: AffineMode) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().show_affine(affine_mode) };

        self
    }

    pub fn set_hflip(&mut self, flip: bool) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_hflip(flip) };

        self
    }

    pub fn set_vflip(&mut self, flip: bool) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_vflip(flip) };

        self
    }

    pub fn set_x(&mut self, x: u16) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_x(x) };

        self
    }

    pub fn set_priority(&mut self, priority: Priority) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_priority(priority) };

        self
    }

    pub fn hide(&mut self) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().hide() };

        self
    }

    pub fn set_y(&mut self, y: u16) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_y(y) };

        self
    }

    pub fn set_position(&mut self, position: Vector2D<i32>) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_position(position) };

        self
    }

    pub fn set_affine_matrix(&mut self, affine_matrix: AffineMatrixInstance) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_affine_matrix(affine_matrix) };

        self
    }

    pub fn set_sprite(&mut self, sprite: SpriteVram) -> &mut Self {
        // safety: only have one of these, doesn't modify slotmap
        unsafe { self.object().set_sprite(sprite) };

        self
    }
}
