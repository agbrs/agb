use core::{alloc::Allocator, mem::ManuallyDrop};

use alloc::{alloc::Global, vec::Vec};

union ArenaItem<T> {
    free: Option<ArenaKey>,
    occupied: ManuallyDrop<T>,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ArenaKey(usize);

pub struct Arena<T, A: Allocator = Global> {
    tip: Option<ArenaKey>,
    data: Vec<ArenaItem<T>, A>,
    inserted: usize,
}

impl<T> Arena<T, Global> {
    pub const fn new() -> Self {
        Self::new_in(Global)
    }
}

impl<T, A: Allocator> Arena<T, A> {
    pub const fn new_in(alloc: A) -> Self {
        Self {
            tip: None,
            data: Vec::new_in(alloc),
            inserted: 0,
        }
    }

    pub unsafe fn insert(&mut self, value: T) -> ArenaKey {
        self.inserted += 1;
        match self.tip {
            Some(tip) => {
                self.tip = self.data[tip.0].free;
                self.data[tip.0].occupied = ManuallyDrop::new(value);
                tip
            }
            None => {
                self.data.push(ArenaItem {
                    occupied: ManuallyDrop::new(value),
                });
                ArenaKey(self.data.len() - 1)
            }
        }
    }

    pub unsafe fn remove(&mut self, key: ArenaKey) {
        self.inserted = self
            .inserted
            .checked_sub(1)
            .expect("removed more items than exist in here!");

        unsafe {
            core::mem::ManuallyDrop::<T>::drop(&mut self.data[key.0].occupied);
        }

        self.data[key.0].free = self.tip;
        self.tip = Some(key);
    }

    pub unsafe fn get(&self, key: ArenaKey) -> &T {
        &self.data[key.0].occupied
    }
}

impl<T, A: Allocator> Drop for Arena<T, A> {
    fn drop(&mut self) {
        assert_eq!(
            self.inserted, 0,
            "must remove all elements from arena before dropping it!"
        );
    }
}
