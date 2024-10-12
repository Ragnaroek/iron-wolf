use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::Read;
use crate::assets::{file_name, WolfFile, WolfVariant};
use crate::patch::{self, PatchConfig};

// TODO Improve err handling by returning a Result on each function
pub trait Loader: Sync + Send {
    fn variant(&self) -> &'static WolfVariant;

    fn load_wolf_file(&self, file: WolfFile) -> Vec<u8>;
    fn load_patch_config_file(&self) -> Option<PatchConfig>;
    fn load_patch_data_file(&self, name: String) -> Vec<u8>;

    /// Read the first 32 bytes of a save game file
    fn load_save_game_head(&self, which: usize) -> Result<Vec<u8>, String>;
    fn load_save_game(&self, which: usize) -> Result<Vec<u8>, String>;
}

pub struct DiskLoader {
    pub variant: &'static WolfVariant,
    pub data_path: PathBuf,
    pub patch_path: Option<PathBuf>,
}

impl Loader for DiskLoader {
    fn variant(&self) -> &'static WolfVariant {
        return self.variant;
    }

    fn load_wolf_file(&self, file: WolfFile) -> Vec<u8> {
        let name = file_name(file, &self.variant);
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

    fn load_save_game_head(&self, which: usize) -> Result<Vec<u8>, String> {
        let mut file = self.open_save_game_file(which)?;
        let mut result = vec![0; 32];
        file.read_exact(result.as_mut_slice()).expect("savegame file header read");
        Ok(result)
    }

    fn load_save_game(&self, which: usize) -> Result<Vec<u8>, String> {
        let mut file = self.open_save_game_file(which)?;
        let mut result = Vec::new();
        file.read_to_end(&mut result).map_err(|e|format!("failed to read save game {}", e.to_string()))?;
        Ok(result)
    }

}

impl DiskLoader {
    fn open_save_game_file(&self, which: usize) -> Result<File, String> {
        let path = &self.data_path.join(format!("SAVEGAM{}.{}", which, self.variant.file_ending));
        let file_result = File::open(path);
        file_result.map_err(|_|format!("savegame {:?} not found", path))
    }
}

// loads a file completely, panics if it cannot be found or read
fn load_file(path: &Path) -> Vec<u8> {
	let mut file = File::open(path).expect(&format!("file not found: {:?}", path));
	let mut data = Vec::new();
	file.read_to_end(&mut data).unwrap();
	data
}