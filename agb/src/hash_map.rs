#![deny(missing_docs)]
//! A lot of the documentation for this module was copied straight out of the rust
//! standard library. The implementation however is not.
use alloc::vec::Vec;
use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
    iter::{self, FromIterator},
    mem::{self, MaybeUninit},
    ops::Index,
    ptr,
};

use rustc_hash::FxHasher;

type HashType = u32;

// # Robin Hood Hash Tables
//
// The problem with regular hash tables where failing to find a slot for a specific
// key will result in a linear search for the first free slot is that often these
// slots can end up being quite far away from the original chosen location in fuller
// hash tables. In Java, the hash table will resize when it is more than 2 thirds full
// which is quite wasteful in terms of space. Robin Hood hash tables can be much
// fuller before needing to resize and also keeps search times lower.
//
// The key concept is to keep the distance from the initial bucket chosen for a given
// key to a minimum. We shall call this distance the "distance to the initial bucket"
// or DIB for short. With each key - value pair, we store its DIB. When inserting
// a value into the hashtable, we check to see if there is an element in the initial
// bucket. If there is, we move onto the next value. Then, we check to see if there
// is already a value there and if there is, we check its DIB. If our DIB is greater
// than or equal to the DIB of the value that is already there, we swap the working
// value and the current entry. This continues until an empty slot is found.
//
// Using this technique, the average DIB is kept fairly low which decreases search
// times. As a simple search time optimisation, the maximum DIB is kept track of
// and so we will only need to search as far as that in order to know whether or
// not a given element is in the hash table.
//
// # Deletion
//
// Special mention is given to deletion. Unfortunately, the maximum DIB is not
// kept track of after deletion, since we would not only need to keep track of
// the maximum DIB but also the number of elements which have that maximum DIB.
//
// In order to delete an element, we search to see if it exists. If it does,
// we remove that element and then iterate through the array from that point
// and move each element back one space (updating its DIB). If the DIB of the
// element we are trying to remove is 0, then we stop this algorithm.
//
// This means that deletion will lower the average DIB of the elements and
// keep searching and insertion fast.
//
// # Rehashing
//
// Currently, no incremental rehashing takes place. Once the HashMap becomes
// more than 85% full (this value may change when I do some benchmarking),
// a new list is allocated with double the capacity and the entire node list
// is migrated.

/// A hash map implemented very simply using robin hood hashing.
///
/// `HashMap` uses `FxHasher` internally, which is a very fast hashing algorithm used
/// by rustc and firefox in non-adversarial places. It is incredibly fast, and good
/// enough for most cases.
///
/// It is required that the keys implement the [`Eq`] and [`Hash`] traits, although this
/// can be frequently achieved by using `#[derive(PartialEq, Eq, Hash)]`. If you
/// implement these yourself, it is important that the following property holds:
///
/// `k1 == k2 => hash(k1) == hash(k2)`
///
/// It is a logic error for the key to be modified in such a way that the key's hash, as
/// determined by the [`Hash`] trait, or its equality as determined by the [`Eq`] trait,
/// changes while it is in the map. The behaviour for such a logic error is not specified,
/// but will not result in undefined behaviour. This could include panics, incorrect results,
/// aborts, memory leaks and non-termination.
///
/// The API surface provided is incredibly similar to the
/// [`std::collections::HashMap`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
/// implementation with fewer guarantees, and better optimised for the GameBoy Advance.
///
/// [`Eq`]: https://doc.rust-lang.org/core/cmp/trait.Eq.html
/// [`Hash`]: https://doc.rust-lang.org/core/hash/trait.Hash.html
pub struct HashMap<K, V> {
    nodes: NodeStorage<K, V>,

    hasher: BuildHasherDefault<FxHasher>,
}

impl<K, V> HashMap<K, V> {
    /// Creates a `HashMap`
    #[must_use]
    pub fn new() -> Self {
        Self::with_size(16)
    }

