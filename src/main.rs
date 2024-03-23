extern crate iw;

use iw::start::iw_start;
use iw::config::read_iw_config;
use iw::loader::DiskLoader;

fn main() -> Result<(), String> {
    let iw_config = read_iw_config()?;
    let loader = DiskLoader{
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path : iw_config.data.patch_data.clone(),
    };
    iw_start(&loader, iw_config)
}