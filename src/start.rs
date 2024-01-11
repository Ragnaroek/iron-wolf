use std::sync::Arc;

use vga::SCReg;
use vga::input::NumCode;
use vga::util::spawn_task;

use crate::def::{Assets, WindowState};
use crate::assets::{GraphicNum, SIGNON, GAMEPAL};
use crate::assets;
use crate::def::IWConfig;
use crate::inter::draw_high_scores;
use crate::loader::Loader;
use crate::config;
use crate::menu::control_panel;
use crate::play;
use crate::us1::c_print;
use crate::vl;
use crate::vga_render::{self, VGARenderer};
use crate::time;
use crate::input::{self, Input};
use crate::game::game_loop;

pub fn iw_start(loader: &dyn Loader, iw_config: IWConfig) -> Result<(), String> {
    let config = config::load_wolf_config(loader);

    let vga = vga::new(0x13);
	//enable Mode Y
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let (graphics, fonts, tiles) = assets::load_all_graphics(loader)?;
    let assets = assets::load_assets(loader)?;

    // TODO calc_projection and setup_scaling have to be re-done if view size changes in config
    let prj = play::calc_projection(config.viewsize as usize);

    let ticker = time::new_ticker();
    let input_monitoring = vga::input::new_input_monitoring();
    let input = input::init(ticker.time_count.clone(), input_monitoring.clone());

    let mut win_state = initial_window_state();

    let vga_screen = Arc::new(vga);
    let vga_loop = vga_screen.clone();
    let rdr = vga_render::init(vga_screen.clone(), graphics, fonts, tiles);

	spawn_task(async move {
        init_game(&vga_loop, &rdr, &input, &mut win_state).await;
        demo_loop(&iw_config, ticker, &vga_loop, &rdr, &input, &prj, &assets, &mut win_state).await;
    });

	let options: vga::Options = vga::Options {
		show_frame_rate: true,
        input_monitoring: Some(input_monitoring),
		..Default::default()
	};
    vga_screen.start(options).unwrap();
    /*
    vga_screen.start_debug_planar_mode(
        1300,
        700,
        options, 
    ).unwrap();
    */

    Ok(())
}

fn initial_window_state() -> WindowState {
    WindowState {
        window_x: 0,
        window_y: 0,
        window_w: 320,
        window_h: 160,
        print_x: 0,
        print_y: 0,
        font_color: 0,
        font_number: 0,
        back_color: 0,
        debug_ok: false,
    }
}

async fn init_game(vga: &vga::VGA, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    vl::set_palette(vga, GAMEPAL);
    signon_screen(vga);

    // TODO InitRedShifts
    finish_signon(vga, rdr, input, win_state).await;
}

async fn finish_signon(vga: &vga::VGA, rdr: &VGARenderer, input: &Input, win_state: &mut WindowState) {
    let peek = vga.read_mem(0);
    rdr.bar(0, 189, 300, 11, peek);

    win_state.window_x = 0;
    win_state.window_w = 320;
    win_state.print_y = 190;
    win_state.set_font_color(14, 4);
    c_print(rdr, win_state, "Press a key");

    input.ack().await;

    rdr.bar(0, 189, 300, 11, peek);

    win_state.print_y = 190;
    win_state.set_font_color(10, 4);
    c_print(rdr, win_state, "Working...");

    win_state.set_font_color(0, 15);
}

async fn demo_loop(config: &IWConfig, ticker: time::Ticker, vga: &vga::VGA, rdr: &VGARenderer, input: &input::Input, prj: &play::ProjectionConfig, assets: &Assets, win_state: &mut WindowState) {
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

            // credits page
            rdr.pic(0,0, GraphicNum::CREDITSPIC);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*10).await {
                break;
            }
            rdr.fade_out().await;
        
            // high scores
            draw_high_scores(rdr);
            rdr.fade_in().await;
            if input.wait_user_input(time::TICK_BASE*10).await {
                break;
            }

            //TODO PlayDemo() here
        }

        rdr.fade_out().await;

        // TODO RecordDemo()
        control_panel(&ticker, rdr, input, win_state, NumCode::None).await;

        game_loop(&ticker, vga, rdr, input, prj, assets, win_state).await;
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