mod kv;
use kv::KeyValueStorage;


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
