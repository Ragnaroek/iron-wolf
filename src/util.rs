use std::fs::File;
use std::io::Read;
use std::path::Path;

// loads a file completely, panics if it cannot be found or read
pub fn load_file(path: &Path) -> Vec<u8> {
	let mut file = File::open(path).unwrap();
	let mut data = Vec::new();
	file.read_to_end(&mut data).unwrap();
	data
}

pub struct DataReader<'a> {
	data: &'a Vec<u8>,
	offset: usize,
}

pub fn new_data_reader(data: &Vec<u8>) -> DataReader {
	new_data_reader_with_offset(data, 0)
}

pub fn new_data_reader_with_offset(data: &Vec<u8>, offset: usize) -> DataReader {
	DataReader {
		data,
		offset
	}
}

impl DataReader<'_> {
	pub fn read_utf8_string(&mut self, size: usize) -> String {
		let str = String::from_utf8_lossy(&self.data[self.offset..(self.offset+size)]).to_string();
		self.offset += size;
		str
	}

	pub fn read_u32(&mut self) -> u32 {
		let u = u32::from_le_bytes(self.data[self.offset..(self.offset+4)].try_into().unwrap());
		self.offset += 4;
		u
	}
	
	pub fn read_i32(&mut self) -> i32 {
		let i = i32::from_le_bytes(self.data[self.offset..(self.offset + 4)].try_into().unwrap());
		self.offset += 4;
		i
	}

	pub fn read_u16(&mut self) -> u16 {
		let u = u16::from_le_bytes(self.data[self.offset..(self.offset+2)].try_into().unwrap());
		self.offset += 2;
		u
	}

	pub fn read_bool(&mut self) -> bool {
		let u = self.read_u16();
		u != 0
	}

	// returns a slice over the bytes there were not read so far
	pub fn unread_bytes(&self) -> &[u8] {
		&self.data[self.offset..]
	}
}