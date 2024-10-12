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