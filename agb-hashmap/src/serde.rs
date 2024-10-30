use core::{hash::Hash, marker::PhantomData};
use serde::{
    de::{MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};

use crate::{ClonableAllocator, HashMap};

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

#[derive(Default)]
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
