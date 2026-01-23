extern crate iw;

use iw::assets::derive_variant;
use iw::config::read_iw_config;
use iw::loader::DiskLoader;
use iw::start::iw_start;

fn main() -> Result<(), String> {
    let iw_config = read_iw_config()?;
    let variant = derive_variant(&iw_config)?;
    let loader = DiskLoader {
        variant,
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path: iw_config.data.patch_data.clone(),
    };

    iw_start(loader, iw_config)
}
