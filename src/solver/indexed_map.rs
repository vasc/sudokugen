#[derive(Debug, Clone, PartialEq, Eq)]
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.keys.len();
        (size, Some(size))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.idx += n;

        self.next()
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

        if let Some(Some(_)) = self.keys.get(idx) {
            let old_val = std::mem::replace(&mut self.values[idx], value);

            self.keys[idx] = Some(key);
            Some(old_val)
        } else {
            self.values[idx] = value;
            self.keys[idx] = Some(key);
            None
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        let idx = key.idx();
        if idx >= self.size {
            panic!("Index out of bounds, index value for key is bigger than the map capacity.");
        }

        if self.keys[idx].is_some() {
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

#[cfg(test)]
mod test {
    use super::{Indexed, IndexedMap, Map};

    impl Indexed for usize {
        fn idx(&self) -> usize {
            *self
        }
    }

    #[test]
    fn insert() {
        let mut map = IndexedMap::new(1);
        let old = map.insert(0, ());
        assert_eq!(old, None);
        assert_eq!(map.get(&0), Some(&()));
    }
    #[test]
    fn remove() {
        let mut map = IndexedMap::new(1);
        map.insert(0, ());

        assert_eq!(map.remove(&0), Some(()));
        assert_eq!(map.get(&0), None);
    }
    #[test]
    fn is_empty() {
        let mut map = IndexedMap::new(1);

        assert!(map.is_empty());
        map.insert(0, ());
        assert!(!map.is_empty());
    }
    #[test]
    fn keys() {
        let mut map = IndexedMap::new(4);
        map.insert(0, 0_i32);
        map.insert(3, 1_i32);

        assert_eq!(map.keys().collect::<Vec<&usize>>(), vec![&0, &3]);
    }
    #[test]
    fn get() {
        let mut map = IndexedMap::new(1);
        map.insert(0, ());
        assert_eq!(map.get(&0), Some(&()));
    }
    #[test]
    fn get_mut() {
        let mut map = IndexedMap::new(1);
        map.insert(0, 0);

        let val = map.get_mut(&0).unwrap();
        assert_eq!(val, &0);
        *val = 1;
        assert_eq!(map.get(&0), Some(&1));
    }
    #[test]
    fn entry_is_mut() {
        let mut map = IndexedMap::new(1);
        map.insert(0, Some(0));
        let mut entry = map.entry(0);
        let value = entry.or_default();

        assert_eq!(value, &mut Some(0));

        *value = Some(1);
        assert_eq!(map.get(&0), Some(&Some(1)));
    }

    #[test]
    fn entry_default() {
        let mut map: IndexedMap<usize, Option<()>> = IndexedMap::new(1);
        let mut entry = map.entry(0);

        assert_eq!(entry.or_default(), &mut None);
    }

    #[test]
    fn iter() {
        let mut map = IndexedMap::new(2);
        map.insert(0, 0_i32);
        map.insert(1, 1_i32);

        assert_eq!(
            map.iter().collect::<Vec<(&usize, &i32)>>(),
            vec![(&0, &0), (&1, &1)]
        );
    }
}
