use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Deserialize)]
pub struct PatchConfig {
    pub graphics : toml::Table,
}

pub fn load_patch_config_file(path: &Path) -> Result<PatchConfig, String> {
    let contents = fs::read_to_string(path).map_err(|e|e.to_string())?;
    let file = toml::from_str(&contents).map_err(|e| e.to_string())?;
    Ok(file)
}

pub fn graphic_patch(config_opt: &Option<PatchConfig>, num: usize) -> Option<String> {
    if let Some(config) = config_opt {
        let patch_opt = config.graphics.get(&num.to_string());
        if let Some(toml::Value::String(file)) = patch_opt {
            return Some(file.to_owned())
        }
    }
    None
}
