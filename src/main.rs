mod resp;
use resp::*;
mod kv;
use kv::Database;

use std::net::{TcpListener, TcpStream};
use std::io::*;
use std::sync::{Arc, RwLock};


fn handle_client(mut stream: TcpStream, storage: Arc<RwLock<Database>>) {
	// TODO: inline mode isn't implemented yet.

	loop {
		let num_of_elements = resp_read_array_header(&mut stream).unwrap();
		println!("Got command - {} elements", num_of_elements);
		assert!(num_of_elements > 0);
		let cmd = resp_expect_bulk_string(&mut stream).unwrap();
		let cmd_as_string: &str = std::str::from_utf8(&cmd).unwrap();
		match (cmd_as_string, num_of_elements) {
			("COMMAND", 2) => {
				let _arg = resp_expect_bulk_string(&mut stream).unwrap();
				stream.write_fmt(format_args!("*0\r\n")).unwrap();
			},
			("SET", 3) => {
				let key = resp_expect_bulk_string(&mut stream).unwrap();
				let value = resp_expect_bulk_string(&mut stream).unwrap();
				println!("SET key: {:?}, value: {:?}", key, value);
				storage.write().unwrap().insert(key, value);
				stream.write_fmt(format_args!("+OK\r\n")).unwrap();
			}
			("GET", 2) => {
				let key = resp_expect_bulk_string(&mut stream).unwrap();
				println!("GET key: {:?}", key);
				match storage.read().unwrap().get(&key) {
					Some(value) => {
						stream.write_fmt(format_args!("${}\r\n", value.len())).unwrap();
						stream.write_all(&value).unwrap();
						stream.write_all(b"\r\n").unwrap();
					}
					None => {
						stream.write_all(b"$-1\r\n").unwrap();
					}
				}
			}
			("DEL", 1) => {
				let key = resp_expect_bulk_string(&mut stream).unwrap();
				println!("DEL key: {:?}", key);
				match storage.write().unwrap().remove(&key) {
					Some(_) => {
						stream.write_all(b":1\r\n").unwrap();
					}
					None => {
						stream.write_all(b":0\r\n").unwrap();
					}
				}
			}
			("KEYS", 2) => {
				let prefix = resp_expect_bulk_string(&mut stream).unwrap();
				println!("KEYS prefix: {:?}", prefix);
				let keys = storage.read().unwrap().search_keys(&prefix);
				stream.write_fmt(format_args!("*{}\r\n", keys.len())).unwrap();
				for key in keys {
					stream.write_fmt(format_args!("${}\r\n", key.len())).unwrap();
					stream.write_all(&key).unwrap();
					stream.write_all(b"\r\n").unwrap();
				}
			}
			_ => {
				println!("Unknown command: {:?}", cmd);
			}
		}
	}
}


fn main() {
	const LISTEN_ADDR: &str = "127.0.0.1";
	const LISTEN_PORT: u16 = 6379;
	let l: String = format!("{}:{}", LISTEN_ADDR, LISTEN_PORT);
	let listener = TcpListener::bind(&l).expect("Failed to bind to address");
	println!("Server listening on {}", &l);

	let mut _storage = Database::new();
	let storage = Arc::new(RwLock::new(_storage));

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				println!("New client connected");
				let storage = storage.clone();
				std::thread::spawn(move || {
					handle_client(stream, storage);
				});
			}
			Err(err) => {
				println!("Failed to accept client connection: {}", err);
			}
		}
	}
}