    /// Creates an empty `HashMap` with specified internal size. The size must be a power of 2
    #[must_use]
    pub fn with_size(size: usize) -> Self {
        Self {
            nodes: NodeStorage::with_size(size),
            hasher: Default::default(),
        }
    }

    /// Creates an empty `HashMap` which can hold at least `capacity` elements before resizing. The actual
    /// internal size may be larger as it must be a power of 2
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        for i in 0..32 {
            let attempted_size = 1usize << i;
            if number_before_resize(attempted_size) > capacity {
                return Self::with_size(attempted_size);
            }
        }

        panic!(
            "Failed to come up with a size which satisfies capacity {}",
            capacity
        );
    }

    /// Returns the number of elements in the map
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of elements the map can hold
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// An iterator visiting all keys in an arbitrary order
    pub fn keys(&self) -> impl Iterator<Item = &'_ K> {
        self.iter().map(|(k, _)| k)
    }

    /// An iterator visiting all values in an arbitrary order
    pub fn values(&self) -> impl Iterator<Item = &'_ V> {
        self.iter().map(|(_, v)| v)
    }

    /// An iterator visiting all values in an arbitrary order allowing for mutation
    pub fn values_mut(&mut self) -> impl Iterator<Item = &'_ mut V> {
        self.iter_mut().map(|(_, v)| v)
    }

    /// Removes all elements from the map
    pub fn clear(&mut self) {
        self.nodes = NodeStorage::with_size(self.nodes.backing_vec_size());
    }

    /// An iterator visiting all key-value pairs in an arbitrary order
    pub fn iter(&self) -> impl Iterator<Item = (&'_ K, &'_ V)> {
        self.nodes.nodes.iter().filter_map(Node::key_value_ref)
    }

    /// An iterator visiting all key-value pairs in an arbitrary order, with mutable references to the values
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&'_ K, &'_ mut V)> {
        self.nodes.nodes.iter_mut().filter_map(Node::key_value_mut)
    }

    /// Returns `true` if the map contains no elements
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn resize(&mut self, new_size: usize) {
        assert!(
            new_size >= self.nodes.backing_vec_size(),
            "Can only increase the size of a hash map"
        );
        if new_size == self.nodes.backing_vec_size() {
            return;
        }

        self.nodes = self.nodes.resized_to(new_size);
    }
}

impl<K, V> Default for HashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

const fn fast_mod(len: usize, hash: HashType) -> usize {
    debug_assert!(len.is_power_of_two(), "Length must be a power of 2");
    (hash as usize) & (len - 1)
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated and the old value
    /// is returned. The key is not updated, which matters for types that can be `==`
    /// without being identical.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hash = self.hash(&key);

        if let Some(location) = self.nodes.location(&key, hash) {
            Some(self.nodes.replace_at_location(location, key, value))
        } else {
            if self.nodes.capacity() <= self.len() {
                self.resize(self.nodes.backing_vec_size() * 2);
            }

            self.nodes.insert_new(key, value, hash);

            None
        }
    }

    fn insert_and_get(&mut self, key: K, value: V) -> &'_ mut V {
        let hash = self.hash(&key);

        let location = if let Some(location) = self.nodes.location(&key, hash) {
            self.nodes.replace_at_location(location, key, value);
            location
        } else {
            if self.nodes.capacity() <= self.len() {
                self.resize(self.nodes.backing_vec_size() * 2);
            }

            self.nodes.insert_new(key, value, hash)
        };

        self.nodes.nodes[location].value_mut().unwrap()
    }

    /// Returns `true` if the map contains a value for the specified key.
    pub fn contains_key(&self, k: &K) -> bool {
        let hash = self.hash(k);
        self.nodes.location(k, hash).is_some()
    }

    /// Returns the key-value pair corresponding to the supplied key
    pub fn get_key_value(&self, key: &K) -> Option<(&K, &V)> {
        let hash = self.hash(key);

        self.nodes
            .location(key, hash)
            .and_then(|location| self.nodes.nodes[location].key_value_ref())
    }

    /// Returns a reference to the value corresponding to the key. Returns [`None`] if there is
    /// no element in the map with the given key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.get_key_value(key).map(|(_, v)| v)
    }

    /// Returns a mutable reference to the value corresponding to the key. Return [`None`] if
    /// there is no element in the map with the given key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash(key);

        if let Some(location) = self.nodes.location(key, hash) {
            self.nodes.nodes[location].value_mut()
        } else {
            None
        }
    }

    /// Removes the given key from the map. Returns the current value if it existed, or [`None`]
    /// if it did not.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);

        self.nodes
            .location(key, hash)
            .map(|location| self.nodes.remove_from_location(location))
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash,
{
    fn hash(&self, key: &K) -> HashType {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        hasher.finish() as HashType
    }
}

