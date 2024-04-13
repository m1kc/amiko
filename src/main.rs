mod kv;
use kv::Database;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};


fn read_byte(stream: &mut TcpStream) -> Result<u8, std::io::Error> {
	let mut buffer = [0; 1]; // 1 byte buffer
	stream.read(&mut buffer)?;

	let ch: String = match buffer[0] {
		b'\n' => "LF".to_string(),
		b'\r' => "CR".to_string(),
		_ => format!("{}", buffer[0] as char),
	};
	println!("Read byte: {} (ascii: {})", buffer[0], ch);

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


fn handle_client(mut stream: TcpStream) {
	// TODO: inline mode isn't implemented yet.

	loop {
		let num_of_elements = resp_read_array_header(&mut stream).unwrap();
		println!("Okay, {} elements", num_of_elements);
	}
}


fn main() {
	println!("HUGE SUCCESS");

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
