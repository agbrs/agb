use core::{alloc::Allocator, borrow::Borrow, mem};

use alloc::{alloc::Global, vec::Vec};

use crate::{node::Node, number_before_resize, ClonableAllocator, HashType};

#[derive(Clone)]
pub(crate) struct NodeStorage<K, V, ALLOCATOR: Allocator = Global> {
    nodes: Vec<Node<K, V>, ALLOCATOR>,
    max_distance_to_initial_bucket: i32,

    number_of_items: usize,
    max_number_before_resize: usize,
}

impl<K, V, ALLOCATOR: ClonableAllocator> NodeStorage<K, V, ALLOCATOR> {
    pub(crate) fn with_size_in(capacity: usize, alloc: ALLOCATOR) -> Self {
        assert!(capacity.is_power_of_two(), "Capacity must be a power of 2");

        let mut nodes = Vec::with_capacity_in(capacity, alloc);
        for _ in 0..capacity {
            nodes.push(Node::default());
        }

        Self {
            nodes,
            max_distance_to_initial_bucket: 0,
            number_of_items: 0,
            max_number_before_resize: number_before_resize(capacity),
        }
    }

    pub(crate) fn allocator(&self) -> &ALLOCATOR {
        self.nodes.allocator()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.max_number_before_resize
    }

    pub(crate) fn backing_vec_size(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn len(&self) -> usize {
        self.number_of_items
    }

    pub(crate) fn insert_new(&mut self, key: K, value: V, hash: HashType) -> usize {
        debug_assert!(
            self.capacity() > self.len(),
            "Do not have space to insert into len {} with {}",
            self.backing_vec_size(),
            self.len()
        );

        let mut new_node = Node::new_with(key, value, hash);
        let mut inserted_location = usize::MAX;

        loop {
            let location =
                (new_node.hash() + new_node.distance()).fast_mod(self.backing_vec_size());

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

    pub(crate) fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        let num_nodes = self.nodes.len();
        let mut i = 0;

        while i < num_nodes {
            let node = &mut self.nodes[i];

            if let Some((k, v)) = node.key_value_mut() {
                if !f(k, v) {
                    self.remove_from_location(i);

                    // Need to continue before adding 1 to i because remove from location could
                    // put the element which was next into the ith location in the nodes array,
                    // so we need to check if that one needs removing too.
                    continue;
                }
            }

            i += 1;
        }
    }

    pub(crate) fn remove_from_location(&mut self, location: usize) -> V {
        let mut current_location = location;
        self.number_of_items -= 1;

        loop {
            let next_location =
                HashType::from(current_location + 1).fast_mod(self.backing_vec_size());

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

    pub(crate) fn location<Q>(&self, key: &Q, hash: HashType) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Eq + ?Sized,
    {
        for distance_to_initial_bucket in 0..(self.max_distance_to_initial_bucket + 1) {
            let location = (hash + distance_to_initial_bucket).fast_mod(self.nodes.len());

            let node = &self.nodes[location];
            let node_key_ref = node.key_ref()?;

            if node_key_ref.borrow() == key {
                return Some(location);
            }
        }

        None
    }

    pub(crate) fn resized_to(&mut self, new_size: usize) -> Self {
        let mut new_node_storage = Self::with_size_in(new_size, self.allocator().clone());

        for mut node in self.nodes.drain(..) {
            if let Some((key, value, hash)) = node.take_key_value() {
                new_node_storage.insert_new(key, value, hash);
            }
        }

        new_node_storage
    }

    pub(crate) fn replace_at_location(&mut self, location: usize, key: K, value: V) -> V {
        self.nodes[location].replace(key, value).1
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut Node<K, V>> {
        self.nodes.iter_mut()
    }

    pub(crate) fn node_at(&self, at: usize) -> &Node<K, V> {
        &self.nodes[at]
    }

    pub(crate) fn node_at_mut(&mut self, at: usize) -> &mut Node<K, V> {
        &mut self.nodes[at]
    }

    pub(crate) unsafe fn node_at_unchecked(&self, at: usize) -> &Node<K, V> {
        self.nodes.get_unchecked(at)
    }

    pub(crate) unsafe fn node_at_unchecked_mut(&mut self, at: usize) -> &mut Node<K, V> {
        self.nodes.get_unchecked_mut(at)
    }
}
