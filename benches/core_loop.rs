#![feature(test)]

extern crate test;
extern crate iw;

use std::path::PathBuf;
use std::env;
use std::sync::Arc;
use test::Bencher;

use iw::def::IWConfig;
use iw::draw::wall_refresh;
use iw::game::setup_game_level;
use iw::assets;
use iw::vga_render;
use iw::play;

#[bench]
fn bench_ray_cast_loop(b: &mut Bencher) -> Result<(), String> {

    let w3d_path = env::var("W3D_DATA").unwrap();
    let mut path = PathBuf::new();
    path.push(&w3d_path);

    let iw_config = IWConfig {
        wolf3d_data: path,
        no_wait: true,
    };
    let assets = assets::load_assets(iw_config)?;
    let prj = play::calc_projection(19);
    let graphics = assets::load_all_graphics(&assets.iw_config)?;

    let vga = vgaemu::new(0x13);
    let vga_screen = Arc::new(vga);
    let render = vga_render::init(vga_screen.clone(), graphics);

    let mut level_state = setup_game_level(&prj, 0, &assets).unwrap();

    b.iter(|| {
        for _ in 0..1000 {
            wall_refresh(&level_state, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2283678; 
            player.y = 3446039;
            player.angle = 98; 
        }
        for _ in 0..1000 {
            wall_refresh(&level_state, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2263965; 
            player.y = 2428470;
            player.angle = 90; 
        }
        for _ in 0..1000 {
            wall_refresh(&level_state, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2263965; 
            player.y = 2061034;
            player.angle = 334; 
        }
        for _ in 0..1000 {
            wall_refresh(&level_state, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2246274; 
            player.y = 833690;
            player.angle = 159; 
        }
        for _ in 0..1000 {
            wall_refresh(&level_state, &render, &prj, &assets);
        }
        {   
            let player = level_state.mut_player();
            player.x = 2859077; 
            player.y = 678021;
            player.angle = 290; 
        }
        for _ in 0..1000 {
            wall_refresh(&level_state, &render, &prj, &assets);
        }
    });
    
    Ok(())
}