/// An iterator over entries of a [`HashMap`]
///
/// This struct is created using the `into_iter()` method on [`HashMap`]. See its
/// documentation for more.
pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.at >= self.map.nodes.backing_vec_size() {
                return None;
            }

            let node = &self.map.nodes.nodes[self.at];
            self.at += 1;

            if node.has_value() {
                return Some((node.key_ref().unwrap(), node.value_ref().unwrap()));
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter { map: self, at: 0 }
    }
}

/// An iterator over entries of a [`HashMap`]
///
/// This struct is created using the `into_iter()` method on [`HashMap`] as part of its implementation
/// of the IntoIterator trait.
pub struct IterOwned<K, V> {
    map: HashMap<K, V>,
    at: usize,
}

impl<K, V> Iterator for IterOwned<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.at >= self.map.nodes.backing_vec_size() {
                return None;
            }

            let maybe_kv = self.map.nodes.nodes[self.at].take_key_value();
            self.at += 1;

            if let Some((k, v, _)) = maybe_kv {
                return Some((k, v));
            }
        }
    }
}

/// An iterator over entries of a [`HashMap`]
///
/// This struct is created using the `into_iter()` method on [`HashMap`] as part of its implementation
/// of the IntoIterator trait.
impl<K, V> IntoIterator for HashMap<K, V> {
    type Item = (K, V);
    type IntoIter = IterOwned<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IterOwned { map: self, at: 0 }
    }
}

/// A view into an occupied entry in a `HashMap`. This is part of the [`Entry`] enum.
pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    key: K,
    map: &'a mut HashMap<K, V>,
    location: usize,
}

impl<'a, K: 'a, V: 'a> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take the ownership of the key and value from the map.
    pub fn remove_entry(self) -> (K, V) {
        let old_value = self.map.nodes.remove_from_location(self.location);
        (self.key, old_value)
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        self.map.nodes.nodes[self.location].value_ref().unwrap()
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the destruction
    /// of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: Self::into_mut
    pub fn get_mut(&mut self) -> &mut V {
        self.map.nodes.nodes[self.location].value_mut().unwrap()
    }

    /// Converts the `OccupiedEntry` into a mutable reference to the value in the entry with
    /// a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: Self::get_mut
    pub fn into_mut(self) -> &'a mut V {
        self.map.nodes.nodes[self.location].value_mut().unwrap()
    }

    /// Sets the value of the entry and returns the entry's old value.
    pub fn insert(&mut self, value: V) -> V {
        self.map.nodes.nodes[self.location].replace_value(value)
    }

    /// Takes the value out of the entry and returns it.
    pub fn remove(self) -> V {
        self.map.nodes.remove_from_location(self.location)
    }
}

/// A view into a vacant entry in a `HashMap`. It is part of the [`Entry`] enum.
pub struct VacantEntry<'a, K: 'a, V: 'a> {
    key: K,
    map: &'a mut HashMap<K, V>,
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    /// Gets a reference to the key that would be used when inserting a value through `VacantEntry`
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key
    pub fn into_key(self) -> K {
        self.key
    }

    /// Sets the value of the entry with the `VacantEntry`'s key and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V
    where
        K: Hash + Eq,
    {
        self.map.insert_and_get(self.key, value)
    }
}

