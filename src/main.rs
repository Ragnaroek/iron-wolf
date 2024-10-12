extern crate iw;

use std::sync::Arc;
use std::io::prelude::*;
use std::fs::File;
use std::thread;

use vga::SCReg;
use libiw::assets::GAMEPAL;

use iw::def::Assets;
use iw::assets::GraphicNum;
use iw::vga_render::Renderer;
use iw::config;
use iw::assets;
use iw::play;
use iw::vl;
use iw::vga_render;
use iw::time;
use iw::input;

fn main() -> Result<(), String> {
    let iw_config = config::load_iw_config();
    let config = config::load_wolf_config(&iw_config);

    let vga = vga::new(0x13);
	//enable Mode Y
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even

    let graphics = assets::load_all_graphics(&iw_config)?;
    let assets = assets::load_assets(iw_config)?;

    init_game(&vga);

    // TODO calc_projection and setup_scaling have to be re-done if view size changes in config
    let prj = play::calc_projection(config.viewsize as usize);

    let input_monitoring = vga::input::new_input_monitoring();

    let vga_screen = Arc::new(vga);
    let vga_loop = vga_screen.clone();
    // TODO get rid of the Renderer abstraction and directly used VGA!
    let render = vga_render::init(vga_screen.clone(), graphics);
    let ticker = time::new_ticker();
    let input = input::init(ticker.time_count.clone(), input_monitoring.clone());

	thread::spawn(move || { 
        demo_loop(ticker, &vga_loop, &render, &input, &prj, &assets);
    });

	let options: vga::Options = vga::Options {
		show_frame_rate: true,
        input_monitoring: Some(input_monitoring),
		..Default::default()
	};
    vga_screen.start(options).unwrap();
    Ok(())
}

fn init_game(vga: &vga::VGA) {
    vl::set_palette(vga, GAMEPAL);
    signon_screen(vga);
}

fn demo_loop(ticker: time::Ticker, vga: &vga::VGA, rdr: &dyn Renderer, input: &input::Input, prj: &play::ProjectionConfig, assets: &Assets) {
    if !assets.iw_config.no_wait {
        pg_13(rdr, input);
    }

    loop {
        while !assets.iw_config.no_wait { // title screen & demo loop
            rdr.pic(0, 0, GraphicNum::TITLEPIC);
            rdr.fade_in();
            if input.wait_user_input(time::TICK_BASE*15) {
                break;
            }
            rdr.fade_out();

            rdr.pic(0,0, GraphicNum::CREDITSPIC);
            rdr.fade_in();
            if input.wait_user_input(time::TICK_BASE*10) {
                break;
            }
            rdr.fade_out();
         
            //TODO DrawHighScore() here
            //TODO PlayDemo() here
        }

        play::game_loop(&ticker, vga, rdr, input, prj, assets);
        rdr.fade_out();
    }
}

fn signon_screen(vga: &vga::VGA) {
    let mut f_signon = File::open("assets/signon.bin").unwrap();
    let mut signon_data = Vec::new();
    f_signon.read_to_end(&mut signon_data).unwrap();


    let mut buf_offset = 0;
    let mut vga_offset = 0;
    while buf_offset < signon_data.len()-4 {
        vga.set_sc_data(SCReg::MapMask, 1);
        vga.write_mem(vga_offset, signon_data[buf_offset]);

		vga.set_sc_data(SCReg::MapMask, 2);
        vga.write_mem(vga_offset, signon_data[buf_offset+1]);
		
		vga.set_sc_data(SCReg::MapMask, 4);
        vga.write_mem(vga_offset, signon_data[buf_offset+2]);

		vga.set_sc_data(SCReg::MapMask, 8);
        vga.write_mem(vga_offset, signon_data[buf_offset+3]);

        vga_offset += 1;
        buf_offset += 4;
    }
}

fn pg_13(rdr: &dyn Renderer, input: &input::Input) {
    rdr.fade_out(); 
    rdr.bar(0, 0, 320, 200, 0x82);
    rdr.pic(216, 110, GraphicNum::PG13PIC);
    
    rdr.fade_in();
    input.wait_user_input(time::TICK_BASE*7);
    rdr.fade_out();
}