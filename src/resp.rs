use std::net::TcpStream;
use std::io::Read;


const CR: u8 = b'\r';
const LF: u8 = b'\n';
const TYPE_ARRAY: u8 = b'*';
const TYPE_BULK_STRING: u8 = b'$';


pub fn read_byte(stream: &mut TcpStream) -> Result<u8, std::io::Error> {
	let mut buffer = [0; 1]; // 1 byte buffer
	stream.read_exact(&mut buffer)?;

	const DEBUG_READ_BYTE: bool = false;
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


pub fn read_byte_and_expect(stream: &mut TcpStream, expected: u8) -> Result<(), std::io::Error> {
	let b = read_byte(stream)?;
	if b != expected {
		return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Expected byte {}, got {}", expected, b)));
	}
	Ok(())
}


pub fn resp_read_array_header(stream: &mut TcpStream) -> Result<u64, std::io::Error> {
	read_byte_and_expect(stream, TYPE_ARRAY)?;
	let mut ret: u64 = 0;
	loop {
		let b = read_byte(stream)?;
		if b == CR {
			break;
		}
		assert!(b >= b'0' && b <= b'9', "Invalid character encountered: {}", b as char);
		ret = ret * 10 + (b - b'0') as u64;
	}
	read_byte_and_expect(stream, LF)?;
	return Ok(ret);
}


pub fn resp_expect_bulk_string(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
	read_byte_and_expect(stream, TYPE_BULK_STRING)?;
	let mut len: u64 = 0;
	loop {
		let b = read_byte(stream)?;
		if b == CR {
			break;
		}
		assert!(b >= b'0' && b <= b'9', "Invalid character encountered: {}", b as char);
		len = len * 10 + (b - b'0') as u64;
	}
	read_byte_and_expect(stream, LF)?;
	let mut buf = vec![0; len as usize];
	stream.read_exact(&mut buf)?;
	read_byte_and_expect(stream, CR)?;
	read_byte_and_expect(stream, LF)?;

	println!("Read bulk string: {:?}", buf);
	let buf_as_string = String::from_utf8_lossy(&buf);
	println!("Bulk string as UTF-8 string: {}", buf_as_string);

	return Ok(buf);
}
