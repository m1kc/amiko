use std::collections::HashMap;

// Implementation of key-value storage in Rust.
// Keys and values are both sequences of bytes.
pub struct KeyValueStorage {
	data: HashMap<Vec<u8>, Vec<u8>>,
}

impl KeyValueStorage {
	fn new() -> Self {
		KeyValueStorage {
			data: HashMap::new(),
		}
	}

	fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
		self.data.insert(key, value);
	}

	fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
		self.data.get(key)
	}

	fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
		self.data.remove(key)
	}
}


fn main() {
	println!("Hello, world!");

	let mut storage = KeyValueStorage::new();
	let key: Vec<u8> = "Carl".as_bytes().to_vec();
	let val: Vec<u8> = "good".as_bytes().to_vec();
	storage.insert(key, val);

	println!("got value: {:?}", storage.get("Carl".as_bytes()));

	let k_val = storage.get("Carl".as_bytes());
	if let Some(value) = k_val {
		if let Ok(value_str) = std::str::from_utf8(value) {
			println!("value as string: {}", value_str);
		} else {
			println!("Failed to convert value to UTF-8 string");
		}
	} else {
		println!("Key not found");
	}

	storage.remove("Carl".as_bytes());

	let k_val = storage.get("Carl".as_bytes());
	if let Some(value) = k_val {
		if let Ok(value_str) = std::str::from_utf8(value) {
			println!("value as string: {}", value_str);
		} else {
			println!("Failed to convert value to UTF-8 string");
		}
	} else {
		println!("Key not found");
	}
}
