extern crate time;

use std::sync::{ Arc, Mutex };
use std::hash::Hash;
use std::borrow::Borrow;
use std::collections::{ hash_map, HashMap };

pub struct LruCache<K, V: Send> {
  limit: usize,
  map: Mutex<HashMap<K, CacheEntry<V>>>,
}

struct CacheEntry<V> {
  last_access: u64,
  arc: Arc<V>,
}

impl<K: Clone + Hash + Eq, V: Send> LruCache<K, V> {
  pub fn with_limit(limit: usize) -> LruCache<K, V> {
    assert!(limit != 0);
    LruCache {
      limit: limit,
      map: Mutex::new(HashMap::with_capacity(limit))
    }
  }

  pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<Arc<V>>
      where K: Borrow<Q>, Q: Hash + Eq {
    if let Some(entry) = self.map.lock().unwrap().get_mut(k) {
      entry.last_access = time::precise_time_ns();
      Some(entry.arc.clone())
    } else {
      None
    }
  }

  pub fn insert(&self, k: K, v: V) -> Option<Arc<V>> {
    let new_entry = CacheEntry {
      last_access: time::precise_time_ns(),
      arc: Arc::new(v),
    };

    let mut map = self.map.lock().unwrap();
    if map.len() == self.limit {
      let oldest = map.iter().min_by_key(|&(_, entry)| entry.last_access).unwrap().0.clone();
      map.remove(&oldest);
    }

    let old_entry = match map.entry(k) {
      hash_map::Entry::Occupied(mut entry) => {
        Some(entry.insert(new_entry))
      },
      hash_map::Entry::Vacant(entry) => {
        entry.insert(new_entry);
        None
      },
    };

    old_entry.map(|entry| entry.arc)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn smoke() {
    let cash = LruCache::with_limit(5);
    cash.insert(0u8, 0u8);
    cash.insert(1, 1);
    cash.insert(2, 2);
    cash.insert(3, 3);
    cash.insert(4, 4);
    cash.insert(5, 5);
    assert_eq!(cash.get(&0), None);
    assert_eq!(cash.get(&1).map(|a| *a), Some(1));
  }

  #[test]
  fn smoke2() {
    let cash = LruCache::with_limit(5);
    cash.insert(0u8, 0u8);
    cash.insert(1, 1);
    cash.insert(2, 2);
    cash.insert(3, 3);
    cash.insert(4, 4);
    assert_eq!(cash.get(&0).map(|a| *a), Some(0));
    cash.insert(5, 5);
    assert_eq!(cash.get(&1).map(|a| *a), None);
  }
}
