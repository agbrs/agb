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
    fn with_new_key_value(&self, new_key: K, new_value: V) -> Self {
        Self {
            hash: self.hash,
            distance_to_initial_bucket: self.distance_to_initial_bucket,
            key: new_key,
            value: new_value,
        }
    }
}

struct HashMap<K, V> {
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
}

fn fast_mod(len: usize, hash: HashType) -> usize {
    debug_assert!(len.is_power_of_two(), "Length must be a power of 2");
    (hash as usize) & (len - 1)
}

impl<K, V> HashMap<K, V>
where
    K: Eq,
{
    fn get_location(&self, key: &K, hash: HashType) -> Option<usize> {
        for distance_to_initial_bucket in 0..=self.max_distance_to_initial_bucket + 1 {
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
    pub fn put(&mut self, key: K, value: V) {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish() as HashType;

        if let Some(location) = self.get_location(&key, hash) {
            let old_node = self.nodes[location].as_ref().unwrap();
            self.nodes[location] = Some(old_node.with_new_key_value(key, value));

            return;
        }

        self.insert_new(key, value, hash);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish() as HashType;

        self.get_location(&key, hash)
            .map(|location| &self.nodes[location].as_ref().unwrap().value)
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Gba;

    #[test_case]
    fn can_store_up_to_initial_capacity_elements(_gba: &mut Gba) {
        let mut map = HashMap::new();

        for i in 0..8 {
            map.put(i, i % 4);
        }

        for i in 0..8 {
            assert_eq!(map.get(&i), Some(&(i % 4)));
        }
    }
}
