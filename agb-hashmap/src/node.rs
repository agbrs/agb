use core::{
    mem::{self, MaybeUninit},
    ptr,
};

use crate::HashType;

pub(crate) struct Node<K, V> {
    hash: HashType,

    // distance_to_initial_bucket = -1 => key and value are uninit.
    // distance_to_initial_bucket >= 0 => key and value are init
    distance_to_initial_bucket: i32,
    key: MaybeUninit<K>,
    value: MaybeUninit<V>,
}

impl<K, V> Node<K, V> {
    pub(crate) const fn new() -> Self {
        Self {
            hash: HashType::new(),
            distance_to_initial_bucket: -1,
            key: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
        }
    }

    pub(crate) fn new_with(key: K, value: V, hash: HashType) -> Self {
        Self {
            hash,
            distance_to_initial_bucket: 0,
            key: MaybeUninit::new(key),
            value: MaybeUninit::new(value),
        }
    }

    /// # Safety
    /// - Self actually has a value
    pub(crate) unsafe fn value_ref_unchecked(&self) -> &V {
        // SAFETY: Has a value
        unsafe { self.value.assume_init_ref() }
    }

    /// # Safety
    /// - Self actually has a value
    pub(crate) unsafe fn value_mut_unchecked(&mut self) -> &mut V {
        // SAFETY: Has a value
        unsafe { self.value.assume_init_mut() }
    }

    pub(crate) fn key_ref(&self) -> Option<&K> {
        if self.distance_to_initial_bucket >= 0 {
            Some(
                // SAFETY: has a value
                unsafe { self.key.assume_init_ref() },
            )
        } else {
            None
        }
    }

    pub(crate) fn key_value_ref(&self) -> Option<(&K, &V)> {
        if self.has_value() {
            Some(
                // SAFETY: has a value
                unsafe { self.key_value_ref_unchecked() },
            )
        } else {
            None
        }
    }

    /// # Safety
    /// - Self actually has a value
    pub(crate) unsafe fn key_value_ref_unchecked(&self) -> (&K, &V) {
        // SAFETY: Self has a value
        unsafe { (self.key.assume_init_ref(), self.value.assume_init_ref()) }
    }

    pub(crate) fn key_value_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.has_value() {
            Some(
                // SAFETY: has a value
                unsafe { (self.key.assume_init_ref(), self.value.assume_init_mut()) },
            )
        } else {
            None
        }
    }

    pub(crate) fn has_value(&self) -> bool {
        self.distance_to_initial_bucket >= 0
    }

    pub(crate) fn take_key_value(&mut self) -> Option<(K, V, HashType)> {
        if self.has_value() {
            let key = mem::replace(&mut self.key, MaybeUninit::uninit());
            let value = mem::replace(&mut self.value, MaybeUninit::uninit());
            self.distance_to_initial_bucket = -1;

            Some(
                // SAFETY: has a value
                unsafe { (key.assume_init(), value.assume_init(), self.hash) },
            )
        } else {
            None
        }
    }

    /// # Safety
    /// - Self has a value
    pub(crate) unsafe fn replace_value_unchecked(&mut self, value: V) -> V {
        // SAFETY: Self has a value
        unsafe {
            let old_value = mem::replace(&mut self.value, MaybeUninit::new(value));
            old_value.assume_init()
        }
    }

    /// # Safety
    /// - Self has a value
    pub(crate) unsafe fn replace_unchecked(&mut self, key: K, value: V) -> (K, V) {
        // SAFETY: Self has a value
        unsafe {
            let old_key = mem::replace(&mut self.key, MaybeUninit::new(key));
            let old_value = mem::replace(&mut self.value, MaybeUninit::new(value));

            (old_key.assume_init(), old_value.assume_init())
        }
    }

    pub(crate) fn increment_distance(&mut self) {
        self.distance_to_initial_bucket += 1;
    }

    /// # Panics
    /// - If current distance <= 0
    pub(crate) fn decrement_distance(&mut self) {
        self.distance_to_initial_bucket -= 1;

        assert!(
            self.distance_to_initial_bucket >= 0,
            "Cannot decrement distance below 0"
        );
    }

    pub(crate) fn distance(&self) -> i32 {
        self.distance_to_initial_bucket
    }

    pub(crate) fn hash(&self) -> HashType {
        self.hash
    }
}

impl<K, V> Drop for Node<K, V> {
    fn drop(&mut self) {
        if self.has_value() {
            // SAFETY: has a value
            unsafe {
                ptr::drop_in_place(self.key.as_mut_ptr());
                ptr::drop_in_place(self.value.as_mut_ptr());
            }
        }
    }
}

impl<K, V> Clone for Node<K, V>
where
    K: Clone,
    V: Clone,
{
    fn clone(&self) -> Self {
        if let Some((k, v)) = self.key_value_ref() {
            Self {
                hash: self.hash,
                distance_to_initial_bucket: self.distance_to_initial_bucket,
                key: MaybeUninit::new(k.clone()),
                value: MaybeUninit::new(v.clone()),
            }
        } else {
            Self {
                hash: self.hash,

                distance_to_initial_bucket: self.distance_to_initial_bucket,
                key: MaybeUninit::uninit(),
                value: MaybeUninit::uninit(),
            }
        }
    }
}
