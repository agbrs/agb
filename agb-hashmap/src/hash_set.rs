use crate::{Allocator, ClonableAllocator, Global};

use core::{borrow::Borrow, fmt::Debug, hash::Hash};

use super::HashMap;

/// A `HashSet` is implemented as a [`HashMap`] where the value is `()`.
///
/// As with the [`HashMap`] type, a `HashSet` requires that the elements implement the
/// [`Eq`] and [`Hash`] traits, although this is frequently achieved by using
/// `#[derive(PartialEq, Eq, Hash)]`. If you implement these yourself, it is important
/// that the following property holds:
///
/// It is a logic error for the key to be modified in such a way that the key's hash, as
/// determined by the [`Hash`] trait, or its equality as determined by the [`Eq`] trait,
/// changes while it is in the map. The behaviour for such a logic error is not specified,
/// but will not result in undefined behaviour. This could include panics, incorrect results,
/// aborts, memory leaks and non-termination.
///
/// The API surface provided is incredibly similar to the
/// [`std::collections::HashSet`](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
/// implementation with fewer guarantees, and better optimised for the `GameBoy Advance`.
///
/// [`Eq`]: https://doc.rust-lang.org/core/cmp/trait.Eq.html
/// [`Hash`]: https://doc.rust-lang.org/core/hash/trait.Hash.html
///
/// # Example
///
/// ```
/// use agb_hashmap::HashSet;
///
/// // Type inference lets you omit the type signature (which would be HashSet<String> in this example)
/// let mut games = HashSet::new();
///
/// // Add some games
/// games.insert("Pokemon Emerald".to_string());
/// games.insert("Golden Sun".to_string());
/// games.insert("Super Dodge Ball Advance".to_string());
///
/// // Check for a specific game
/// if !games.contains("Legend of Zelda: The Minish Cap") {
///     println!("We've got {} games, but The Minish Cap ain't one", games.len());
/// }
///
/// // Remove a game
/// games.remove("Golden Sun");
///
/// // Iterate over everything
/// for game in &games {
///     println!("{game}");
/// }
/// ```
#[derive(Clone)]
pub struct HashSet<K, ALLOCATOR: Allocator = Global> {
    map: HashMap<K, (), ALLOCATOR>,
}

impl<K> HashSet<K> {
    /// Creates a `HashSet`
    #[must_use]
    pub const fn new() -> Self {
        Self::new_in(Global)
    }

    /// Creates an empty `HashSet` with specified internal size. The size must be a power of 2
    #[must_use]
    pub fn with_size(size: usize) -> Self {
        Self::with_size_in(size, Global)
    }

    /// Creates an empty `HashSet` which can hold at least `capacity` elements before resizing. The actual
    /// internal size may be larger as it must be a power of 2
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_in(capacity, Global)
    }
}

impl<K, ALLOCATOR: ClonableAllocator> HashSet<K, ALLOCATOR> {
    /// Creates an empty `HashSet` with specified internal size using the specified allocator.
    /// The size must be a power of 2
    #[must_use]
    pub fn with_size_in(size: usize, alloc: ALLOCATOR) -> Self {
        Self {
            map: HashMap::with_size_in(size, alloc),
        }
    }

    /// Creates a `HashSet` with a specified allocator
    #[must_use]
    pub const fn new_in(alloc: ALLOCATOR) -> Self {
        Self {
            map: HashMap::new_in(alloc),
        }
    }

    /// Creates an empty `HashSet` which can hold at least `capacity` elements before resizing. The actual
    /// internal size may be larger as it must be a power of 2
    ///
    /// # Panics
    ///
    /// Panics if capacity >= 2^31 * 0.6
    #[must_use]
    pub fn with_capacity_in(capacity: usize, alloc: ALLOCATOR) -> Self {
        Self {
            map: HashMap::with_capacity_in(capacity, alloc),
        }
    }

    /// Returns a reference to the underlying allocator
    pub fn allocator(&self) -> &ALLOCATOR {
        self.map.allocator()
    }

    /// Returns the number of elements in the set
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns whether or not the set is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the number of elements the set can hold without resizing
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Removes all elements from the set
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// An iterator visiting all the values in the set
    pub fn iter(&self) -> impl Iterator<Item = &'_ K> {
        self.map.keys()
    }

    /// Retains only the elements specified by the predicate `f`
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&K) -> bool,
    {
        self.map.retain(|k, _| f(k));
    }
}

