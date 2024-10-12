#![feature(test)]

extern crate test;
extern crate iw;

use std::path::PathBuf;
use std::env;
use std::sync::Arc;
use iw::loader::DiskLoader;
use test::Bencher;

use iw::def::IWConfig;
use iw::draw::{wall_refresh, init_ray_cast_consts, init_ray_cast};
use iw::game::setup_game_level;
use iw::assets;
use iw::vga_render;
use iw::play::{self, new_game_state};

#[bench]
fn bench_ray_cast_loop(b: &mut Bencher) -> Result<(), String> {
    let w3d_path = env::var("W3D_DATA").unwrap();
    let mut path = PathBuf::new();
    path.push(&w3d_path);

    let iw_config = IWConfig {
        wolf3d_data: path,
        no_wait: true,
    };
    let loader = DiskLoader{
        data_path: iw_config.wolf3d_data.clone(),
    };
    let assets = assets::load_assets(&loader)?;
    let prj = play::calc_projection(19);
    let (graphics, fonts, tiles) = assets::load_all_graphics(&loader)?;

    let vga = vga::new(0x13);
    let vga_screen = Arc::new(vga);
    let render = vga_render::init(vga_screen.clone(), graphics, fonts, tiles);

    let mut game_state = new_game_state();

    let mut level_state = setup_game_level(&prj, &mut game_state, &assets).unwrap();

    let player = level_state.player();
    let mut rc = init_ray_cast(prj.view_width);
    let consts = init_ray_cast_consts(&prj, player, 0);

    b.iter(|| {
        for _ in 0..1000 {
            wall_refresh(&mut level_state, &mut rc, &consts, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2283678; 
            player.y = 3446039;
            player.angle = 98; 
        }
        for _ in 0..1000 {
            wall_refresh(&mut level_state, &mut rc, &consts, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2263965; 
            player.y = 2428470;
            player.angle = 90; 
        }
        for _ in 0..1000 {
            wall_refresh(&mut level_state, &mut rc, &consts, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2263965; 
            player.y = 2061034;
            player.angle = 334; 
        }
        for _ in 0..1000 {
            wall_refresh(&mut level_state, &mut rc, &consts, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2246274; 
            player.y = 833690;
            player.angle = 159; 
        }
        for _ in 0..1000 {
            wall_refresh(&mut level_state, &mut rc, &consts, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2859077; 
            player.y = 678021;
            player.angle = 290; 
        }
        for _ in 0..1000 {
            wall_refresh(&mut level_state, &mut rc, &consts, &render, &prj, &assets);
        }
    });
    
    Ok(())
}