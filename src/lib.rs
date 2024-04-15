mod resp;
use resp::*;
pub mod kv;
use kv::Database;

use std::net::{Shutdown, TcpListener, TcpStream};
use std::io::*;
use std::sync::{Arc, RwLock};


#[cfg(debug_assertions)]
const VERBOSE: bool = true;
#[cfg(not(debug_assertions))]
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
		if leading_byte.is_ascii_alphabetic() {
			log!("Inline mode? No way");
			stream.write_all("-ERR inline mode not supported\r\n".as_bytes()).unwrap();
			skip_line(&mut stream).unwrap();
			continue;
		}
		// Commands always come as arrays, so reject other types.
		if leading_byte != TYPE_ARRAY {
			log!("Expected array type, got: {}", leading_byte);
			stream.write_all("-ERR expected array type\r\n".as_bytes()).unwrap();
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
			stream.write_all("-ERR negative array length\r\n".as_bytes()).unwrap();
			stream.flush().unwrap();
			stream.shutdown(Shutdown::Both).unwrap();
			return;
		}
		let argc = num_of_elements - 1;

		// Now, read the actual command.
		let _cmd = resp_expect_bulk_string(&mut stream).unwrap();
		let cmd: &str = std::str::from_utf8(&_cmd).unwrap();
		match (cmd, argc) {
			("PING", 0) | ("PING", 1) => {
				log!("PING");
				if argc == 0 {
					stream.write_all(b"+PONG\r\n").unwrap();
				} else {
					let msg = resp_expect_bulk_string(&mut stream).unwrap();
					resp_write_bulk_string(&mut stream, &msg).unwrap();
				}
			},
			("ECHO", 1) => {
				log!("ECHO");
				let msg = resp_expect_bulk_string(&mut stream).unwrap();
				resp_write_bulk_string(&mut stream, &msg).unwrap();
			},
			("QUIT", 0) => {
				log!("QUIT");
				stream.write_all(b"+OK\r\n").unwrap();
				stream.flush().unwrap();
				stream.shutdown(Shutdown::Both).unwrap();
				return;
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
			("FLUSHDB", 0) | ("FLUSHDB", 1) => {
				log!("FLUSHDB");
				// ignore the argument (always flush synchronously)
				if argc == 1 {
					resp_expect_bulk_string(&mut stream).unwrap();
				}
				storage.write().unwrap().clear();
				stream.write_all(b"+OK\r\n").unwrap();
			},
			_ => {
				log!("Bad command or wrong number of args: {:?}", _cmd);
				stream.write_all("-ERR bad command or wrong number of args\r\n".as_bytes()).unwrap();
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
	println!("Server listening on {}", &l);

	let mut _storage = Database::new();
	let storage = Arc::new(RwLock::new(_storage));

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				log!("Client connected: {}", stream.peer_addr().unwrap());
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