/// A view into a single entry in a map, which may be vacant or occupied.
///
/// This is constructed using the [`entry`] method on [`HashMap`]
///
/// [`entry`]: HashMap::entry()
pub enum Entry<'a, K: 'a, V: 'a> {
    /// An occupied entry
    Occupied(OccupiedEntry<'a, K, V>),
    /// A vacant entry
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
{
    /// Ensures a value is in the entry by inserting the given value, and returns a mutable
    /// reference to the value in the entry.
    pub fn or_insert(self, value: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(value),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the function if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(f()),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the function if empty, and
    /// returns a mutable reference to the value in the entry. This method allows for key-derived
    /// values for insertion by providing the function with a reference to the key.
    ///
    /// The reference to the moved key is provided so that cloning or copying the key is unnecessary,
    /// unlike with `.or_insert_with(|| ...)`.
    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let value = f(&e.key);
                e.insert(value)
            }
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential inserts
    /// into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut e) => {
                f(e.get_mut());
                Entry::Occupied(e)
            }
            Entry::Vacant(e) => Entry::Vacant(e),
        }
    }

    /// Ensures a value is in th entry by inserting the default value if empty. Returns a
    /// mutable reference to the value in the entry.
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        match self {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => e.insert(Default::default()),
        }
    }

    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(e) => &e.key,
            Entry::Vacant(e) => &e.key,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        let hash = self.hash(&key);
        let location = self.nodes.location(&key, hash);

        if let Some(location) = location {
            Entry::Occupied(OccupiedEntry {
                key,
                location,
                map: self,
            })
        } else {
            Entry::Vacant(VacantEntry { key, map: self })
        }
    }
}

impl<K, V> FromIterator<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut map = HashMap::new();
        map.extend(iter);
        map
    }
}

impl<K, V> Extend<(K, V)> for HashMap<K, V>
where
    K: Eq + Hash,
{
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (k, v) in iter {
            self.insert(k, v);
        }
    }
}

impl<K, V> Index<&K> for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Output = V;

    fn index(&self, key: &K) -> &V {
        self.get(key).expect("no entry found for key")
    }
}

impl<K, V> Index<K> for HashMap<K, V>
where
    K: Eq + Hash,
{
    type Output = V;

    fn index(&self, key: K) -> &V {
        self.get(&key).expect("no entry found for key")
    }
}

const fn number_before_resize(capacity: usize) -> usize {
    capacity * 85 / 100
}

struct NodeStorage<K, V> {
    nodes: Vec<Node<K, V>>,
    max_distance_to_initial_bucket: i32,

    number_of_items: usize,
    max_number_before_resize: usize,
}

impl<K, V> NodeStorage<K, V> {
    fn with_size(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be a power of 2");

        Self {
            nodes: iter::repeat_with(Default::default).take(capacity).collect(),
            max_distance_to_initial_bucket: 0,
            number_of_items: 0,
            max_number_before_resize: number_before_resize(capacity),
        }
    }

    fn capacity(&self) -> usize {
        self.max_number_before_resize
    }

    fn backing_vec_size(&self) -> usize {
        self.nodes.len()
    }

    fn len(&self) -> usize {
        self.number_of_items
    }

