extern crate iw;

#[cfg(not(feature = "web"))]
use iw::assets::derive_variant;
#[cfg(not(feature = "web"))]
use iw::config::read_iw_config;
#[cfg(not(feature = "web"))]
use iw::loader::Loader;
#[cfg(not(feature = "web"))]
use iw::start::iw_start;

#[cfg(not(feature = "web"))]
fn main() -> Result<(), String> {
    let iw_config = read_iw_config()?;
    let variant = derive_variant(&iw_config)?;
    let loader = Loader {
        variant,
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path: iw_config.data.patch_data.clone(),
    };

    iw_start(loader, iw_config)
}

#[cfg(feature = "web")]
fn main() {}
