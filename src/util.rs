pub struct DataReader<'a> {
	data: &'a [u8],
	offset: usize,
}

pub fn new_data_reader(data: &[u8]) -> DataReader {
	new_data_reader_with_offset(data, 0)
}

pub fn new_data_reader_with_offset(data: &[u8], offset: usize) -> DataReader {
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

	pub fn read_u8(&mut self) -> u8 {
		let u = self.data[self.offset];
		self.offset += 1;
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