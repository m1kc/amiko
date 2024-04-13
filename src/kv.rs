use std::collections::HashMap;

// Implementation of key-value storage in Rust.
// Keys and values are both sequences of bytes.
pub struct KeyValueStorage {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl KeyValueStorage {
    pub fn new() -> Self {
        KeyValueStorage {
            data: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.data.get(key)
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.data.remove(key)
    }
}
