use std::env;

pub struct DataReader<'a> {
    data: &'a [u8],
    offset: usize,
}

impl DataReader<'_> {
    pub fn new(data: &[u8]) -> DataReader<'_> {
        DataReader::new_with_offset(data, 0)
    }

    pub fn new_with_offset(data: &[u8], offset: usize) -> DataReader<'_> {
        DataReader { data, offset }
    }

    pub fn read_utf8_string(&mut self, size: usize) -> String {
        let str =
            String::from_utf8_lossy(&self.data[self.offset..(self.offset + size)]).to_string();
        self.offset += size;
        str
    }

    pub fn read_u32(&mut self) -> u32 {
        let u = u32::from_le_bytes(
            self.data[self.offset..(self.offset + 4)]
                .try_into()
                .unwrap(),
        );
        self.offset += 4;
        u
    }

    pub fn read_i32(&mut self) -> i32 {
        let i = i32::from_le_bytes(
            self.data[self.offset..(self.offset + 4)]
                .try_into()
                .unwrap(),
        );
        self.offset += 4;
        i
    }

    pub fn read_u16(&mut self) -> u16 {
        let u = u16::from_le_bytes(
            self.data[self.offset..(self.offset + 2)]
                .try_into()
                .unwrap(),
        );
        self.offset += 2;
        u
    }

    pub fn read_i16(&mut self) -> i16 {
        let i = i16::from_le_bytes(
            self.data[self.offset..(self.offset + 2)]
                .try_into()
                .unwrap(),
        );
        self.offset += 2;
        i
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

    // returns a slice over the bytes that were not read so far
    pub fn unread_bytes(&self) -> &[u8] {
        &self.data[self.offset..]
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.data[start..end]
    }

    pub fn skip(&mut self, bytes: usize) {
        self.offset += bytes;
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub struct DataWriter {
    pub data: Vec<u8>,
    offset: usize,
}

impl DataWriter {
    pub fn new(size: usize) -> DataWriter {
        DataWriter {
            data: vec![0; size],
            offset: 0,
        }
    }

    pub fn write_utf8_string(&mut self, str: &str, size: usize) {
        let mut data = vec![0; size];

        let mut i = 0;
        for byte in str.as_bytes() {
            data[i] = *byte;
            i += 1;
            if i >= size {
                break;
            }
        }

        self.write_bytes(&data)
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.data[self.offset] = *byte;
            self.offset += 1;
        }
    }

    pub fn write_u8(&mut self, v: u8) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_u16(&mut self, v: u16) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_i16(&mut self, v: i16) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_u32(&mut self, v: u32) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_i32(&mut self, v: i32) {
        self.write_bytes(&v.to_le_bytes());
    }

    pub fn write_bool(&mut self, v: bool) {
        if v {
            self.write_u16(1);
        } else {
            self.write_u16(0);
        }
    }

    pub fn skip(&mut self, bytes: usize) {
        self.offset += bytes;
    }

    pub fn slice(&self, start: usize, end: usize) -> &[u8] {
        &self.data[start..end]
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

pub fn check_param(check: &str) -> bool {
    for arg in env::args() {
        let normal_arg: String = arg.chars().filter(|&c| c.is_alphanumeric()).collect();
        if normal_arg == check {
            return true;
        }
    }
    false
}
