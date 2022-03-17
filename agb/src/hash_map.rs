use alloc::{vec, vec::Vec};
use core::{
    hash::{BuildHasher, BuildHasherDefault, Hash, Hasher},
    mem,
};

use rustc_hash::FxHasher;

type HashType = u32;

struct Node<K: Sized, V: Sized> {
    hash: HashType,
    distance_to_initial_bucket: u32,
    key: K,
    value: V,
}

impl<K, V> Node<K, V>
where
    K: Sized,
    V: Sized,
{
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

pub struct HashMap<K, V> {
    number_of_elements: usize,
    max_distance_to_initial_bucket: u32,
    nodes: Vec<Option<Node<K, V>>>,

    hasher: BuildHasherDefault<FxHasher>,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        Self {
            number_of_elements: 0,
            max_distance_to_initial_bucket: 0,
            nodes: vec![
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
            hasher: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.number_of_elements
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
            let location = fast_mod(self.nodes.len(), hash + distance_to_initial_bucket);

            let node = &self.nodes[location];
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
    pub fn put(&mut self, key: K, value: V) -> Option<V> {
        let hash = self.hash(&key);

        if let Some(location) = self.get_location(&key, hash) {
            let old_node = self.nodes[location].take().unwrap();
            let (new_node, old_value) = old_node.with_new_key_value(key, value);
            self.nodes[location] = Some(new_node);

            return Some(old_value);
        }

        self.insert_new(key, value, hash);
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);

        self.get_location(key, hash)
            .map(|location| &self.nodes[location].as_ref().unwrap().value)
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let hash = self.hash(key);

        self.get_location(key, hash)
            .map(|location| self.remove_from_location(location))
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

impl<K, V> HashMap<K, V> {
    fn insert_new(&mut self, key: K, value: V, hash: HashType) {
        // if we need to resize
        if self.nodes.len() * 85 / 100 < self.number_of_elements {
            todo!("resize not implemented yet");
        }

        let mut new_node = Node {
            hash,
            distance_to_initial_bucket: 0,
            key,
            value,
        };

        loop {
            let location = fast_mod(self.nodes.len(), hash + new_node.distance_to_initial_bucket);
            let current_node = self.nodes[location].as_mut();

            if let Some(current_node) = current_node {
                if current_node.distance_to_initial_bucket <= new_node.distance_to_initial_bucket {
                    self.max_distance_to_initial_bucket = new_node
                        .distance_to_initial_bucket
                        .max(self.max_distance_to_initial_bucket);

                    mem::swap(&mut new_node, current_node);
                }
            } else {
                self.nodes[location] = Some(new_node);
                break;
            }

            new_node.distance_to_initial_bucket += 1;
        }

        self.number_of_elements += 1;
    }

    fn remove_from_location(&mut self, location: usize) -> V {
        let mut current_location = location;

        let result = loop {
            let next_location = fast_mod(self.nodes.len(), (current_location + 1) as HashType);

            // if the next node is empty, then we can clear the current node
            if self.nodes[next_location].is_none() {
                break self.nodes[current_location].take().unwrap();
            }

            self.nodes.swap(current_location, next_location);
            self.nodes[current_location]
                .as_mut()
                .unwrap()
                .distance_to_initial_bucket -= 1;
            current_location = next_location;
        };

        self.number_of_elements -= 1;
        result.value
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

            if let Some(node) = &self.map.nodes[self.at] {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::Gba;

    #[test_case]
    fn can_store_and_retrieve_8_elements(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.put(i, i % 4);
        }

        for i in 0..8 {
            assert_eq!(map.get(&i), Some(&(i % 4)));
        }
    }

    #[test_case]
    fn can_get_the_length(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.put(i / 2, true);
        }

        assert_eq!(map.len(), 4);
    }

    #[test_case]
    fn returns_none_if_element_does_not_exist(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.put(i, i % 3);
        }

        assert_eq!(map.get(&12), None);
    }

    #[test_case]
    fn can_delete_entries(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.put(i, i % 3);
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
            map.put(i, i);
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
}
