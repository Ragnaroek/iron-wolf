use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use crate::assets::{WolfFile, file_name};

pub trait Loader {
    fn load_file(&self, file: WolfFile) -> Vec<u8>;
}

pub struct DiskLoader {
    pub data_path: PathBuf,
}

impl Loader for DiskLoader {
    fn load_file(&self, file: WolfFile) -> Vec<u8> {
        let name = file_name(file);
        load_file(&self.data_path.join(name))
    }
}

// loads a file completely, panics if it cannot be found or read
fn load_file(path: &Path) -> Vec<u8> {
	let mut file = File::open(path).unwrap();
	let mut data = Vec::new();
	file.read_to_end(&mut data).unwrap();
	data
}

// TODO Impl Disk loader && Web load for Loader
// Disk loader from file on request
// Web loader, uses preloaded files and returns them?