use ahash::AHashMap;
use regex;

/// Redis-like key-value storage.
/// Keys and values are both sequences of bytes.
/// Not thread-safe.
pub struct Database {
	data: AHashMap<Vec<u8>, Vec<u8>>,
}

impl Default for Database {
	fn default() -> Self {
		Self::new()
	}
}

impl Database {
	pub fn new() -> Self {
		Database {
			data: AHashMap::new(),
		}
	}

	/// Sets the value of a key.
	/// If the key already exists, the previous value is overwritten.
	///
	/// ```
	/// # use amiko::kv::Database;
	/// let mut db = Database::new();
	///
	/// db.insert(vec![1, 2, 3], vec![4, 5, 6]);
	/// assert_eq!(db.get(&[1, 2, 3]), Some(vec![4, 5, 6]));
	///
	/// db.insert(vec![1, 2, 3], vec![9, 9, 9]);
	/// assert_eq!(db.get(&[1, 2, 3]), Some(vec![9, 9, 9]));
	/// ```
	pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
		self.data.insert(key, value);
	}

	/// Gets the value of a key.
	/// Returns `Some(value)` if the key exists, `None` otherwise.
	///
	/// ```
	/// # use amiko::kv::Database;
	/// let mut db = Database::new();
	///
	/// assert_eq!(db.get(&[1, 2, 3]), None);
	///
	/// db.insert(vec![1, 2, 3], vec![4, 5, 6]);
	/// assert_eq!(db.get(&[1, 2, 3]), Some(vec![4, 5, 6]));
	/// ```
	pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
		self.data.get(key).cloned()
	}

	/// Removes a key from the database.
	/// Returns the value of the key if it existed, `None` otherwise.
	///
	/// ```
	/// # use amiko::kv::Database;
	/// let mut db = Database::new();
	///
	/// let last_value = db.remove(&[1, 2, 3]);
	/// assert_eq!(last_value, None);
	///
	/// db.insert(vec![1, 2, 3], vec![4, 5, 6]);
	/// let last_value = db.remove(&[1, 2, 3]);
	/// assert_eq!(last_value, Some(vec![4, 5, 6]));
	/// assert_eq!(db.get(&[1, 2, 3]), None);
	/// ```
	pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
		self.data.remove(key)
	}

	/// Removes all keys from the database.
	///
	/// ```
	/// # use amiko::kv::Database;
	/// let mut db = Database::new();
	///
	/// db.insert(vec![1, 2, 3], vec![4, 5, 6]);
	/// db.insert(vec![2, 3, 4], vec![5, 6, 7]);
	/// db.clear();
	/// assert_eq!(db.get(&[1, 2, 3]), None);
	/// ```
	pub fn clear(&mut self) {
		self.data.clear();
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
						b'?' => regex_pattern.push('.'),
						// Unsafe symbols (very naive filter, but that'll do for now)
						b'(' | b')' | b'{' | b'}' | b'+' | b'.' | b'\\' | b'$' | b'|' => {
							regex_pattern.push('\\');
							regex_pattern.push(byte as char);
						},
						// Everything else
						_ => regex_pattern.push(byte as char),
					}
				}
				regex_pattern.push('$');
				// println!("regex_pattern: {:?}", regex_pattern);
				let re = regex::Regex::new(&regex_pattern).unwrap();
				keys.filter(|key| re.is_match(&String::from_utf8_lossy(key))).collect()
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
		assert!(result.contains(&"Alleria".as_bytes().to_vec()));
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*".as_bytes());
		assert_eq!(result.len(), 3);
		assert!(result.contains(&"Alleria".as_bytes().to_vec()));
		assert!(result.contains(&"Alan".as_bytes().to_vec()));
		assert!(result.contains(&"Boris".as_bytes().to_vec()));

		let result = storage.search_keys("B*".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Boris".as_bytes().to_vec()));

		let result = storage.search_keys("b*".as_bytes());
		assert_eq!(result.len(), 0);

		let result = storage.search_keys("A???".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("Ala[kn]".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("Ala[^z]".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("Ala[k-p]".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*la*".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		let result = storage.search_keys("*lan".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"Alan".as_bytes().to_vec()));

		// Overall validity
		storage.insert("K{v".as_bytes().to_vec(), vec![]);

		let result = storage.search_keys("K{v".as_bytes());
		assert_eq!(result.len(), 1);
		assert!(result.contains(&"K{v".as_bytes().to_vec()));
	}
}