    fn insert_new(&mut self, key: K, value: V, hash: HashType) -> usize {
        debug_assert!(
            self.capacity() > self.len(),
            "Do not have space to insert into len {} with {}",
            self.backing_vec_size(),
            self.len()
        );

        let mut new_node = Node::new_with(key, value, hash);
        let mut inserted_location = usize::MAX;

        loop {
            let location = fast_mod(
                self.backing_vec_size(),
                new_node.hash + new_node.distance() as HashType,
            );
            let current_node = &mut self.nodes[location];

            if current_node.has_value() {
                if current_node.distance() <= new_node.distance() {
                    mem::swap(&mut new_node, current_node);

                    if inserted_location == usize::MAX {
                        inserted_location = location;
                    }
                }
            } else {
                self.nodes[location] = new_node;
                if inserted_location == usize::MAX {
                    inserted_location = location;
                }
                break;
            }

            new_node.increment_distance();
            self.max_distance_to_initial_bucket =
                new_node.distance().max(self.max_distance_to_initial_bucket);
        }

        self.number_of_items += 1;
        inserted_location
    }

    fn remove_from_location(&mut self, location: usize) -> V {
        let mut current_location = location;
        self.number_of_items -= 1;

        loop {
            let next_location =
                fast_mod(self.backing_vec_size(), (current_location + 1) as HashType);

            // if the next node is empty, or the next location has 0 distance to initial bucket then
            // we can clear the current node
            if !self.nodes[next_location].has_value() || self.nodes[next_location].distance() == 0 {
                return self.nodes[current_location].take_key_value().unwrap().1;
            }

            self.nodes.swap(current_location, next_location);
            self.nodes[current_location].decrement_distance();
            current_location = next_location;
        }
    }

    fn location(&self, key: &K, hash: HashType) -> Option<usize>
    where
        K: Eq,
    {
        for distance_to_initial_bucket in 0..(self.max_distance_to_initial_bucket + 1) {
            let location = fast_mod(
                self.nodes.len(),
                hash + distance_to_initial_bucket as HashType,
            );

            let node = &self.nodes[location];
            if let Some(node_key_ref) = node.key_ref() {
                if node_key_ref == key {
                    return Some(location);
                }
            } else {
                return None;
            }
        }

        None
    }

    fn resized_to(&mut self, new_size: usize) -> Self {
        let mut new_node_storage = Self::with_size(new_size);

        for mut node in self.nodes.drain(..) {
            if let Some((key, value, hash)) = node.take_key_value() {
                new_node_storage.insert_new(key, value, hash);
            }
        }

        new_node_storage
    }

    fn replace_at_location(&mut self, location: usize, key: K, value: V) -> V {
        self.nodes[location].replace(key, value).1
    }
}

struct Node<K, V> {
    hash: HashType,

    // distance_to_initial_bucket = -1 => key and value are uninit.
    // distance_to_initial_bucket >= 0 => key and value are init
    distance_to_initial_bucket: i32,
    key: MaybeUninit<K>,
    value: MaybeUninit<V>,
}

impl<K, V> Node<K, V> {
    fn new() -> Self {
        Self {
            hash: 0,
            distance_to_initial_bucket: -1,
            key: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
        }
    }

    fn new_with(key: K, value: V, hash: HashType) -> Self {
        Self {
            hash,
            distance_to_initial_bucket: 0,
            key: MaybeUninit::new(key),
            value: MaybeUninit::new(value),
        }
    }

    fn value_ref(&self) -> Option<&V> {
        if self.has_value() {
            Some(unsafe { self.value.assume_init_ref() })
        } else {
            None
        }
    }

    fn value_mut(&mut self) -> Option<&mut V> {
        if self.has_value() {
            Some(unsafe { self.value.assume_init_mut() })
        } else {
            None
        }
    }

    fn key_ref(&self) -> Option<&K> {
        if self.distance_to_initial_bucket >= 0 {
            Some(unsafe { self.key.assume_init_ref() })
        } else {
            None
        }
    }

    fn key_value_ref(&self) -> Option<(&K, &V)> {
        if self.has_value() {
            Some(unsafe { (self.key.assume_init_ref(), self.value.assume_init_ref()) })
        } else {
            None
        }
    }

