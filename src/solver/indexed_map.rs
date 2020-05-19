#[derive(Debug, Clone, PartialEq)]
pub struct IndexedMap<K, V> {
    size: usize,
    values: Vec<V>,
    keys: Vec<Option<K>>,
}

impl<K: Indexed + Clone, V: Default + Clone> IndexedMap<K, V> {
    pub fn new(size: usize) -> Self {
        IndexedMap {
            size,
            values: vec![V::default(); size],
            keys: vec![None; size],
        }
    }
}

pub trait Indexed {
    fn idx(&self) -> usize;
}

pub trait Map<K: Indexed, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V>;
    fn remove(&mut self, key: &K) -> Option<V>;
    fn is_empty(&self) -> bool;
    fn keys(&self) -> IterSome<K>;
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;
    fn entry(&mut self, key: K) -> Entry<K, V>;
    fn iter(&self) -> Iter<K, V>;
}

pub struct IterSome<'a, T> {
    values: &'a Vec<Option<T>>,
    idx: usize,
}

impl<'a, T> Iterator for IterSome<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.values.len() {
            let idx = self.idx;
            self.idx += 1;
            if let Some(Some(value)) = self.values.get(idx) {
                return Some(value);
            }
        }

        None
    }
}

pub struct Iter<'a, K, V> {
    idx: usize,
    keys: &'a Vec<Option<K>>,
    values: &'a Vec<V>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < self.keys.len() {
            let idx = self.idx;
            self.idx += 1;

            if let Some(Some(key)) = self.keys.get(idx) {
                let value = self.values.get(idx).unwrap();
                return Some((key, value));
            }
        }

        None
    }
}

pub struct Entry<'a, K, V> {
    key: K,
    key_ref: &'a mut Option<K>,
    value: &'a mut V,
}

impl<'a, K: Clone, V> Entry<'a, K, V> {
    pub fn or_default(&mut self) -> &mut V {
        if self.key_ref.is_none() {
            *self.key_ref = Some(self.key.clone());
        }

        self.value
    }
}

impl<K: Indexed, V: Clone + Default> Map<K, V> for IndexedMap<K, V> {
    fn insert(&mut self, key: K, value: V) -> Option<V> {
        let idx = key.idx();
        if idx >= self.size {
            panic!("Index out of bounds, index value for key is bigger than the map capacity.");
        }

        if let Some(_) = self.keys.get(idx) {
            let old_val = std::mem::take(&mut self.values[idx]);
            self.values[idx] = value;
            self.keys[idx] = Some(key);
            Some(old_val)
        } else {
            self.values[idx] = value;
            None
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let idx = key.idx();
        if idx >= self.size {
            panic!("Index out of bounds, index value for key is bigger than the map capacity.");
        }

        if let Some(_) = self.keys[idx] {
            self.keys[idx] = None;
            return Some(std::mem::take(&mut self.values[idx]));
        }

        None
    }

    fn is_empty(&self) -> bool {
        self.keys.iter().all(|key| key.is_none())
    }

    fn keys(&self) -> IterSome<K> {
        IterSome {
            values: &self.keys,
            idx: 0,
        }
    }

    fn get(&self, key: &K) -> Option<&V> {
        if let Some(Some(_)) = self.keys.get(key.idx()) {
            return Some(&self.values[key.idx()]);
        }
        None
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if let Some(Some(_)) = self.keys.get(key.idx()) {
            return Some(&mut self.values[key.idx()]);
        }
        None
    }

    fn entry(&mut self, key: K) -> Entry<K, V> {
        let idx = key.idx();
        if idx >= self.size {
            panic!("Index out of bounds, index value for key is bigger than the map capacity.");
        }
        Entry {
            key,
            key_ref: &mut self.keys[idx],
            value: &mut self.values[idx],
        }
    }

    fn iter(&self) -> Iter<K, V> {
        Iter {
            idx: 0,
            keys: &self.keys,
            values: &self.values,
        }
    }
}
