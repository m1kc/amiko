use std::collections::HashMap;
use regex;

/// Redis-like key-value storage.
/// Keys and values are both sequences of bytes.
/// Not thread-safe.
pub struct Database {
	data: HashMap<Vec<u8>, Vec<u8>>,
}

impl Database {
	pub fn new() -> Self {
		Database {
			data: HashMap::new(),
		}
	}

	pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
		self.data.insert(key, value);
	}

	pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		self.data.get(key).cloned()
	}

	pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
		self.data.remove(key)
	}

	/// Supported glob-style patterns:
	///
    /// * `h?llo` matches `hello`, `hallo` and `hxllo`
    /// * `h*llo` matches `hllo` and `heeeello`
    /// * `h[ae]llo` matches `hello` and `hallo`, but not `hillo`
    /// * `h[^e]llo` matches `hallo`, `hbllo`, ... but not `hello`
    /// * `h[a-b]llo` matches `hallo` and `hbllo`
	pub fn search_keys(&self, pattern: &[u8]) -> Vec<Vec<u8>> {
		let keys = self.data.keys().cloned();
		match pattern {
			[b'*'] => keys.collect(),
			_ => {
				let mut regex_pattern = String::from("^");
				for &byte in pattern {
					match byte {
						// Globs (not filtered: [ ] - ^)
						b'*' => regex_pattern.push_str(".*?"),
						b'?' => regex_pattern.push_str("."),
						// Unsafe symbols (very naive filter, but that'll do for now)
						b'(' | b')' | b'{' | b'}' | b'+' | b'.' | b'\\' | b'$' | b'|' => {
							regex_pattern.push_str(format!("\\{}", byte as char).as_str())
						},
						// Everything else
						_ => regex_pattern.push(byte as char),
					}
				}
				regex_pattern.push('$');
				// println!("regex_pattern: {:?}", regex_pattern);
				let re = regex::Regex::new(&regex_pattern).unwrap();
				keys.filter(|key| re.is_match(&String::from_utf8_lossy(&key))).collect()
			},
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

		let result = storage.search_keys("Al*".as_bytes());
		assert_eq!(result.len(), 2);
		assert!(result.contains(&&"Alleria".as_bytes().to_vec()));
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*".as_bytes());
		assert_eq!(result.len(), 3);
		assert!(result.contains(&&"Alleria".as_bytes().to_vec()));
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));
		assert!(result.contains(&&"Boris".as_bytes().to_vec()));

		let result = storage.search_keys("B*".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Boris".as_bytes().to_vec()));

		let result = storage.search_keys("b*".as_bytes());
		assert_eq!(result.len(), 0);

		let result = storage.search_keys("A???".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("Ala[kn]".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("Ala[^z]".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("Ala[k-p]".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*la*".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*lan".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"Alan".as_bytes().to_vec()));

		// Overall validity
		storage.insert("K{v".as_bytes().to_vec(), vec![]);

		let result = storage.search_keys("K{v".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&&"K{v".as_bytes().to_vec()));
	}
}
