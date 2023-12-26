use std::sync::Arc;

use vga::SCReg;
use vga::util::spawn_task;

use crate::def::{Assets, UserState};
use crate::assets::{GraphicNum, SIGNON, GAMEPAL};
use crate::assets;
use crate::def::IWConfig;
use crate::loader::Loader;
use crate::config;
use crate::play;
use crate::vl;
use crate::vga_render::{self, VGARenderer};
use crate::time;
use crate::input;
use crate::game::game_loop;

pub fn iw_start(loader: &dyn Loader, iw_config: IWConfig) -> Result<(), String> {
    let config = config::load_wolf_config(loader);

    let vga = vga::new(0x13);
	//enable Mode Y
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let (graphics, fonts) = assets::load_all_graphics(loader)?;
    let assets = assets::load_assets(loader)?;

    let mut user_state = init_game(&vga);

    // TODO calc_projection and setup_scaling have to be re-done if view size changes in config
    let prj = play::calc_projection(config.viewsize as usize);

    let input_monitoring = vga::input::new_input_monitoring();

    let vga_screen = Arc::new(vga);
    let vga_loop = vga_screen.clone();
    let render = vga_render::init(vga_screen.clone(), graphics, fonts);
    let ticker = time::new_ticker();
    let input = input::init(ticker.time_count.clone(), input_monitoring.clone());

	spawn_task(async move { 
        demo_loop(&iw_config, ticker, &vga_loop, &render, &input, &prj, &assets, &mut user_state).await;
    });

	let options: vga::Options = vga::Options {
		show_frame_rate: true,
        input_monitoring: Some(input_monitoring),
		..Default::default()
	};
    vga_screen.start(options).unwrap();
    Ok(())
}

fn init_game(vga: &vga::VGA) -> UserState {
    vl::set_palette(vga, GAMEPAL);
    signon_screen(vga);

    // TODO InitRedShifts
    finish_signon()
}

fn finish_signon() -> UserState {
    UserState {
        window_x: 0,
        window_y: 0,
        window_w: 320,
        window_h: 160,
    }
}

async fn demo_loop(config: &IWConfig, ticker: time::Ticker, vga: &vga::VGA, rdr: &VGARenderer, input: &input::Input, prj: &play::ProjectionConfig, assets: &Assets, user_state: &mut UserState) {
    if !config.no_wait {
        pg_13(rdr, input).await;
    }

    loop {
        while !config.no_wait { // title screen & demo loop
            rdr.pic(0, 0, GraphicNum::TITLEPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*15).await {
                break;
            }
            rdr.fade_out().await;

            rdr.pic(0,0, GraphicNum::CREDITSPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*10).await {
                break;
            }
            rdr.fade_out().await;
         
            //TODO DrawHighScore() here
            //TODO PlayDemo() here
        }

        game_loop(&ticker, vga, rdr, input, prj, assets, user_state).await;
        rdr.fade_out().await;
    }
}

fn signon_screen(vga: &vga::VGA) {
    let mut buf_offset = 0;
    let mut vga_offset = 0;
    while buf_offset < SIGNON.len()-4 {
        vga.set_sc_data(SCReg::MapMask, 1);
        vga.write_mem(vga_offset, SIGNON[buf_offset]);

		vga.set_sc_data(SCReg::MapMask, 2);
        vga.write_mem(vga_offset, SIGNON[buf_offset+1]);
		
		vga.set_sc_data(SCReg::MapMask, 4);
        vga.write_mem(vga_offset, SIGNON[buf_offset+2]);

		vga.set_sc_data(SCReg::MapMask, 8);
        vga.write_mem(vga_offset, SIGNON[buf_offset+3]);

        vga_offset += 1;
        buf_offset += 4;
    }
}

async fn pg_13(rdr: &VGARenderer, input: &input::Input) {
    rdr.fade_out().await; 
    rdr.bar(0, 0, 320, 200, 0x82);
    rdr.pic(216, 110, GraphicNum::PG13PIC);
    
    rdr.fade_in().await;
    input.wait_user_input(time::TICK_BASE*7).await;
    rdr.fade_out().await;
}