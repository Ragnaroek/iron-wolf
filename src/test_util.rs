use std::path::PathBuf;

use vga::{SCReg, VGABuilder};

use crate::assets;
use crate::config;
use crate::draw::init_ray_cast;
use crate::loader::{DiskLoader, Loader};
use crate::rc::{Input, RenderContext};
use crate::sd;
use crate::start::new_view_size;
use crate::time::new_test_ticker;

#[cfg(feature = "test")]
pub fn start_test_iw(loader: &dyn Loader) -> RenderContext {
    let wolf_config = config::load_wolf_config(loader);
    let mut vga = VGABuilder::new()
        .video_mode(0x13)
        .build()
        .expect("VGA test instance");

    let sound = sd::test_sound();

    //enable Mode Y
    let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
    vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let assets = assets::load_graphic_assets(loader, &None).expect("load graphic assets");

    let projection = new_view_size(wolf_config.viewsize);
    let input = Input::init_demo_playback(Vec::with_capacity(0));
    let ticker = new_test_ticker();
    let cast = init_ray_cast(projection.view_width);
    let rc = RenderContext::init(
        vga,
        ticker,
        assets,
        loader.variant(),
        input,
        projection,
        cast,
        sound,
    );

    rc
}

#[cfg(feature = "test")]
pub fn test_context() -> RenderContext {
    let mut data_path = PathBuf::new();
    data_path.push("./testdata/shareware_data");

    start_test_iw(&DiskLoader {
        variant: &assets::W3D1,
        data_path,
        patch_path: None,
    })
}
