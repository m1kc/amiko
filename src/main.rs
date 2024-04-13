mod kv;
use kv::Database;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};


fn read_byte(stream: &mut TcpStream) -> Result<u8, std::io::Error> {
	let mut buffer = [0; 1]; // 1 byte buffer
	stream.read(&mut buffer)?;

	const DEBUG_READ_BYTE: bool = false;
	if DEBUG_READ_BYTE {
		let ch: String = match buffer[0] {
			b'\n' => "LF".to_string(),
			b'\r' => "CR".to_string(),
			_ => format!("{}", buffer[0] as char),
		};
		println!("Read byte: {} (ascii: {})", buffer[0], ch);
	}

	Ok(buffer[0])
}


fn read_byte_and_expect(stream: &mut TcpStream, expected: u8) -> Result<(), std::io::Error> {
	let b = read_byte(stream)?;
	if b != expected {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Expected byte {}, got {}", expected, b)));
	}
	Ok(())
}


fn resp_read_array_header(stream: &mut TcpStream) -> Result<u64, std::io::Error> {
	read_byte_and_expect(stream, b'*')?;
	let mut ret: u64 = 0;
	loop {
		let b = read_byte(stream)?;
		if b == b'\r' {
			break;
		}
		assert!(b >= b'0' && b <= b'9', "Invalid character encountered: {}", b as char);
		ret = ret * 10 + (b - b'0') as u64;
	}
	read_byte_and_expect(stream, b'\n')?;
	return Ok(ret);
}


fn resp_expect_bulk_string(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
	read_byte_and_expect(stream, b'$')?;
	let mut len: u64 = 0;
	loop {
		let b = read_byte(stream)?;
		if b == b'\r' {
			break;
		}
		assert!(b >= b'0' && b <= b'9', "Invalid character encountered: {}", b as char);
		len = len * 10 + (b - b'0') as u64;
	}
	read_byte_and_expect(stream, b'\n')?;
	let mut buf = vec![0; len as usize];
	stream.read_exact(&mut buf)?;
	read_byte_and_expect(stream, b'\r')?;
	read_byte_and_expect(stream, b'\n')?;

	println!("Read bulk string: {:?}", buf);
	let buf_as_string = String::from_utf8_lossy(&buf);
	println!("Bulk string as UTF-8 string: {}", buf_as_string);

	return Ok(buf);
}


fn handle_client(mut stream: TcpStream) {
	// TODO: inline mode isn't implemented yet.

	let mut storage = Database::new();

	loop {
		let num_of_elements = resp_read_array_header(&mut stream).unwrap();
		println!("Okay, {} elements", num_of_elements);
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
				storage.insert(key, value);
				stream.write_fmt(format_args!("+OK\r\n")).unwrap();
			}
			("GET", 2) => {
				let key = resp_expect_bulk_string(&mut stream).unwrap();
				println!("GET key: {:?}", key);
				match storage.get(&key) {
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
				match storage.remove(&key) {
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
				let keys = storage.search_keys(&prefix);
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
	// println!("HUGE SUCCESS");

	const LISTEN_ADDR: &str = "127.0.0.1";
	const LISTEN_PORT: u16 = 6379;
	let l: String = format!("{}:{}", LISTEN_ADDR, LISTEN_PORT);
	let listener = TcpListener::bind(&l).expect("Failed to bind to address");
	println!("Server listening on {}", &l);

	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				println!("New client connected");
				std::thread::spawn(move || {
					handle_client(stream);
				});
			}
			Err(err) => {
				println!("Failed to accept client connection: {}", err);
			}
		}
	}
}
