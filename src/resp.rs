use std::io::{Read, Write};

// #[cfg(not(test))]
use std::net::TcpStream;
// #[cfg(test)]
// use stub::MockTcpStream as TcpStream;


const DEBUG_READ_BYTE: bool = false;

#[cfg(debug_assertions)]
const DEBUG_READ_BULK_STRING: bool = true;
#[cfg(not(debug_assertions))]
const DEBUG_READ_BULK_STRING: bool = false;

pub const CR: u8 = b'\r';
pub const LF: u8 = b'\n';
pub const TYPE_ARRAY: u8 = b'*';
pub const TYPE_BULK_STRING: u8 = b'$';


#[cfg(never)]
pub mod stub {
	pub struct MockTcpStream {
		data: Vec<u8>,
		cursor: usize,
	}
	impl MockTcpStream {
		pub fn new(data: Vec<u8>) -> Self {
			MockTcpStream {
				data: data,
				cursor: 0,
			}
		}

		pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), std::io::Error> {
			if self.cursor + buf.len() > self.data.len() {
				return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Unexpected EOF"));
			}
			buf.copy_from_slice(&self.data[self.cursor..self.cursor + buf.len()]);
			self.cursor += buf.len();
			Ok(())
		}

		pub fn shutdown(&self, _: std::net::Shutdown) -> Result<(), std::io::Error> {
			Ok(())
		}

		pub fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
			self.data.extend_from_slice(buf);
			Ok(())
		}
	}
}


pub fn read_byte(stream: &mut TcpStream) -> Result<u8, std::io::Error> {
	let mut buffer = [0; 1]; // 1 byte buffer
	stream.read_exact(&mut buffer)?;

	if DEBUG_READ_BYTE {
		let ch: String = match buffer[0] {
			CR => "CR".to_string(),
			LF => "LF".to_string(),
			_ => format!("{}", buffer[0] as char),
		};
		println!("Read byte: {} (ascii: {})", buffer[0], ch);
	}

	Ok(buffer[0])
}


#[cfg(never)]
mod test {
	/*
	use mockall_double::double;

	use crate::read_byte;

	#[double]
	use super::stub::TcpStream;

	#[test]
	fn test_read_byte() {
		let expected = [b'A', b'B', b'C'];
		let mut stream = TcpStream::new(expected.to_vec());
		assert_eq!(read_byte(&mut stream).unwrap(), b'A');
		assert_eq!(read_byte(&mut stream).unwrap(), b'B');
		assert_eq!(read_byte(&mut stream).unwrap(), b'C');
		assert!(read_byte(&mut stream).is_err());
	}
	*/
}


pub fn read_byte_and_expect(stream: &mut TcpStream, expected: u8) -> Result<(), std::io::Error> {
	let b = read_byte(stream)?;
	if b != expected {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Expected byte {}, got {}", expected, b)));
	}
	Ok(())
}


pub fn skip_line(stream: &mut TcpStream) -> Result<(), std::io::Error> {
	loop {
		let b = read_byte(stream)?;
		if b == CR {
			break;
		}
	}
	read_byte_and_expect(stream, LF)?;
	Ok(())
}


pub fn read_number(stream: &mut TcpStream) -> Result<i64, std::io::Error> {
	let mut ret: i64 = 0;
	let mut sign = 1;
	loop {
		let b = read_byte(stream)?;
		if b == b'-' {
			sign = -1;
			continue;
		}
		if b == CR {
			break;
		}
		assert!(b.is_ascii_digit(), "Invalid character encountered: {}", b as char);
		ret = ret * 10 + (b - b'0') as i64;
	}
	read_byte_and_expect(stream, LF)?;
	Ok(ret * sign)
}


#[allow(dead_code)]
pub fn read_number_unsigned(stream: &mut TcpStream) -> Result<u64, std::io::Error> {
	let buf = read_number(stream)?;
	if buf < 0 {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Expected unsigned number, got negative"));
	}
	Ok(buf as u64)
}


#[allow(dead_code)]
pub fn resp_read_array_header(stream: &mut TcpStream) -> Result<u64, std::io::Error> {
	read_byte_and_expect(stream, TYPE_ARRAY)?;
	let mut ret: u64 = 0;
	loop {
		let b = read_byte(stream)?;
		if b == CR {
			break;
		}
		assert!(b.is_ascii_digit(), "Invalid character encountered: {}", b as char);
		ret = ret * 10 + (b - b'0') as u64;
	}
	read_byte_and_expect(stream, LF)?;
	Ok(ret)
}


pub fn resp_expect_bulk_string(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
	read_byte_and_expect(stream, TYPE_BULK_STRING)?;
	let mut len: u64 = 0;
	loop {
		let b = read_byte(stream)?;
		if b == CR {
			break;
		}
		assert!(b.is_ascii_digit(), "Invalid character encountered: {}", b as char);
		len = len * 10 + (b - b'0') as u64;
	}
	read_byte_and_expect(stream, LF)?;
	let mut buf = vec![0; len as usize];
	stream.read_exact(&mut buf)?;
	read_byte_and_expect(stream, CR)?;
	read_byte_and_expect(stream, LF)?;

	if DEBUG_READ_BULK_STRING {
		println!("Read bulk string, length {}", len);
		match std::str::from_utf8(&buf) {
			Ok(s) => {
				println!("  valid UTF-8: {}", s)
			},
			Err(_) => {
				println!("  not UTF-8: {:?}", buf)
			}
		}
	}

	Ok(buf)
}


pub fn resp_write_bulk_string(stream: &mut TcpStream, s: &[u8]) -> Result<(), std::io::Error> {
	stream.write_all(&[TYPE_BULK_STRING])?;
	stream.write_all(format!("{}\r\n", s.len()).as_bytes())?;
	stream.write_all(s)?;
	stream.write_all(&[CR, LF])?;
	Ok(())
}
