use std::{collections::HashMap, hash::Hash};

pub struct BiMap<K, V> {
    kv: HashMap<K, V>,
    vk: HashMap<V, K>,
}

impl<K, V> BiMap<K, V> {
    pub fn new() -> Self {
        BiMap {
            kv: HashMap::new(),
            vk: HashMap::new(),
        }
    }
}

impl<K, V> BiMap<K, V>
where
    K: Eq + Hash + Clone,
    V: Eq + Hash + Clone,
{
    pub fn insert(&mut self, key: K, value: V) {
        self.vk.insert(value.clone(), key.clone());

        self.kv.insert(key, value);
    }
}

impl<K, V> BiMap<K, V>
where
    K: Eq + Hash,
{
    pub fn remove_by_key(&mut self, key: &K) -> Option<V> {
        self.kv.remove(key)
    }
}

impl<K, V> BiMap<K, V>
where
    V: Eq + Hash,
{
    pub fn remove_by_value(&mut self, val: &V) -> Option<K> {
        self.vk.remove(val)
    }
}

impl<K, V> BiMap<K, V>
where
    K: Eq + Hash,
{
    pub fn get_by_key(&self, key: &K) -> Option<&V> {
        self.kv.get(key)
    }
}

impl<K, V> BiMap<K, V>
where
    V: Eq + Hash,
{
    pub fn get_by_val(&self, val: &V) -> Option<&K> {
        self.vk.get(val)
    }
}

impl<K, V> BiMap<K, V>
where
    K: Eq + Hash,
{
    pub fn get_mut_by_key(&mut self, key: &K) -> Option<&mut V> {
        self.kv.get_mut(key)
    }
}

impl<K, V> BiMap<K, V>
where
    V: Eq + Hash,
{
    pub fn get_mut_by_val(&mut self, val: &V) -> Option<&mut K> {
        self.vk.get_mut(val)
    }
}
