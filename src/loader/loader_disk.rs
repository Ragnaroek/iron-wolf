use crate::assets::{WolfFile, WolfVariant, file_name};
use crate::patch::{self, PatchConfig};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub struct Loader {
    pub variant: &'static WolfVariant,
    pub data_path: PathBuf,
    pub patch_path: Option<PathBuf>,
}

// TODO Improve err handling by returning a Result on each function
impl Loader {
    pub fn variant(&self) -> &'static WolfVariant {
        return self.variant;
    }

    pub fn write_wolf_file(&self, file: WolfFile, data: &[u8]) -> Result<(), String> {
        let name = file_name(file, &self.variant);
        let path = &self.data_path.join(name);
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(data).map_err(|e| e.to_string())
    }

    pub fn load_wolf_file(&self, file: WolfFile) -> Vec<u8> {
        let name = file_name(file, &self.variant);
        load_file(&self.data_path.join(name))
    }

    pub fn load_wolf_file_slice(
        &self,
        file: WolfFile,
        offset: u64,
        len: usize,
    ) -> Result<Vec<u8>, String> {
        let name = file_name(file, &self.variant);
        let mut file = File::open(&self.data_path.join(name)).map_err(|e| e.to_string())?;
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        let mut buf = vec![0; len];
        file.read_exact(&mut buf).map_err(|e| e.to_string())?;
        Ok(buf)
    }

    pub fn load_patch_config_file(&self) -> Result<Option<PatchConfig>, String> {
        if self.patch_path.is_none() {
            return Ok(None);
        }

        if let Some(ref_path) = &self.patch_path.as_ref() {
            let loaded_file = patch::load_patch_config_file(&ref_path.join("patch.toml"))?;
            return Ok(Some(loaded_file));
        }

        Err("error loading patch file".to_string())
    }
    // panics, if patch path is not set
    pub fn load_patch_data_file(&self, name: String) -> Vec<u8> {
        load_file(
            &self
                .patch_path
                .as_ref()
                .expect("no patch path configured")
                .join(name),
        )
    }

    pub async fn load_save_game_head(&self, which: usize) -> Result<Vec<u8>, String> {
        let mut file = self.open_save_game_file(which)?;
        let mut result = vec![0; 32];
        file.read_exact(result.as_mut_slice())
            .expect("savegame file header read");
        Ok(result)
    }

    pub async fn load_save_game(&self, which: usize) -> Result<Vec<u8>, String> {
        let mut file = self.open_save_game_file(which)?;
        let mut result = Vec::new();
        file.read_to_end(&mut result)
            .map_err(|e| format!("failed to read save game {}", e.to_string()))?;
        Ok(result)
    }

    pub async fn save_save_game(&self, which: usize, bytes: &[u8]) -> Result<(), String> {
        let mut file = self.create_save_game_file(which)?;
        file.write_all(bytes)
            .map_err(|e| format!("failed to save save game {}", e.to_string()))
    }

    // helper functions

    fn open_save_game_file(&self, which: usize) -> Result<File, String> {
        let path = &self.save_game_path(which);
        let file_result = File::open(path);
        file_result.map_err(|_| format!("savegame {:?} not found", path))
    }

    fn create_save_game_file(&self, which: usize) -> Result<File, String> {
        let path = &self.save_game_path(which);
        let file_result = File::create(path);
        file_result.map_err(|_| format!("savegame {:?} cannot be created", path))
    }

    fn save_game_path(&self, which: usize) -> PathBuf {
        self.data_path
            .join(format!("SAVEGAM{}.{}", which, self.variant.file_ending))
    }
}

// loads a file completely, panics if it cannot be found or read
fn load_file(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).expect(&format!("file not found: {:?}", path));
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();
    data
}
