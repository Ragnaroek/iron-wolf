#![feature(test)]

extern crate iw;
extern crate test;

use iw::sd::test_sound;
use std::path::PathBuf;
use test::Bencher;

use vga::VGABuilder;

use iw::assets;
use iw::config::default_iw_config;
use iw::def::new_game_state;
use iw::draw::{init_ray_cast, wall_refresh};
use iw::game::setup_game_level;
use iw::loader::DiskLoader;
use iw::rc::{Input, RenderContext};
use iw::start::new_view_size;
use iw::time::new_test_ticker;

#[bench]
fn bench_ray_cast_loop(b: &mut Bencher) -> Result<(), String> {
    let mut data_path = PathBuf::new();
    data_path.push("./testdata/shareware_data");

    let mut iw_config = default_iw_config()?;
    iw_config.data.wolf3d_data = data_path;

    let loader = DiskLoader {
        variant: &assets::W3D1,
        data_path: iw_config.data.wolf3d_data.clone(),
        patch_path: iw_config.data.patch_data,
    };

    let assets = assets::load_graphic_assets(&loader)?;

    let prj = new_view_size(19);
    let (graphics, fonts, tiles, texts) = assets::load_all_graphics(&loader, &None)?;

    let sound = test_sound();

    let vga = VGABuilder::new().video_mode(0x13).build()?;
    let input = Input::init_demo_playback(Vec::with_capacity(0));
    let ticker = new_test_ticker();
    let mut rc = RenderContext::init(
        vga,
        ticker,
        graphics,
        fonts,
        tiles,
        texts,
        assets,
        &assets::W3D1,
        input,
        prj,
        sound,
    );

    let mut game_state = new_game_state();

    let mut level_state = setup_game_level(&mut game_state, &rc.assets, true).unwrap();

    let player = level_state.player();
    let mut cast = init_ray_cast(rc.projection.view_width);
    cast.init_ray_cast_consts(&rc.projection, player, 0);

    b.iter(|| {
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state, &mut cast);
        }
        {
            let player = level_state.mut_player();
            player.x = 2283678;
            player.y = 3446039;
            player.angle = 98;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state, &mut cast);
        }
        {
            let player = level_state.mut_player();
            player.x = 2263965;
            player.y = 2428470;
            player.angle = 90;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state, &mut cast);
        }
        {
            let player = level_state.mut_player();
            player.x = 2263965;
            player.y = 2061034;
            player.angle = 334;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state, &mut cast);
        }
        {
            let player = level_state.mut_player();
            player.x = 2246274;
            player.y = 833690;
            player.angle = 159;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state, &mut cast);
        }
        {
            let player = level_state.mut_player();
            player.x = 2859077;
            player.y = 678021;
            player.angle = 290;
        }
        for _ in 0..1000 {
            wall_refresh(&mut rc, &game_state, &mut level_state, &mut cast);
        }
    });

    Ok(())
}
