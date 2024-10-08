extern crate iw;

use iw::assets;
use iw::config::read_iw_config;
use iw::loader::DiskLoader;
use iw::start::iw_start;

fn main() -> Result<(), String> {
    let variant = &assets::W3D6; // TODO determine this with conditional compilation
    let iw_config = read_iw_config()?;
    let loader = DiskLoader {
        variant,
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path: iw_config.data.patch_data.clone(),
    };
    iw_start(loader, iw_config)
}