    fn key_value_mut(&mut self) -> Option<(&K, &mut V)> {
        if self.has_value() {
            Some(unsafe { (self.key.assume_init_ref(), self.value.assume_init_mut()) })
        } else {
            None
        }
    }

    fn has_value(&self) -> bool {
        self.distance_to_initial_bucket >= 0
    }

    fn take_key_value(&mut self) -> Option<(K, V, HashType)> {
        if self.has_value() {
            let key = mem::replace(&mut self.key, MaybeUninit::uninit());
            let value = mem::replace(&mut self.value, MaybeUninit::uninit());
            self.distance_to_initial_bucket = -1;

            Some(unsafe { (key.assume_init(), value.assume_init(), self.hash) })
        } else {
            None
        }
    }

    fn replace_value(&mut self, value: V) -> V {
        if self.has_value() {
            let old_value = mem::replace(&mut self.value, MaybeUninit::new(value));
            unsafe { old_value.assume_init() }
        } else {
            panic!("Cannot replace an unininitalised node");
        }
    }

    fn replace(&mut self, key: K, value: V) -> (K, V) {
        if self.has_value() {
            let old_key = mem::replace(&mut self.key, MaybeUninit::new(key));
            let old_value = mem::replace(&mut self.value, MaybeUninit::new(value));

            unsafe { (old_key.assume_init(), old_value.assume_init()) }
        } else {
            panic!("Cannot replace an uninitialised node");
        }
    }

    fn increment_distance(&mut self) {
        self.distance_to_initial_bucket += 1;
    }

    fn decrement_distance(&mut self) {
        self.distance_to_initial_bucket -= 1;
        if self.distance_to_initial_bucket < 0 {
            panic!("Cannot decrement distance to below 0");
        }
    }

    fn distance(&self) -> i32 {
        self.distance_to_initial_bucket
    }
}

impl<K, V> Drop for Node<K, V> {
    fn drop(&mut self) {
        if self.has_value() {
            unsafe { ptr::drop_in_place(self.key.as_mut_ptr()) };
            unsafe { ptr::drop_in_place(self.value.as_mut_ptr()) };
        }
    }
}

