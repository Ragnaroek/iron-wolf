pub mod assets;
pub mod vga_render;
pub mod vl;
pub mod config;
pub mod input;
pub mod time;
pub mod game;
pub mod user;
pub mod util;

use std::sync::Arc;
use std::io::prelude::*;
use std::fs::File;
use std::thread;

use vgaemu::screen;
use vgaemu::{SCReg, set_vertical_display_end};

use assets::{GraphicNum};
use vga_render::Renderer;

fn main() -> Result<(), String> {

    let iw_config = config::load_iw_config();

    let vga = vgaemu::new(0x13);
	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
	set_vertical_display_end(&vga, 480);

    let graphics = assets::load_all_graphics(&iw_config)?;

    init_game(&vga);

    let config = config::load_wolf_config(&iw_config);

    let prj = game::new_projection_config(&config);

    let input_monitoring = vgaemu::input::new_input_monitoring();

    let vga_screen = Arc::new(vga);
    let render = vga_render::init(vga_screen.clone(), graphics);
    let time = time::init();
    let input = input::init(Arc::new(time), input_monitoring.clone());

	thread::spawn(move || { 
        demo_loop(&render, &input, &prj, &iw_config);
    });

	let options: screen::Options = vgaemu::screen::Options {
		show_frame_rate: true,
        input_monitoring: Some(input_monitoring),
		..Default::default()
	};
	screen::start(vga_screen, options).unwrap();
    Ok(())
}

fn init_game(vga: &vgaemu::VGA) {
    vl::set_palette(vga, assets::GAMEPAL);
    signon_screen(vga);
}

fn demo_loop(rdr: &dyn Renderer, input: &input::Input, prj: &game::ProjectionConfig, iw_config: &config::IWConfig) {
    if !iw_config.no_wait {
        pg_13(rdr, input);
    }

    loop {
        while !iw_config.no_wait { // title screen & demo loop
            rdr.pic(0, 0, GraphicNum::TITLEPIC);
            rdr.fade_in();
            if input.user_input(time::TICK_BASE*15) {
                break;
            }
            rdr.fade_out();

            rdr.pic(0,0, GraphicNum::CREDITSPIC);
            rdr.fade_in();
            if input.user_input(time::TICK_BASE*10) {
                break;
            }
            rdr.fade_out();
         
            //TODO DrawHighScore() here
            //TODO PlayDemo() here
        }

        game::game_loop(rdr, input, prj);
        rdr.fade_out();
    }
}

fn signon_screen(vga: &vgaemu::VGA) {
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
    input.user_input(time::TICK_BASE*7);
    rdr.fade_out();
}