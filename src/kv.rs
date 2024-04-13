use std::{collections::HashMap, sync::RwLock};

// Implementation of key-value storage in Rust.
// Keys and values are both sequences of bytes.
// It's thread-safe!
pub struct Database {
    data: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
			data: RwLock::new(HashMap::new()),
        }
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
		let mut db = self.data.write().unwrap();
		db.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		let db = self.data.read().unwrap();
        db.get(key).cloned()
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
		let mut db = self.data.write().unwrap();
		db.remove(key)
    }

	pub fn search_keys(&self, pattern: &[u8]) -> Vec<Vec<u8>> {
		let db = self.data.read().unwrap();
		let keys = db.keys().cloned();
		match pattern {
			[b'*'] => keys.collect(),
			_ => keys.filter(|key| key.starts_with(pattern)).collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_insert_and_get() {
		let mut storage = Database::new();
		let key = vec![1, 2, 3];
		let value = vec![4, 5, 6];

		storage.insert(key.clone(), value.clone());

		assert_eq!(storage.get(&key), Some(value));
	}

	#[test]
	fn test_insert_and_remove() {
		let mut storage = Database::new();
		let key = vec![1, 2, 3];
		let value = vec![4, 5, 6];

		storage.insert(key.clone(), value.clone());

		assert_eq!(storage.remove(&key), Some(value));
		assert_eq!(storage.get(&key), None);
	}

	#[test]
	fn test_search_keys() {
		let mut storage = Database::new();

		storage.insert("Alleria".as_bytes().to_vec(), vec![]);
		storage.insert("Alan".as_bytes().to_vec(), vec![]);
		storage.insert("Boris".as_bytes().to_vec(), vec![]);

		let result = storage.search_keys("Al".as_bytes());
		assert_eq!(result.len(), 2);
		assert!(result.contains(&&"Alleria".as_bytes().to_vec()));
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*".as_bytes());
		assert_eq!(result.len(), 3);
		assert!(result.contains(&&"Alleria".as_bytes().to_vec()));
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));
		assert!(result.contains(&&"Boris".as_bytes().to_vec()));

		let result = storage.search_keys("B".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Boris".as_bytes().to_vec()));
	}
}
