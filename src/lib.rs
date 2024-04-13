mod resp;
use resp::*;
mod kv;
use kv::Database;

use std::net::{Shutdown, TcpListener, TcpStream};
use std::io::*;
use std::sync::{Arc, RwLock};


const VERBOSE: bool = false;


// This macro accepts the same arguments as println!.
macro_rules! log {
	($($arg:tt)*) => {
		if VERBOSE {
			println!($($arg)*);
		}
	}
}


fn handle_client(mut stream: TcpStream, storage: Arc<RwLock<Database>>) {
	loop {
		// Read the leading byte to determine the type of the next RESP message.
		// If read failed, just stop the thread.
		let leading_byte = match read_byte(&mut stream) {
			Ok(b) => b,
			Err(_) => return,
		};
		// If we get something ASCII, the person is using inline mode over telnet and we don't support that (yet).
		if (leading_byte >= b'A' && leading_byte <= b'Z') || (leading_byte >= b'a' && leading_byte <= b'z') {
			log!("Inline mode? No way");
			stream.write("-ERR inline mode not supported\r\n".as_bytes()).unwrap();
			skip_line(&mut stream).unwrap();
			continue;
		}
		// Commands always come as arrays, so reject other types.
		if leading_byte != TYPE_ARRAY {
			log!("Expected array type, got: {}", leading_byte);
			stream.write("-ERR expected array type\r\n".as_bytes()).unwrap();
			stream.flush().unwrap();
			stream.shutdown(Shutdown::Both).unwrap();
			return;
		}

		// Next, read array length.
		let num_of_elements = read_number(&mut stream).unwrap();
		log!("Got command - {} elements", num_of_elements);
		// Zero length is weird but valid I guess.
		if num_of_elements == 0 {
			continue;
		}
		// Sub-zero length is not valid, close the connection.
		if num_of_elements < 0 {
			stream.write("-ERR negative array length\r\n".as_bytes()).unwrap();
			stream.flush().unwrap();
			stream.shutdown(Shutdown::Both).unwrap();
			return;
		}
		let argc = num_of_elements - 1;

		// Now, read the actual command.
		let _cmd = resp_expect_bulk_string(&mut stream).unwrap();
		let cmd: &str = std::str::from_utf8(&_cmd).unwrap();
		match (cmd, argc) {
			("COMMAND", 1) => {
				// COMMAND DOCS stub
				let _arg = resp_expect_bulk_string(&mut stream).unwrap();
				stream.write_fmt(format_args!("*0\r\n")).unwrap();
			},
			("SET", 2) => {
				let key = resp_expect_bulk_string(&mut stream).unwrap();
				let value = resp_expect_bulk_string(&mut stream).unwrap();
				log!("SET key: {:?}, value: {:?}", key, value);
				storage.write().unwrap().insert(key, value);
				stream.write_fmt(format_args!("+OK\r\n")).unwrap();
			}
			("GET", 1) => {
				let key = resp_expect_bulk_string(&mut stream).unwrap();
				log!("GET key: {:?}", key);
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
				log!("DEL key: {:?}", key);
				match storage.write().unwrap().remove(&key) {
					Some(_) => {
						stream.write_all(b":1\r\n").unwrap();
					}
					None => {
						stream.write_all(b":0\r\n").unwrap();
					}
				}
			}
			("KEYS", 1) => {
				let prefix = resp_expect_bulk_string(&mut stream).unwrap();
				log!("KEYS with pattern: {:?}", prefix);
				let keys = storage.read().unwrap().search_keys(&prefix);
				stream.write_fmt(format_args!("*{}\r\n", keys.len())).unwrap();
				for key in keys {
					stream.write_fmt(format_args!("${}\r\n", key.len())).unwrap();
					stream.write_all(&key).unwrap();
					stream.write_all(b"\r\n").unwrap();
				}
			}
			_ => {
				log!("Unknown command: {:?}", _cmd);
				stream.write("-ERR unknown command\r\n".as_bytes()).unwrap();
				stream.flush().unwrap();
				for _ in 0..argc {
					resp_expect_bulk_string(&mut stream).unwrap();
				}
			}
		}
	}
}


pub fn serve() {
	const LISTEN_ADDR: &str = "127.0.0.1";
	const LISTEN_PORT: u16 = 6379;
	let l: String = format!("{}:{}", LISTEN_ADDR, LISTEN_PORT);
	let listener = TcpListener::bind(&l).expect("Failed to bind to address");
	log!("Server listening on {}", &l);

	let mut _storage = Database::new();
	let storage = Arc::new(RwLock::new(_storage));

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				log!("New client connected");
				let storage = storage.clone();
				std::thread::spawn(move || {
					handle_client(stream, storage);
				});
			}
			Err(err) => {
				log!("Failed to accept client connection: {}", err);
			}
		}
	}
}