impl<K> Default for HashSet<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K, ALLOCATOR: ClonableAllocator> HashSet<K, ALLOCATOR>
where
    K: Eq + Hash,
{
    /// Inserts a value into the set. This does not replace the value if it already existed.
    ///
    /// Returns whether the value was newly inserted, that is:
    ///
    /// * If the set did not previously contain this value, `true` is returned
    /// * If the set already contained this value, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let mut set = HashSet::new();
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, value: K) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Removes a value from the set. Returns whether the value was present in the set.
    ///
    /// # Examples
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let mut set = HashSet::new();
    /// set.insert(2);
    ///
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// assert!(set.is_empty());
    /// ```
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.remove(value).is_some()
    }

    /// Returns `true` if the set contains the value `value`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let set = HashSet::from([1, 2, 3]);
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(value)
    }

    /// Visits the values representing the difference i.e. the values that are in `self` but not in `other`.
    ///
    /// # Examples
    ///
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let a = HashSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// // Can be seen as `a - b`
    /// let diff: HashSet<_> = a.difference(&b).collect();
    /// assert_eq!(diff, HashSet::from([&1]));
    ///
    /// // Difference is not symmetric. `b - a` means something different
    /// let diff: HashSet<_> = b.difference(&a).collect();
    /// assert_eq!(diff, HashSet::from([&4]));
    /// ``````
    pub fn difference<'a>(
        &'a self,
        other: &'a HashSet<K, ALLOCATOR>,
    ) -> impl Iterator<Item = &'a K> {
        self.iter().filter(|k| !other.contains(k))
    }

    /// Visits the values which are in `self` or `other` but not both.
    ///
    /// # Examples
    ///
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let a = HashSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// let diff1: HashSet<_> = a.symmetric_difference(&b).collect();
    /// let diff2: HashSet<_> = b.symmetric_difference(&a).collect();
    ///
    /// assert_eq!(diff1, diff2);
    /// assert_eq!(diff1, HashSet::from([&1, &4]));
    /// ```
    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a HashSet<K, ALLOCATOR>,
    ) -> impl Iterator<Item = &'a K> {
        self.iter()
            .filter(|k| !other.contains(k))
            .chain(other.iter().filter(|k| !self.contains(k)))
    }

    /// Visits the values in the intersection of `self` and `other`.
    ///
    /// When an equal element is present in `self` and `other`, then the resulting intersection may
    /// yield references to one or the other. This can be relevant if `K` contains fields which are not
    /// covered by the `Eq` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let a = HashSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// let intersection: HashSet<_> = a.intersection(&b).collect();
    /// assert_eq!(intersection, HashSet::from([&2, &3]));
    /// ```
    pub fn intersection<'a>(
        &'a self,
        other: &'a HashSet<K, ALLOCATOR>,
    ) -> impl Iterator<Item = &'a K> {
        let (smaller, larger) = if self.len() < other.len() {
            (self, other)
        } else {
            (other, self)
        };

        smaller.iter().filter(|k| larger.contains(k))
    }

    /// Visits the values in self and other without duplicates.
    ///
    /// When an equal element is present in `self` and `other`, then the resulting union may
    /// yield references to one or the other. This can be relevant if `K` contains fields which are not
    /// covered by the `Eq` implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use agb_hashmap::HashSet;
    ///
    /// let a = HashSet::from([1, 2, 3]);
    /// let b = HashSet::from([4, 2, 3, 4]);
    ///
    /// let union: Vec<_> = a.union(&b).collect();
    /// assert_eq!(union.len(), 4);
    /// assert_eq!(HashSet::from_iter(union), HashSet::from([&1, &2, &3, &4]));
    /// ```
    pub fn union<'a>(&'a self, other: &'a HashSet<K, ALLOCATOR>) -> impl Iterator<Item = &'a K> {
        let (smaller, larger) = if self.len() < other.len() {
            (self, other)
        } else {
            (other, self)
        };

        larger.iter().chain(smaller.difference(self))
    }
}

impl<K, ALLOCATOR: ClonableAllocator> IntoIterator for HashSet<K, ALLOCATOR> {
    type Item = K;
    type IntoIter = IterOwned<K, ALLOCATOR>;

    fn into_iter(self) -> Self::IntoIter {
        IterOwned {
            map_iter: self.map.into_iter(),
        }
    }
}

/// An iterator over the entries of a [`HashSet`].
///
/// This struct is created using the `into_iter()` method on [`HashSet`] as part of its implementation
/// of the `IntoIterator` trait.
pub struct IterOwned<K, ALLOCATOR: ClonableAllocator> {
    map_iter: super::IterOwned<K, (), ALLOCATOR>,
}

impl<K, ALLOCATOR: ClonableAllocator> Iterator for IterOwned<K, ALLOCATOR> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.map_iter.size_hint()
    }
}

impl<K, ALLOCATOR: ClonableAllocator> ExactSizeIterator for IterOwned<K, ALLOCATOR> {}

impl<'a, K, ALLOCATOR: ClonableAllocator> IntoIterator for &'a HashSet<K, ALLOCATOR> {
    type Item = &'a K;
    type IntoIter = Iter<'a, K, ALLOCATOR>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map_iter: (&self.map).into_iter(),
        }
    }
}

pub struct Iter<'a, K, ALLOCATOR: ClonableAllocator> {
    map_iter: super::Iter<'a, K, (), ALLOCATOR>,
}

impl<'a, K, ALLOCATOR: ClonableAllocator> Iterator for Iter<'a, K, ALLOCATOR> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        self.map_iter.next().map(|(k, _)| k)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.map_iter.size_hint()
    }
}

impl<K, ALLOCATOR: ClonableAllocator> ExactSizeIterator for Iter<'_, K, ALLOCATOR> {}

impl<K> FromIterator<K> for HashSet<K>
where
    K: Eq + Hash,
{
    fn from_iter<T: IntoIterator<Item = K>>(iter: T) -> Self {
        let mut set = HashSet::new();
        set.extend(iter);
        set
    }
}

impl<K> Extend<K> for HashSet<K>
where
    K: Eq + Hash,
{
    fn extend<T: IntoIterator<Item = K>>(&mut self, iter: T) {
        for k in iter {
            self.insert(k);
        }
    }
}

impl<K, ALLOCATOR: ClonableAllocator> PartialEq for HashSet<K, ALLOCATOR>
where
    K: Eq + Hash,
{
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl<K, ALLOCATOR: ClonableAllocator> Eq for HashSet<K, ALLOCATOR> where K: Eq + Hash {}

impl<K, ALLOCATOR: ClonableAllocator> Debug for HashSet<K, ALLOCATOR>
where
    K: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<K, const N: usize> From<[K; N]> for HashSet<K>
where
    K: Eq + Hash,
{
    fn from(value: [K; N]) -> Self {
        HashSet::from_iter(value)
    }
}
