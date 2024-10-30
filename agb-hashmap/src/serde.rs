use core::{hash::Hash, marker::PhantomData};
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};

use crate::{ClonableAllocator, HashMap, HashSet};

mod hashmap {
    use super::*;

    impl<K: Serialize, V: Serialize, ALLOCATOR: ClonableAllocator> Serialize
        for HashMap<K, V, ALLOCATOR>
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(Some(self.len()))?;

            for (key, value) in self {
                map.serialize_entry(key, value)?;
            }

            map.end()
        }
    }

    impl<'de, K, V> Deserialize<'de> for HashMap<K, V>
    where
        K: Deserialize<'de> + Hash + Eq,
        V: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_map(HashMapVisitor::new())
        }
    }

    struct HashMapVisitor<K, V> {
        _marker: PhantomData<fn() -> HashMap<K, V>>,
    }

    impl<K, V> HashMapVisitor<K, V> {
        fn new() -> Self {
            Self {
                _marker: PhantomData,
            }
        }
    }

    impl<'de, K, V> Visitor<'de> for HashMapVisitor<K, V>
    where
        K: Deserialize<'de> + Hash + Eq,
        V: Deserialize<'de>,
    {
        type Value = HashMap<K, V>;

        fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
            formatter.write_str("an agb::HashMap")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(8));

            while let Some((key, value)) = access.next_entry()? {
                map.insert(key, value);
            }

            Ok(map)
        }
    }
}

mod hashset {

    use super::*;

    impl<K: Serialize, ALLOCATOR: ClonableAllocator> Serialize for HashSet<K, ALLOCATOR> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_seq(self)
        }
    }

    struct HashSetVisitor<K> {
        _marker: PhantomData<fn() -> HashSet<K>>,
    }

    impl<K> HashSetVisitor<K> {
        fn new() -> Self {
            Self {
                _marker: PhantomData,
            }
        }
    }

    impl<'de, K> Visitor<'de> for HashSetVisitor<K>
    where
        K: Deserialize<'de> + Hash + Eq,
    {
        type Value = HashSet<K>;

        fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
            formatter.write_str("an agb::HashSet")
        }

        fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut set = HashSet::with_capacity(access.size_hint().unwrap_or(8));

            while let Some(value) = access.next_element()? {
                set.insert(value);
            }

            Ok(set)
        }
    }

    impl<'de, K> Deserialize<'de> for HashSet<K>
    where
        K: Deserialize<'de> + Hash + Eq,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_seq(HashSetVisitor::new())
        }
    }
}

#[cfg(test)]
mod test {
    use alloc::{
        string::{String, ToString},
        vec::Vec,
    };

    use crate::{HashMap, HashSet};

    #[test]
    fn deserialize_map() {
        let json = r#"
            {
                "three": 3,
                "seven": 7
            }
        "#;

        let map = serde_json::from_str::<HashMap<String, i32>>(json).unwrap();

        assert_eq!(
            map,
            HashMap::from_iter([("three".to_string(), 3), ("seven".to_string(), 7)])
        );
    }

    #[test]
    fn serialize_map() {
        let map = HashMap::from_iter([("three".to_string(), 3), ("seven".to_string(), 7)]);

        let json = serde_json::to_string(&map).unwrap();

        let possibilities = &[r#"{"three":3,"seven":7}"#, r#"{"seven":7,"three":3}"#];

        assert!(possibilities.contains(&json.as_str()));
    }

    #[test]
    fn deserialize_hashset() {
        let json = "[1, 2, 5, 8, 9, 3, 4]";
        let set = serde_json::from_str::<HashSet<i32>>(json).unwrap();

        assert_eq!(set, HashSet::from_iter([1, 2, 3, 4, 5, 8, 9]));
    }

    #[test]
    fn serialize_hashset() {
        let set = HashSet::from_iter([1, 2, 3, 5, 8, 9, 10]);
        let serialized = serde_json::to_string(&set).unwrap();

        let mut deserialized = serde_json::from_str::<Vec<i32>>(&serialized).unwrap();
        deserialized.sort();

        assert_eq!(deserialized, &[1, 2, 3, 5, 8, 9, 10]);
    }
}