impl<K, V> Default for Node<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use core::cell::RefCell;

    use super::*;
    use crate::{rng::RandomNumberGenerator, Gba};

    #[test_case]
    fn can_store_and_retrieve_8_elements(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.insert(i, i % 4);
        }

        for i in 0..8 {
            assert_eq!(map.get(&i), Some(&(i % 4)));
        }
    }

    #[test_case]
    fn can_get_the_length(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.insert(i / 2, true);
        }

        assert_eq!(map.len(), 4);
    }

    #[test_case]
    fn returns_none_if_element_does_not_exist(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.insert(i, i % 3);
        }

        assert_eq!(map.get(&12), None);
    }

    #[test_case]
    fn can_delete_entries(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.insert(i, i % 3);
        }

        for i in 0..4 {
            map.remove(&i);
        }

        assert_eq!(map.len(), 4);
        assert_eq!(map.get(&3), None);
        assert_eq!(map.get(&7), Some(&1));
    }

    #[test_case]
    fn can_iterate_through_all_entries(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.insert(i, i);
        }

        let mut max_found = -1;
        let mut num_found = 0;

        for (_, value) in map.into_iter() {
            max_found = max_found.max(value);
            num_found += 1;
        }

        assert_eq!(num_found, 8);
        assert_eq!(max_found, 7);
    }

    #[test_case]
    fn can_insert_more_than_initial_capacity(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..65 {
            map.insert(i, i % 4);
        }

        for i in 0..65 {
            assert_eq!(map.get(&i), Some(&(i % 4)));
        }
    }

    struct NoisyDrop {
        i: i32,
        dropped: bool,
    }

    impl NoisyDrop {
        fn new(i: i32) -> Self {
            Self { i, dropped: false }
        }
    }

    impl PartialEq for NoisyDrop {
        fn eq(&self, other: &Self) -> bool {
            self.i == other.i
        }
    }

    impl Eq for NoisyDrop {}

    impl Hash for NoisyDrop {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            hasher.write_i32(self.i);
        }
    }

    impl Drop for NoisyDrop {
        fn drop(&mut self) {
            if self.dropped {
                panic!("NoisyDropped dropped twice");
            }

            self.dropped = true;
        }
    }

    #[test_case]
    fn extreme_case(_gba: &mut Gba) {
        let mut map = HashMap::new();
        let mut rng = RandomNumberGenerator::new();

        let mut answers: [Option<i32>; 128] = [None; 128];

        for _ in 0..5_000 {
            let command = rng.gen().rem_euclid(2);
            let key = rng.gen().rem_euclid(answers.len() as i32);
            let value = rng.gen();

            match command {
                0 => {
                    // insert
                    answers[key as usize] = Some(value);
                    map.insert(NoisyDrop::new(key), NoisyDrop::new(value));
                }
                1 => {
                    // remove
                    answers[key as usize] = None;
                    map.remove(&NoisyDrop::new(key));
                }
                _ => {}
            }

            for (i, answer) in answers.iter().enumerate() {
                assert_eq!(
                    map.get(&NoisyDrop::new(i as i32)).map(|nd| &nd.i),
                    answer.as_ref()
                );
            }
        }
    }

    #[derive(Clone)]
    struct Droppable<'a> {
        id: usize,
        drop_registry: &'a DropRegistry,
    }

    impl Hash for Droppable<'_> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            hasher.write_usize(self.id);
        }
    }

    impl PartialEq for Droppable<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Eq for Droppable<'_> {}

    impl Drop for Droppable<'_> {
        fn drop(&mut self) {
            self.drop_registry.dropped(self.id);
        }
    }

    struct DropRegistry {
        are_dropped: RefCell<Vec<i32>>,
    }

    impl DropRegistry {
        pub fn new() -> Self {
            Self {
                are_dropped: Default::default(),
            }
        }

        pub fn new_droppable(&self) -> Droppable<'_> {
            self.are_dropped.borrow_mut().push(0);
            Droppable {
                id: self.are_dropped.borrow().len() - 1,
                drop_registry: self,
            }
        }

        pub fn dropped(&self, id: usize) {
            self.are_dropped.borrow_mut()[id] += 1;
        }

        pub fn assert_dropped_once(&self, id: usize) {
            assert_eq!(self.are_dropped.borrow()[id], 1);
        }

        pub fn assert_not_dropped(&self, id: usize) {
            assert_eq!(self.are_dropped.borrow()[id], 0);
        }

        pub fn assert_dropped_n_times(&self, id: usize, num_drops: i32) {
            assert_eq!(self.are_dropped.borrow()[id], num_drops);
        }
    }

    #[test_case]
    fn correctly_drops_on_remove_and_overall_drop(_gba: &mut Gba) {
        let drop_registry = DropRegistry::new();

        let droppable1 = drop_registry.new_droppable();
        let droppable2 = drop_registry.new_droppable();

        let id1 = droppable1.id;
        let id2 = droppable2.id;

        {
            let mut map = HashMap::new();

            map.insert(1, droppable1);
            map.insert(2, droppable2);

            drop_registry.assert_not_dropped(id1);
            drop_registry.assert_not_dropped(id2);

            map.remove(&1);
            drop_registry.assert_dropped_once(id1);
            drop_registry.assert_not_dropped(id2);
        }

        drop_registry.assert_dropped_once(id2);
    }

    #[test_case]
    fn correctly_drop_on_override(_gba: &mut Gba) {
        let drop_registry = DropRegistry::new();

        let droppable1 = drop_registry.new_droppable();
        let droppable2 = drop_registry.new_droppable();

        let id1 = droppable1.id;
        let id2 = droppable2.id;

        {
            let mut map = HashMap::new();

            map.insert(1, droppable1);
            drop_registry.assert_not_dropped(id1);
            map.insert(1, droppable2);

            drop_registry.assert_dropped_once(id1);
            drop_registry.assert_not_dropped(id2);
        }

        drop_registry.assert_dropped_once(id2);
    }

    #[test_case]
    fn correctly_drops_key_on_override(_gba: &mut Gba) {
        let drop_registry = DropRegistry::new();

        let droppable1 = drop_registry.new_droppable();
        let droppable1a = droppable1.clone();

        let id1 = droppable1.id;

        {
            let mut map = HashMap::new();

            map.insert(droppable1, 1);
            drop_registry.assert_not_dropped(id1);
            map.insert(droppable1a, 2);

            drop_registry.assert_dropped_once(id1);
        }

        drop_registry.assert_dropped_n_times(id1, 2);
    }

    // Following test cases copied from the rust source
    // https://github.com/rust-lang/rust/blob/master/library/std/src/collections/hash/map/tests.rs
    mod rust_std_tests {
        use crate::{
            hash_map::{Entry::*, HashMap},
            Gba,
        };

        #[test_case]
        fn test_entry(_gba: &mut Gba) {
            let xs = [(1, 10), (2, 20), (3, 30), (4, 40), (5, 50), (6, 60)];

            let mut map: HashMap<_, _> = xs.iter().copied().collect();

            // Existing key (insert)
            match map.entry(1) {
                Vacant(_) => unreachable!(),
                Occupied(mut view) => {
                    assert_eq!(view.get(), &10);
                    assert_eq!(view.insert(100), 10);
                }
            }
            assert_eq!(map.get(&1).unwrap(), &100);
            assert_eq!(map.len(), 6);

            // Existing key (update)
            match map.entry(2) {
                Vacant(_) => unreachable!(),
                Occupied(mut view) => {
                    let v = view.get_mut();
                    let new_v = (*v) * 10;
                    *v = new_v;
                }
            }
            assert_eq!(map.get(&2).unwrap(), &200);
            assert_eq!(map.len(), 6);

            // Existing key (take)
            match map.entry(3) {
                Vacant(_) => unreachable!(),
                Occupied(view) => {
                    assert_eq!(view.remove(), 30);
                }
            }
            assert_eq!(map.get(&3), None);
            assert_eq!(map.len(), 5);

            // Inexistent key (insert)
            match map.entry(10) {
                Occupied(_) => unreachable!(),
                Vacant(view) => {
                    assert_eq!(*view.insert(1000), 1000);
                }
            }
            assert_eq!(map.get(&10).unwrap(), &1000);
            assert_eq!(map.len(), 6);
        }

        #[test_case]
        fn test_occupied_entry_key(_gba: &mut Gba) {
            let mut a = HashMap::new();
            let key = "hello there";
            let value = "value goes here";
            assert!(a.is_empty());
            a.insert(key, value);
            assert_eq!(a.len(), 1);
            assert_eq!(a[key], value);

            match a.entry(key) {
                Vacant(_) => panic!(),
                Occupied(e) => assert_eq!(key, *e.key()),
            }
            assert_eq!(a.len(), 1);
            assert_eq!(a[key], value);
        }

        #[test_case]
        fn test_vacant_entry_key(_gba: &mut Gba) {
            let mut a = HashMap::new();
            let key = "hello there";
            let value = "value goes here";

            assert!(a.is_empty());
            match a.entry(key) {
                Occupied(_) => panic!(),
                Vacant(e) => {
                    assert_eq!(key, *e.key());
                    e.insert(value);
                }
            }
            assert_eq!(a.len(), 1);
            assert_eq!(a[key], value);
        }

        #[test_case]
        fn test_index(_gba: &mut Gba) {
            let mut map = HashMap::new();

            map.insert(1, 2);
            map.insert(2, 1);
            map.insert(3, 4);

            assert_eq!(map[&2], 1);
        }
    }
}
