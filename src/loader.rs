use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use crate::assets::{file_name, WolfFile, WolfVariant};
use crate::patch::{self, PatchConfig};

// TODO Improve err handling by returning a Result
pub trait Loader {
    fn load_wolf_file(&self, file: WolfFile, variant: &WolfVariant) -> Vec<u8>;
    fn load_patch_config_file(&self) -> Option<PatchConfig>;
    fn load_patch_data_file(&self, name: String) -> Vec<u8>;
}

pub struct DiskLoader {
    pub data_path: PathBuf,
    pub patch_path: Option<PathBuf>,
}

impl Loader for DiskLoader {
    fn load_wolf_file(&self, file: WolfFile, variant: &WolfVariant) -> Vec<u8> {
        let name = file_name(file, variant);
        load_file(&self.data_path.join(name))
    }
    fn load_patch_config_file(&self) -> Option<PatchConfig> {
        if self.patch_path.is_none() {
            return None;
        }
        Some(patch::load_patch_config_file(&self.patch_path.as_ref().unwrap().join("patch.toml")).unwrap())
    }
    // panics, if patch path is not set
    fn load_patch_data_file(&self, name: String) -> Vec<u8> {
        load_file(&self.patch_path.as_ref().expect("no patch path configured").join(name))
    }
}

// loads a file completely, panics if it cannot be found or read
fn load_file(path: &Path) -> Vec<u8> {
	let mut file = File::open(path).expect(&format!("file not found: {:?}", path));
	let mut data = Vec::new();
	file.read_to_end(&mut data).unwrap();
	data
}