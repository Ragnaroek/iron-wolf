extern crate iw;

use iw::start::iw_start;
use iw::config;
use iw::loader::DiskLoader;

fn main() -> Result<(), String> {
    let iw_config = config::default_iw_config();
    let loader = DiskLoader{
        data_path: iw_config.wolf3d_data.clone(),
    };
    iw_start(&loader, iw_config)
}