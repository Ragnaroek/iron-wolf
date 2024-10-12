pub mod assets;
pub mod vga_render;
pub mod vl;
pub mod config;
pub mod input;
pub mod time;

use std::sync::Arc;
use std::io::prelude::*;
use std::fs::File;
use std::thread;
use std::path::Path;

use vgaemu::screen;
use vgaemu::{SCReg, set_vertical_display_end};

use assets::{GraphicNum};
use vga_render::Renderer;
use config::Config;

fn main() -> Result<(), String> {

    let config = load_config();

    let vga = vgaemu::new(0x13);
	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
	set_vertical_display_end(&vga, 480);

    let graphics = assets::load_all_graphics(&config)?;

    init_game(&vga);

    let vga_screen = Arc::new(vga);
    let render = vga_render::init(vga_screen.clone(), graphics);
    let time = time::init();
    let input = input::init(Arc::new(time));

	thread::spawn(move || { 
        // TODO Wait for key press instead
        //thread::sleep(time::Duration::from_secs(3));
        pg_13(&render, &input);
    });

    // TODO game loop

	let options: screen::Options = vgaemu::screen::Options {
		show_frame_rate: true,
		..Default::default()
	};
	screen::start(vga_screen, options).unwrap();
    Ok(())
}

fn init_game(vga: &vgaemu::VGA) {
    vl::set_palette(vga, assets::GAMEPAL);
    signon_screen(vga);
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
    
    rdr.bar(0, 0, 320, 200, 0x10); //TODO just a demo
    //TODO wait for user input and fade_out

    input.user_input(time::TICK_BASE*9000);
}

fn load_config() -> Config {
    //TODO load from file
    Config {
        wolf3d_data: Path::new("/Users/mb/_w3d/w3d_data")
    }
}