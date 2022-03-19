use alloc::vec::Vec;
use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
    iter, mem,
};

use rustc_hash::FxHasher;

type HashType = u32;

struct Node<K, V> {
    hash: HashType,
    distance_to_initial_bucket: i32,
    key: K,
    value: V,
}

impl<K, V> Node<K, V> {
    fn with_new_key_value(self, new_key: K, new_value: V) -> (Self, V) {
        (
            Self {
                hash: self.hash,
                distance_to_initial_bucket: self.distance_to_initial_bucket,
                key: new_key,
                value: new_value,
            },
            self.value,
        )
    }
}

struct NodeStorage<K, V>(Vec<Option<Node<K, V>>>);

impl<K, V> NodeStorage<K, V> {
    fn with_size(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be a power of 2");

        Self(iter::repeat_with(|| None).take(capacity).collect())
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn insert_new(
        &mut self,
        key: K,
        value: V,
        hash: HashType,
        number_of_elements: usize,
        max_distance_to_initial_bucket: i32,
    ) -> i32 {
        debug_assert!(
            self.len() * 85 / 100 > number_of_elements,
            "Do not have space to insert into len {} with {number_of_elements}",
            self.len()
        );

        let mut new_node = Node {
            hash,
            distance_to_initial_bucket: 0,
            key,
            value,
        };

        let mut max_distance_to_initial_bucket = max_distance_to_initial_bucket;

        loop {
            let location = fast_mod(
                self.len(),
                new_node.hash + new_node.distance_to_initial_bucket as HashType,
            );
            let current_node = self.0[location].as_mut();

            if let Some(current_node) = current_node {
                if current_node.distance_to_initial_bucket <= new_node.distance_to_initial_bucket {
                    mem::swap(&mut new_node, current_node);
                }
            } else {
                self.0[location] = Some(new_node);
                break;
            }

            new_node.distance_to_initial_bucket += 1;
            max_distance_to_initial_bucket = new_node
                .distance_to_initial_bucket
                .max(max_distance_to_initial_bucket);
        }

        max_distance_to_initial_bucket
    }

    fn remove_from_location(&mut self, location: usize) -> V {
        let mut current_location = location;

        let result = loop {
            let next_location = fast_mod(self.len(), (current_location + 1) as HashType);

            // if the next node is empty, or the next location has 0 distance to initial bucket then
            // we can clear the current node
            if self.0[next_location].is_none()
                || self.0[next_location]
                    .as_ref()
                    .unwrap()
                    .distance_to_initial_bucket
                    == 0
            {
                break self.0[current_location].take().unwrap();
            }
            if self.0[next_location].is_none() {}

            self.0.swap(current_location, next_location);
            self.0[current_location]
                .as_mut()
                .unwrap()
                .distance_to_initial_bucket -= 1;
            current_location = next_location;
        };

        result.value
    }
}

pub struct HashMap<K, V> {
    number_of_elements: usize,
    max_distance_to_initial_bucket: i32,
    nodes: NodeStorage<K, V>,

    hasher: BuildHasherDefault<FxHasher>,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            number_of_elements: 0,
            max_distance_to_initial_bucket: 0,
            nodes: NodeStorage::with_size(capacity),
            hasher: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.number_of_elements
    }

    pub fn is_empty(&self) -> bool {
        self.number_of_elements == 0
    }

    pub fn resize(&mut self, new_size: usize) {
        assert!(
            new_size >= self.nodes.len(),
            "Can only increase the size of a hash map"
        );
        if new_size == self.nodes.len() {
            return;
        }

        let mut new_node_storage = NodeStorage::with_size(new_size);
        let mut new_max_distance_to_initial_bucket = 0;
        let number_of_elements = self.number_of_elements;

        for node in self.nodes.0.drain(..).flatten() {
            new_max_distance_to_initial_bucket = new_node_storage.insert_new(
                node.key,
                node.value,
                node.hash,
                number_of_elements,
                new_max_distance_to_initial_bucket,
            );
        }

        self.nodes = new_node_storage;
        self.max_distance_to_initial_bucket = new_max_distance_to_initial_bucket;
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
    K: Eq,
{
    fn get_location(&self, key: &K, hash: HashType) -> Option<usize> {
        for distance_to_initial_bucket in 0..=self.max_distance_to_initial_bucket {
            let location = fast_mod(
                self.nodes.len(),
                hash + distance_to_initial_bucket as HashType,
            );

            let node = &self.nodes.0[location];
            if let Some(node) = node {
                if &node.key == key {
                    return Some(location);
                }
            } else {
                return None;
            }
        }

        None
    }
}

impl<K, V> HashMap<K, V>
where
    K: Eq + Hash,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let hash = self.hash(&key);

        if let Some(location) = self.get_location(&key, hash) {
            let old_node = self.nodes.0[location].take().unwrap();
            let (new_node, old_value) = old_node.with_new_key_value(key, value);
            self.nodes.0[location] = Some(new_node);

            return Some(old_value);
        }

        if self.nodes.len() * 85 / 100 <= self.number_of_elements {
            self.resize(self.nodes.len() * 2);
        }

        self.max_distance_to_initial_bucket = self.nodes.insert_new(
            key,
            value,
            hash,
            self.number_of_elements,
            self.max_distance_to_initial_bucket,
        );
        self.number_of_elements += 1;
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);

        self.get_location(key, hash)
            .map(|location| &self.nodes.0[location].as_ref().unwrap().value)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let hash = self.hash(key);

        if let Some(location) = self.get_location(key, hash) {
            Some(&mut self.nodes.0[location].as_mut().unwrap().value)
        } else {
            None
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);

        self.get_location(key, hash)
            .map(|location| self.nodes.remove_from_location(location))
            .map(|value| {
                self.number_of_elements -= 1;
                value
            })
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
            if self.at >= self.map.nodes.len() {
                return None;
            }

            if let Some(node) = &self.map.nodes.0[self.at] {
                self.at += 1;
                return Some((&node.key, &node.value));
            }

            self.at += 1;
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
                f(&mut e.entry.value);
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
        let location = self.get_location(&key, hash);

        if let Some(location) = location {
            Entry::Occupied(OccupiedEntry {
                entry: self.nodes.0[location].as_mut().unwrap(),
            })
        } else {
            Entry::Vacant(VacantEntry { key, map: self })
        }
    }
}

#[cfg(test)]
mod test {
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
}
