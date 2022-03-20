use alloc::vec::Vec;
use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
    iter,
    mem::{self, MaybeUninit},
    ptr,
};

use rustc_hash::FxHasher;

type HashType = u32;

struct Node<K, V> {
    hash: HashType,

    // distance_to_initial_bucket = -1 => key and value are uninit.
    // distance_to_initial_bucket >= 0 => key and value are init
    distance_to_initial_bucket: i32,
    key: MaybeUninit<K>,
    value: MaybeUninit<V>,
}

impl<K, V> Node<K, V> {
    fn with_new_key_value(self, new_key: K, new_value: V) -> (Self, Option<V>) {
        (
            Self {
                hash: self.hash,
                distance_to_initial_bucket: self.distance_to_initial_bucket,
                key: MaybeUninit::new(new_key),
                value: MaybeUninit::new(new_value),
            },
            self.get_owned_value(),
        )
    }

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

    fn get_value_ref(&self) -> Option<&V> {
        if self.has_value() {
            Some(unsafe { self.value.assume_init_ref() })
        } else {
            None
        }
    }

    fn get_value_mut(&mut self) -> Option<&mut V> {
        if self.has_value() {
            Some(unsafe { self.value.assume_init_mut() })
        } else {
            None
        }
    }

    fn get_owned_value(mut self) -> Option<V> {
        if self.has_value() {
            let value = mem::replace(&mut self.value, MaybeUninit::uninit());
            self.distance_to_initial_bucket = -1;

            Some(unsafe { value.assume_init() })
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

    fn has_value(&self) -> bool {
        self.distance_to_initial_bucket >= 0
    }

    fn take(&mut self) -> Self {
        mem::take(self)
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
}

impl<K, V> Drop for Node<K, V> {
    fn drop(&mut self) {
        if self.distance_to_initial_bucket >= 0 {
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

struct NodeStorage<K, V> {
    nodes: Vec<Node<K, V>>,
    max_distance_to_initial_bucket: i32,

    number_of_items: usize,
}

impl<K, V> NodeStorage<K, V> {
    fn with_size(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be a power of 2");

        Self {
            nodes: iter::repeat_with(Default::default).take(capacity).collect(),
            max_distance_to_initial_bucket: 0,
            number_of_items: 0,
        }
    }

    fn capacity(&self) -> usize {
        self.nodes.len()
    }

    fn len(&self) -> usize {
        self.number_of_items
    }

    fn insert_new(&mut self, key: K, value: V, hash: HashType) {
        debug_assert!(
            self.capacity() * 85 / 100 > self.len(),
            "Do not have space to insert into len {} with {}",
            self.capacity(),
            self.len()
        );

        let mut new_node = Node::new_with(key, value, hash);

        loop {
            let location = fast_mod(
                self.capacity(),
                new_node.hash + new_node.distance_to_initial_bucket as HashType,
            );
            let current_node = &mut self.nodes[location];

            if current_node.has_value() {
                if current_node.distance_to_initial_bucket <= new_node.distance_to_initial_bucket {
                    mem::swap(&mut new_node, current_node);
                }
            } else {
                self.nodes[location] = new_node;
                break;
            }

            new_node.distance_to_initial_bucket += 1;
            self.max_distance_to_initial_bucket = new_node
                .distance_to_initial_bucket
                .max(self.max_distance_to_initial_bucket);
        }

        self.number_of_items += 1;
    }

    fn remove_from_location(&mut self, location: usize) -> V {
        let mut current_location = location;
        self.number_of_items -= 1;

        loop {
            let next_location = fast_mod(self.capacity(), (current_location + 1) as HashType);

            // if the next node is empty, or the next location has 0 distance to initial bucket then
            // we can clear the current node
            if !self.nodes[next_location].has_value()
                || self.nodes[next_location].distance_to_initial_bucket == 0
            {
                return self.nodes[current_location].take_key_value().unwrap().1;
            }

            self.nodes.swap(current_location, next_location);
            self.nodes[current_location].distance_to_initial_bucket -= 1;
            current_location = next_location;
        }
    }

    fn get_location(&self, key: &K, hash: HashType) -> Option<usize>
    where
        K: Eq,
    {
        for distance_to_initial_bucket in 0..=self.max_distance_to_initial_bucket {
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
}

pub struct HashMap<K, V, H = BuildHasherDefault<FxHasher>>
where
    H: BuildHasher,
{
    nodes: NodeStorage<K, V>,

    hasher: H,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: NodeStorage::with_size(capacity),
            hasher: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn resize(&mut self, new_size: usize) {
        assert!(
            new_size >= self.nodes.capacity(),
            "Can only increase the size of a hash map"
        );
        if new_size == self.nodes.capacity() {
            return;
        }

        let mut new_node_storage = NodeStorage::with_size(new_size);

        for mut node in self.nodes.nodes.drain(..) {
            if let Some((key, value, hash)) = node.take_key_value() {
                new_node_storage.insert_new(key, value, hash);
            }
        }

        self.nodes = new_node_storage;
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hash = self.hash(&key);

        if let Some(location) = self.nodes.get_location(&key, hash) {
            let old_node = self.nodes.nodes[location].take();
            let (new_node, old_value) = old_node.with_new_key_value(key, value);
            self.nodes.nodes[location] = new_node;

            return old_value;
        }

        if self.nodes.capacity() * 85 / 100 <= self.len() {
            self.resize(self.nodes.capacity() * 2);
        }

        self.nodes.insert_new(key, value, hash);
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);

        self.nodes
            .get_location(key, hash)
            .and_then(|location| self.nodes.nodes[location].get_value_ref())
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash(key);

        if let Some(location) = self.nodes.get_location(key, hash) {
            self.nodes.nodes[location].get_value_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);

        self.nodes
            .get_location(key, hash)
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

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.at >= self.map.nodes.capacity() {
                return None;
            }

            let node = &self.map.nodes.nodes[self.at];
            self.at += 1;

            if node.has_value() {
                return Some((node.key_ref().unwrap(), node.get_value_ref().unwrap()));
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

pub struct OccupiedEntry<'a, K: 'a, V: 'a> {
    entry: &'a mut Node<K, V>,
}

pub struct VacantEntry<'a, K: 'a, V: 'a> {
    key: K,
    map: &'a mut HashMap<K, V>,
}

impl<'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V)
    where
        K: Hash + Eq,
    {
        self.map.insert(self.key, value);
    }
}

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>),
}

impl<'a, K, V> Entry<'a, K, V>
where
    K: Hash + Eq,
{
    pub fn or_insert(self, value: V) {
        match self {
            Entry::Occupied(_) => {}
            Entry::Vacant(e) => e.insert(value),
        }
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(e) => {
                f(e.entry.get_value_mut().unwrap());
                Entry::Occupied(e)
            }
            Entry::Vacant(e) => Entry::Vacant(e),
        }
    }
}

impl<'a, K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        let hash = self.hash(&key);
        let location = self.nodes.get_location(&key, hash);

        if let Some(location) = location {
            Entry::Occupied(OccupiedEntry {
                entry: &mut self.nodes.nodes[location],
            })
        } else {
            Entry::Vacant(VacantEntry { key, map: self })
        }
    }
}

#[cfg(test)]
mod test {
    use core::cell::RefCell;

    use super::*;
    use crate::Gba;

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
            max_found = max_found.max(*value);
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

    struct RandomNumberGenerator {
        state: [u32; 4],
    }

    impl RandomNumberGenerator {
        const fn new() -> Self {
            Self {
                state: [1014776995, 476057059, 3301633994, 706340607],
            }
        }

        fn next(&mut self) -> i32 {
            let result = (self.state[0].wrapping_add(self.state[3]))
                .rotate_left(7)
                .wrapping_mul(9);
            let t = self.state[1].wrapping_shr(9);

            self.state[2] ^= self.state[0];
            self.state[3] ^= self.state[1];
            self.state[1] ^= self.state[2];
            self.state[0] ^= self.state[3];

            self.state[2] ^= t;
            self.state[3] = self.state[3].rotate_left(11);

            result as i32
        }
    }

    #[test_case]
    fn extreme_case(_gba: &mut Gba) {
        let mut map = HashMap::new();
        let mut rng = RandomNumberGenerator::new();

        let mut answers: [Option<i32>; 128] = [None; 128];

        for _ in 0..5_000 {
            let command = rng.next().rem_euclid(2);
            let key = rng.next().rem_euclid(128);
            let value = rng.next();

            match command {
                0 => {
                    // insert
                    answers[key as usize] = Some(value);
                    map.insert(key, value);
                }
                1 => {
                    // remove
                    answers[key as usize] = None;
                    map.remove(&key);
                }
                _ => {}
            }

            for (i, answer) in answers.iter().enumerate() {
                assert_eq!(map.get(&(i as i32)), answer.as_ref());
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
}
