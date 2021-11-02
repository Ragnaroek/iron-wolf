pub mod assets;
pub mod vga_render;
pub mod vl;

use std::sync::Arc;
use std::io::prelude::*;
use std::fs::File;
use std::{thread, time};

use vgaemu::screen;
use vgaemu::{SCReg, set_vertical_display_end};

use vga_render::Renderer;

fn main() {
    let vga = vgaemu::new(0x13);

	//enable Mode X
	let mem_mode = vga.get_sc_data(SCReg::MemoryMode);
	vga.set_sc_data(SCReg::MemoryMode, (mem_mode & !0x08) | 0x04); //turn off chain 4 & odd/even
	set_vertical_display_end(&vga, 480);

    init_game(&vga);

    let vga_screen = Arc::new(vga);
    let render = vga_render::init(vga_screen.clone());

	thread::spawn(move || { 
        // TODO Wait for key press instead
        thread::sleep(time::Duration::from_secs(3));
        pg_13(&render);
    });

    // TODO game loop

	let options: screen::Options = vgaemu::screen::Options {
		show_frame_rate: true,
		..Default::default()
	};
	screen::start(vga_screen, options).unwrap();
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

fn pg_13(rdr: &dyn Renderer) {
    rdr.bar(0, 0, 320, 200, 0x82);
    //TODO draw pg13 pic from assets
}