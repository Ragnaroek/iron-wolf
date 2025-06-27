#![feature(test)]

extern crate iw;
extern crate test;

use iw::start::new_view_size;
use vga::VGABuilder;

use iw::config::default_iw_config;
use iw::loader::DiskLoader;
use std::path::PathBuf;
use std::sync::Arc;
use test::Bencher;

use iw::assets;
use iw::def::new_game_state;
use iw::draw::{init_ray_cast, wall_refresh};
use iw::game::setup_game_level;
use iw::vga_render;

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

    let vga = VGABuilder::new().video_mode(0x13).build_no_backend();
    let vga_screen = Arc::new(vga);
    let render = vga_render::init(
        vga_screen.clone(),
        graphics,
        fonts,
        tiles,
        texts,
        &assets::W3D1,
    );

    let mut game_state = new_game_state();

    let mut level_state = setup_game_level(&mut game_state, &assets, true).unwrap();

    let player = level_state.player();
    let mut rc = init_ray_cast(prj.view_width);
    rc.init_ray_cast_consts(&prj, player, 0);

    b.iter(|| {
        for _ in 0..1000 {
            wall_refresh(
                &game_state,
                &mut level_state,
                &mut rc,
                &render,
                &prj,
                &assets,
            );
        }
        {
            let player = level_state.mut_player();
            player.x = 2283678;
            player.y = 3446039;
            player.angle = 98;
        }
        for _ in 0..1000 {
            wall_refresh(
                &game_state,
                &mut level_state,
                &mut rc,
                &render,
                &prj,
                &assets,
            );
        }
        {
            let player = level_state.mut_player();
            player.x = 2263965;
            player.y = 2428470;
            player.angle = 90;
        }
        for _ in 0..1000 {
            wall_refresh(
                &game_state,
                &mut level_state,
                &mut rc,
                &render,
                &prj,
                &assets,
            );
        }
        {
            let player = level_state.mut_player();
            player.x = 2263965;
            player.y = 2061034;
            player.angle = 334;
        }
        for _ in 0..1000 {
            wall_refresh(
                &game_state,
                &mut level_state,
                &mut rc,
                &render,
                &prj,
                &assets,
            );
        }
        {
            let player = level_state.mut_player();
            player.x = 2246274;
            player.y = 833690;
            player.angle = 159;
        }
        for _ in 0..1000 {
            wall_refresh(
                &game_state,
                &mut level_state,
                &mut rc,
                &render,
                &prj,
                &assets,
            );
        }
        {
            let player = level_state.mut_player();
            player.x = 2859077;
            player.y = 678021;
            player.angle = 290;
        }
        for _ in 0..1000 {
            wall_refresh(
                &game_state,
                &mut level_state,
                &mut rc,
                &render,
                &prj,
                &assets,
            );
        }
    });

    Ok(())
}
